mod schema;

use openapiv3::{
    ArrayType, ObjectType, OpenAPI as OpenApi, Operation, Parameter, ParameterData, PathItem,
    ReferenceOr, Response, Schema, SchemaKind, StatusCode, Type as OpenApiType,
};
use sdkgen_core::{HttpMethod, Member, Primitive, Route, Type, UrlParameter};
use serde_yaml;

use crate::schema::resolve_schema;

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
        .and_then(|schema| resolve_schema(&openapi, schema))
        .ok_or_else(|| format!("No schema."))?;

    let return_type = schema_to_type(&openapi, schema.clone());

    Ok(return_type)
}

fn schema_to_type(openapi: &OpenApi, schema: NamedOrAnonymous<Schema>) -> Type {
    let name = schema.name().cloned();

    let mut ty = match schema.into_value().schema_kind {
        SchemaKind::Type(ty) => openapi_type_to_type(&openapi, ty),
        _ => unimplemented!(),
    };

    if let Some(name) = name {
        ty = ty.set_name(name);
    }

    ty
}

fn openapi_type_to_type(openapi: &OpenApi, ty: OpenApiType) -> Type {
    match ty {
        OpenApiType::String(_) => Type::Primitive(Primitive::String),
        OpenApiType::Number(_) => Type::Primitive(Primitive::Float),
        OpenApiType::Integer(_) => Type::Primitive(Primitive::Integer),
        OpenApiType::Boolean {} => Type::Primitive(Primitive::Boolean),
        OpenApiType::Object(ObjectType {
            properties,
            required,
            ..
        }) => Type::Record {
            name: "No Name".into(),
            members: properties
                .into_iter()
                .map(|(name, schema)| {
                    let is_optional = !required.contains(&name);

                    let schema = resolve_schema(&openapi, schema.unbox());

                    Member {
                        name,
                        description: None,
                        ty: schema
                            .clone()
                            .map(|schema| schema_to_type(&openapi, schema))
                            .unwrap_or(Type::Primitive(Primitive::String)),
                        is_optional,
                    }
                })
                .collect(),
        },
        OpenApiType::Array(ArrayType { items, .. }) => {
            let item_type = match resolve_schema(&openapi, items.unbox()) {
                Some(schema) => schema_to_type(&openapi, schema),
                None => Type::Primitive(Primitive::String),
            };

            Type::Array(Box::new(item_type))
        }
    }
}

#[derive(Debug, Clone)]
enum NamedOrAnonymous<T> {
    Named(String, T),
    Anonymous(T),
}

impl<T> NamedOrAnonymous<T> {
    fn name(&self) -> Option<&String> {
        match self {
            NamedOrAnonymous::Named(name, _) => Some(name),
            NamedOrAnonymous::Anonymous(_) => None,
        }
    }

    fn into_value(self) -> T {
        match self {
            NamedOrAnonymous::Named(_, value) | NamedOrAnonymous::Anonymous(value) => value,
        }
    }
}
