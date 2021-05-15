use std::collections::HashMap;

use emitter_csharp::CsharpSdk;
use emitter_typescript::TypeScriptSdk;
use sdkgen_core::{GenerateSdk, Route, SdkResource, SdkVersion, TypeDeclarations};

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

    let mut type_decls = TypeDeclarations::new();

    for route in routes.iter() {
        if let Some(payload_type) = route.payload_type.as_ref() {
            type_decls.register(payload_type.to_owned());
        }

        if let Some(return_type) = route.return_type.as_ref() {
            type_decls.register(return_type.to_owned());
        }
    }

    let versions = versions_from_routes(routes);

    let csharp_output = CsharpSdk.generate_sdk(type_decls.clone(), versions.clone());
    let typescript_output = TypeScriptSdk.generate_sdk(type_decls, versions);

    use std::fs::File;
    use std::io::prelude::*;

    let mut csharp_file = File::create("generated/csharp.cs")?;
    csharp_file.write_all(&csharp_output.as_bytes())?;

    let mut typescript_file = File::create("generated/typescript.ts")?;
    typescript_file.write_all(&typescript_output.as_bytes())?;

    Ok(())
}
