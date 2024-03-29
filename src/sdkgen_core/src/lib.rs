mod non_empty_string;

pub use non_empty_string::*;

use indexmap::map::IntoIter;
use indexmap::IndexMap;

pub trait CasingRules<T: ToOwned> {
    fn to_type_name_case(&self, identifier: T) -> T::Owned;
    fn to_record_member_case(&self, identifier: T) -> T::Owned;
    fn to_function_name_case(&self, identifier: T) -> T::Owned;
}

pub trait GenerateSdk {
    fn generate_sdk(&self, types: TypeDeclarations, versions: Vec<SdkVersion>) -> String;
}

#[derive(Debug, Clone)]
pub enum Primitive {
    String,
    Boolean,
    Integer,
    Float,
    Double,
}

#[derive(Debug, Clone)]
pub enum Type {
    Primitive(Primitive),
    Array(Box<Type>),
    Map { key: Box<Type>, value: Box<Type> },
    Union { name: String, cases: Vec<String> },
    Record { name: String, members: Vec<Member> },
}

impl Type {
    pub fn name(&self) -> Option<&str> {
        match self {
            Type::Union { name, .. } | Type::Record { name, .. } => Some(name),
            Type::Primitive(_) | Type::Array(_) | Type::Map { .. } => None,
        }
    }

    pub fn set_name<N: Into<String>>(mut self, name: N) -> Self {
        let new_name = name;

        match self {
            Type::Union { ref mut name, .. } | Type::Record { ref mut name, .. } => {
                *name = new_name.into();
            }
            Type::Primitive(_) | Type::Array(_) | Type::Map { .. } => (),
        };

        self
    }

    pub fn referenced_types(&self) -> Vec<Type> {
        let mut referenced_types = Vec::new();

        match self {
            Type::Primitive(_) | Type::Union { .. } => (),
            Type::Array(ty) => {
                referenced_types.push(*ty.clone());
                referenced_types.extend(ty.referenced_types());
            }
            Type::Map { key, value } => {
                referenced_types.push(*key.clone());
                referenced_types.push(*value.clone());
                referenced_types.extend(key.referenced_types());
                referenced_types.extend(value.referenced_types());
            }
            Type::Record { members, .. } => {
                for member in members {
                    referenced_types.push(member.ty.clone());
                    referenced_types.extend(member.ty.referenced_types());
                }
            }
        };

        referenced_types
    }
}

#[derive(Debug, Clone)]
pub struct TypeDeclarations {
    declarations: IndexMap<String, Type>,
}

impl TypeDeclarations {
    pub fn new() -> Self {
        Self {
            declarations: IndexMap::new(),
        }
    }

    /// Registers a type declaration.
    pub fn register(&mut self, ty: Type) {
        if let Some(name) = ty.name() {
            self.declarations.insert(name.to_owned(), ty.clone());

            self.register_referenced_types(ty);
        }
    }

    fn register_referenced_types(&mut self, ty: Type) {
        for ty in ty.referenced_types() {
            self.register(ty);
        }
    }
}

impl IntoIterator for TypeDeclarations {
    type Item = (String, Type);
    type IntoIter = IntoIter<String, Type>;

    fn into_iter(self) -> Self::IntoIter {
        self.declarations.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct Member {
    pub name: String,
    pub description: Option<String>,
    pub ty: Type,
    pub is_optional: bool,
}

#[derive(Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub name: String,
    pub description: Option<NonEmptyString>,
    pub method: HttpMethod,
    pub url: String,
    pub group: String,
    pub version: String,
    pub url_parameters: Vec<UrlParameter>,
    pub payload_type: Option<Type>,
    pub return_type: Option<Type>,
}

#[derive(Debug)]
pub enum UrlSegment {
    Literal(String),
    Parameter(String),
}

impl Route {
    pub fn url_segments(&self) -> Vec<UrlSegment> {
        self.url
            .split('/')
            .map(|segment| {
                if let Some(parameter) = segment.strip_prefix(':') {
                    UrlSegment::Parameter(parameter.into())
                } else {
                    UrlSegment::Literal(segment.into())
                }
            })
            .collect()
    }

    pub fn all_parameters(&self) -> Vec<(String, Type)> {
        let mut all_parameters: Vec<(String, Type)> = self
            .url_parameters
            .iter()
            .map(|parameter| {
                (
                    parameter.name.clone(),
                    Type::Primitive(parameter.ty.clone()),
                )
            })
            .collect();

        if let Some((name, ty)) = self.payload_type.as_ref().map(|ty| ("payload".into(), ty)) {
            all_parameters.push((name, ty.to_owned()));
        }

        all_parameters
    }
}

#[derive(Debug, Clone)]
pub struct UrlParameter {
    pub name: String,
    pub ty: Primitive,
}

#[derive(Debug, Clone)]
pub struct SdkVersion {
    pub version: String,
    pub resources: Vec<SdkResource>,
}

#[derive(Debug, Clone)]
pub struct SdkResource {
    pub resource: String,
    pub routes: Vec<Route>,
}
