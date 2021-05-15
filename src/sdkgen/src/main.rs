use std::path::PathBuf;

use indexmap::IndexMap;
use sdkgen_core::{GenerateSdk, Route, SdkResource, SdkVersion, TypeDeclarations};
use sdkgen_emitter_csharp::CsharpSdk;
use sdkgen_emitter_typescript::TypeScriptSdk;
use structopt::StructOpt;

fn versions_from_routes(routes: Vec<Route>) -> Vec<SdkVersion> {
    let mut versions = IndexMap::new();

    for route in routes.iter() {
        let version = versions.entry(&route.version).or_insert_with(IndexMap::new);

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

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(name = "API_DEFINITION")]
    api_definition: PathBuf,
}

fn main() -> std::io::Result<()> {
    let args = Args::from_args();

    use std::ffi::OsStr;
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    let api_definition = File::open(&args.api_definition)?;
    let mut buf_reader = BufReader::new(api_definition);
    let mut api_definition = String::new();
    buf_reader.read_to_string(&mut api_definition)?;

    let routes = match args.api_definition.extension().and_then(OsStr::to_str) {
        Some("json") => sdkgen_adapter_openapi::from_json(&api_definition)
            .expect("Failed to deserialize API data"),
        Some("yaml") => sdkgen_adapter_openapi::from_yaml(&api_definition)
            .expect("Failed to deserialize API data"),
        Some(extension) => panic!("Invalid file extension: '{}'", extension),
        None => panic!("Could not determine extension"),
    };

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

    let mut csharp_file = File::create("generated/csharp.cs")?;
    csharp_file.write_all(&csharp_output.as_bytes())?;

    let mut typescript_file = File::create("generated/typescript.ts")?;
    typescript_file.write_all(&typescript_output.as_bytes())?;

    Ok(())
}
