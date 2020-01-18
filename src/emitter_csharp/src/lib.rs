use std::path::PathBuf;

use sdkgen_core::{GenerateSdk, SdkResource, SdkVersion, Type};

pub struct CsharpSdk;

impl GenerateSdk for CsharpSdk {
    fn generate_sdk<O: Into<PathBuf>>(&self, versions: Vec<SdkVersion>, output_directory: O) {
        for version in versions {
            for resource in version.resources {
                emit_sdk_resource(version.version.clone(), resource);
            }
        }
    }
}

fn emit_sdk_resource(version: String, resource: SdkResource) {
    let type_decls: Vec<Type> = resource
        .routes
        .into_iter()
        .flat_map(|route| vec![route.payload_type.map(|ty| vec![ty])])
        .filter_map(|x| x)
        .flat_map(|x| x)
        .collect();

    println!("Type decls: {:?}", type_decls);
}
