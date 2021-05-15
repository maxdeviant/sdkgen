use std::convert::TryFrom;

use openapiv3::{
    ObjectType, OpenAPI as OpenApi, Operation, Parameter, ParameterData, PathItem, ReferenceOr,
    Response, Schema, SchemaKind, StatusCode, Type as OpenApiType,
};
use sdkgen_core::{HttpMethod, Primitive, Route, Type, UrlParameter};
use serde_yaml;

pub fn from_yaml(openapi_yaml: &str) -> serde_yaml::Result<Vec<Route>> {
    let openapi: OpenApi = serde_yaml::from_str(openapi_yaml)?;

    Ok(from_openapi(openapi))
}

fn from_openapi(openapi: OpenApi) -> Vec<Route> {
    let mut routes = Vec::new();

    for (path, reference_or_path_item) in openapi.paths.iter() {
        match reference_or_path_item {
            ReferenceOr::Reference { .. } => {
                println!("Unhandled reference at path: '{}'", &path);
                continue;
            }
            ReferenceOr::Item(path_item) => {
                let routes_for_path = path_to_routes(&openapi, path.clone(), path_item.clone());

                routes.extend(routes_for_path);
            }
        }
    }

    routes
}

fn path_to_routes(openapi: &OpenApi, path: String, path_item: PathItem) -> Vec<Route> {
    let get_route = path_item
        .get
        .map(|operation| operation_to_route(&openapi, path.clone(), HttpMethod::Get, operation));

    let post_route = path_item
        .post
        .map(|operation| operation_to_route(&openapi, path.clone(), HttpMethod::Post, operation));

    let put_route = path_item
        .put
        .map(|operation| operation_to_route(&openapi, path.clone(), HttpMethod::Put, operation));

    let patch_route = path_item
        .patch
        .map(|operation| operation_to_route(&openapi, path.clone(), HttpMethod::Patch, operation));

    let delete_route = path_item
        .delete
        .map(|operation| operation_to_route(&openapi, path.clone(), HttpMethod::Delete, operation));

    vec![get_route, post_route, put_route, patch_route, delete_route]
        .into_iter()
        .filter_map(|x| x)
        .collect()
}

fn parameter_to_url_parameter(parameter: ParameterData) -> UrlParameter {
    UrlParameter {
        name: parameter.name,
        ty: Primitive::String,
    }
}

fn operation_to_route(
    openapi: &OpenApi,
    path: String,
    method: HttpMethod,
    operation: Operation,
) -> Route {
    let url_parameters: Vec<UrlParameter> = operation
        .parameters
        .iter()
        .filter_map(|parameter| match parameter {
            ReferenceOr::Reference { .. } => None,
            ReferenceOr::Item(Parameter::Path { parameter_data, .. }) => {
                Some(parameter_to_url_parameter(parameter_data.clone()))
            }
            ReferenceOr::Item(_) => None,
        })
        .collect();

    let default_or_ok_response = operation.responses.default.clone().or_else(|| {
        operation
            .responses
            .responses
            .get(&StatusCode::Code(200))
            .cloned()
    });

    Route {
        name: operation
            .operation_id
            .expect(&format!("No operation ID for {}", &path)),
        url: path.clone().replace("{", ":").replace("}", ""),
        method,
        group: path,
        version: "".into(),
        url_parameters,
        payload_type: None,
        return_type: default_or_ok_response.and_then(|default_response| match default_response {
            ReferenceOr::Item(default_response) => {
                response_to_return_type(&openapi, default_response).ok()
            }
            ReferenceOr::Reference { .. } => None,
        }),
    }
}

fn response_to_return_type(openapi: &OpenApi, response: Response) -> Result<Type, String> {
    let media_type = "application/json";

    let media_type = response
        .content
        .get(media_type)
        .ok_or_else(|| format!("No response found for {}", media_type))?;

    let schema = media_type
        .schema
        .clone()
        .and_then(|schema| match schema {
            ReferenceOr::Item(schema) => Some(NamedOrAnonymous::Anonymous(schema)),
            ReferenceOr::Reference { reference } => SchemaReference::try_from(reference)
                .ok()
                .and_then(|schema| schema.resolve(&openapi))
                .map(|named_schema| {
                    NamedOrAnonymous::Named(named_schema.name, named_schema.schema)
                }),
        })
        .ok_or_else(|| format!("No schema."))?;

    let return_type = schema_to_type(schema.into_value());

    Ok(return_type)
}

fn schema_to_type(schema: Schema) -> Type {
    match schema.schema_kind {
        SchemaKind::Type(ty) => openapi_type_to_type(ty),
        _ => unimplemented!(),
    }
}

fn openapi_type_to_type(ty: OpenApiType) -> Type {
    match ty {
        OpenApiType::String(_) => Type::Primitive(Primitive::String),
        OpenApiType::Number(_) => Type::Primitive(Primitive::Float),
        OpenApiType::Integer(_) => Type::Primitive(Primitive::Integer),
        OpenApiType::Boolean {} => Type::Primitive(Primitive::Boolean),
        OpenApiType::Object(ObjectType { properties, .. }) => Type::Map {
            key: Box::new(Type::Primitive(Primitive::String)),
            value: Box::new(Type::Primitive(Primitive::String)),
        },
        OpenApiType::Array(_) => Type::Array(Box::new(Type::Primitive(Primitive::String))),
    }
}

#[derive(Debug)]
enum NamedOrAnonymous<T> {
    Named(String, T),
    Anonymous(T),
}

impl<T> NamedOrAnonymous<T> {
    fn into_value(self) -> T {
        match self {
            NamedOrAnonymous::Named(_, value) | NamedOrAnonymous::Anonymous(value) => value,
        }
    }
}

#[derive(Debug)]
struct NamedSchema {
    name: String,
    schema: Schema,
}

#[derive(Debug)]
struct SchemaReference(String);

impl SchemaReference {
    fn resolve(&self, openapi: &OpenApi) -> Option<NamedSchema> {
        let components = openapi.components.as_ref()?;
        let schema_or_reference = components.schemas.get(&self.0)?;

        match schema_or_reference {
            ReferenceOr::Item(schema) => Some(NamedSchema {
                name: self.0.clone(),
                schema: schema.clone(),
            }),
            ReferenceOr::Reference { reference } => SchemaReference::try_from(reference.clone())
                .ok()
                .and_then(|schema| schema.resolve(openapi)),
        }
    }
}

impl TryFrom<String> for SchemaReference {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let schema_path = "#/components/schemas/";

        if !value.starts_with(schema_path) {
            return Err(format!("Not a schema reference: '{}'.", value));
        }

        let path = value.trim_start_matches(schema_path);

        Ok(Self(path.into()))
    }
}
