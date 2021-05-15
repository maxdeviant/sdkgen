use std::convert::identity;

use heck::{CamelCase, MixedCase};
use sdkgen_core::{
    GenerateSdk, HttpMethod, Primitive, Route, SdkResource, SdkVersion, Type, UrlSegment,
};

pub struct TypeScriptSdk;

impl GenerateSdk for TypeScriptSdk {
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
        Primitive::Boolean => "boolean",
        Primitive::Integer | Primitive::Float | Primitive::Double => "number",
    }
}

fn emit_type_name(ty: Type) -> String {
    match ty {
        Type::Primitive(primitive) => emit_primitive(primitive).into(),
        Type::Array(ty) => format!("{}[]", emit_type_name(*ty)),
        Type::Map { key, value } => format!(
            "Record<{}, {}>",
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
        .filter_map(identity)
        .flat_map(identity)
        .collect();

    let contents = resource
        .routes
        .into_iter()
        .map(emit_route)
        .collect::<Vec<String>>()
        .join("\n");

    contents
}

fn emit_route(route: Route) -> String {
    let return_type = route
        .return_type
        .as_ref()
        .unwrap_or(&Type::Primitive(Primitive::String));

    let parameter_list = route
        .all_parameters()
        .iter()
        .map(|(name, ty)| format!("{}: {}", name, emit_type_name(ty.clone())))
        .collect::<Vec<String>>()
        .join(", ");

    let url = route
        .url_segments()
        .into_iter()
        .map(|segment| match segment {
            UrlSegment::Parameter(param) => format!("${{{}}}", param),
            UrlSegment::Literal(value) => value,
        })
        .collect::<Vec<String>>()
        .join("/");

    let request_data = route.payload_type.map(|_| format!(r#"data: payload,"#));

    format!(
        r#"
export async function {function_name}({parameter_list}): Promise<{return_type}> {{
    const response = await axios({{
        method: '{http_method}',
        url: `{url}`,
        {request_data}
    }});

    return response.data;
}}
    "#,
        function_name = route.name.to_mixed_case(),
        parameter_list = parameter_list,
        http_method = match route.method {
            HttpMethod::Get => "get",
            HttpMethod::Post => "post",
            HttpMethod::Put => "put",
            HttpMethod::Patch => "patch",
            HttpMethod::Delete => "delete",
        },
        url = url,
        request_data = request_data.unwrap_or_default(),
        return_type = emit_type_name(return_type.to_owned())
    )
}
