pub trait GenerateSdk {
    fn generate_sdk(&self, versions: Vec<SdkVersion>) -> String;
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
    Delete,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub group: String,
    pub version: String,
    pub url_parameters: Vec<UrlParameter>,
    pub payload_type: Option<Type>,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct UrlParameter {
    pub name: String,
    pub ty: Primitive,
}

#[derive(Debug)]
pub struct SdkVersion {
    pub version: String,
    pub resources: Vec<SdkResource>,
}

#[derive(Debug)]
pub struct SdkResource {
    pub resource: String,
    pub routes: Vec<Route>,
}
