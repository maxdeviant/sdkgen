use heck::CamelCase;
use sdkgen_core::{GenerateSdk, Primitive, Route, SdkResource, SdkVersion, Type};

pub struct CsharpSdk;

impl GenerateSdk for CsharpSdk {
    fn generate_sdk(&self, versions: Vec<SdkVersion>) -> String {
        let mut buffer = String::new();

        for version in versions {
            for resource in version.resources {
                buffer += &emit_sdk_resource(version.version.clone(), resource);
            }
        }

        buffer
    }
}

fn emit_primitive(primitive: Primitive) -> &'static str {
    match primitive {
        Primitive::String => "string",
        Primitive::Boolean => "bool",
        Primitive::Integer => "int",
        Primitive::Float => "float",
        Primitive::Double => "double",
    }
}

fn emit_type_name(ty: Type) -> String {
    match ty {
        Type::Primitive(primitive) => emit_primitive(primitive).into(),
        Type::Array(ty) => format!("List<{}>", emit_type_name(*ty)),
        Type::Map { key, value } => format!(
            "Dictionary<{}, {}>",
            emit_type_name(*key),
            emit_type_name(*value)
        ),
        Type::Union { name, .. } | Type::Record { name, .. } => name.to_camel_case(),
    }
}

fn emit_sdk_resource(version: String, resource: SdkResource) -> String {
    let type_decls: Vec<Type> = resource
        .routes
        .iter()
        .flat_map(|route| vec![route.clone().payload_type.map(|ty| vec![ty])])
        .filter_map(|x| x)
        .flat_map(|x| x)
        .collect();

    let contents = resource
        .routes
        .into_iter()
        .map(emit_route)
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        r#"
namespace Sdk.V{version}
{{
    public static class Sdk
    {{
        {class_body}
    }}
}}
        "#,
        version = version,
        class_body = contents
    )
}

fn emit_route(route: Route) -> String {
    let return_type = route
        .return_type
        .unwrap_or(Type::Primitive(Primitive::String));

    format!(
        r#"
public static async Task<{return_type}> {function_name}()
{{

}}
    "#,
        return_type = emit_type_name(return_type),
        function_name = route.name
    )
}
