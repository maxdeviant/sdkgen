use std::collections::HashMap;

use apidoc;
use emitter_csharp::CsharpSdk;
use emitter_typescript::TypeScriptSdk;
use sdkgen_adapter_openapi;
use sdkgen_core::{
    GenerateSdk, HttpMethod, Primitive, Route, SdkResource, SdkVersion, Type, UrlParameter,
};

fn convert_http_method(method: apidoc::HttpMethod) -> HttpMethod {
    match method {
        apidoc::HttpMethod::Get => HttpMethod::Get,
        apidoc::HttpMethod::Post => HttpMethod::Post,
        apidoc::HttpMethod::Put => HttpMethod::Put,
        apidoc::HttpMethod::Delete => HttpMethod::Delete,
    }
}

fn convert_primitive(ty: String) -> Result<Primitive, String> {
    match ty.as_str() {
        "String" | "GUID" => Ok(Primitive::String),
        "Boolean" => Ok(Primitive::Boolean),
        "Integer" => Ok(Primitive::Integer),
        "Float" => Ok(Primitive::Float),
        "Number" | "Double" => Ok(Primitive::Double),
        unknown => Err(format!("Unknown primitive: {}", unknown)),
    }
}

fn convert_route(route: apidoc::Route) -> Route {
    let url_parameters = route
        .parameter
        .map(|section| section.fields)
        .and_then(|fields| fields.get("Parameter").cloned())
        .map(|parameters| {
            parameters
                .into_iter()
                .map(|parameter| UrlParameter {
                    name: parameter.field,
                    ty: convert_primitive(parameter.ty)
                        .expect("Failed to convert to primitive type"),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Route {
        name: route.name,
        method: convert_http_method(route.method),
        url: route.url,
        group: route.group,
        version: route.version,
        url_parameters,
        payload_type: None,
        return_type: None,
    }
}

fn versions_from_routes(routes: Vec<Route>) -> Vec<SdkVersion> {
    let mut versions = HashMap::new();

    for route in routes.iter() {
        let version = versions.entry(&route.version).or_insert_with(HashMap::new);

        let resource_routes = version
            .entry(&route.group)
            .or_insert_with(Vec::<Route>::new);

        resource_routes.push(route.clone());
    }

    versions
        .into_iter()
        .map(|(version, resources)| SdkVersion {
            version: version.clone(),
            resources: resources
                .into_iter()
                .map(|(resource, routes)| SdkResource {
                    resource: resource.clone(),
                    routes,
                })
                .collect(),
        })
        .collect()
}

fn main() -> std::io::Result<()> {
    let petstore_yaml = include_str!("../../../fixtures/petstore.yaml");

    let routes =
        sdkgen_adapter_openapi::from_yaml(petstore_yaml).expect("Failed to deserialize API data");

    let versions = versions_from_routes(routes);

    let csharp_output = CsharpSdk.generate_sdk(versions.clone());
    let typescript_output = TypeScriptSdk.generate_sdk(versions);

    use std::fs::File;
    use std::io::prelude::*;

    let mut csharp_file = File::create("generated/csharp.cs")?;
    csharp_file.write_all(&csharp_output.as_bytes())?;

    let mut typescript_file = File::create("generated/typescript.ts")?;
    typescript_file.write_all(&typescript_output.as_bytes())?;

    Ok(())
}
