use std::convert::identity;

use heck::CamelCase;
use sdkgen_core::{
    GenerateSdk, HttpMethod, Primitive, Route, SdkResource, SdkVersion, Type, UrlSegment,
};

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
        .filter_map(identity)
        .flat_map(identity)
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
        .as_ref()
        .unwrap_or(&Type::Primitive(Primitive::String));

    let parameter_list = route
        .all_parameters()
        .iter()
        .map(|(name, ty)| format!("{} {}", emit_type_name(ty.clone()), name))
        .collect::<Vec<String>>()
        .join(", ");

    let url = route
        .url_segments()
        .into_iter()
        .map(|segment| match segment {
            UrlSegment::Parameter(param) => format!("{{{}}}", param),
            UrlSegment::Literal(value) => value,
        })
        .collect::<Vec<String>>()
        .join("/");

    let request_content = route.payload_type.map(|_| format!(r#"request.Content = new StringContent(JsonConvert.SerializeObject(payload), Encoding.UTF8, "application/json");"#));

    format!(
        r#"
/// <summary>
/// {summary}
/// </summary>
public static async Task<{return_type}> {function_name}({parameter_list})
{{
    var request = new HttpRequestMessage
    {{
        Method = HttpMethod.{http_method},
        RequestUri = new Uri(apiUrl, $"{url}")
    }};
    request.Headers.Authorization = new AuthenticationHeaderValue("Bearer", accessToken);
    {request_content}

    var response = await httpClient.SendAsync(request).ConfigureAwait(false);
    var responseBody = await response.Content.ReadAsStringAsync().ConfigureAwait(false);

    return JsonConvert.DeserializeObject<{return_type}>(responseBody);
}}
    "#,
        function_name = route.name.to_camel_case(),
        parameter_list = parameter_list,
        http_method = match route.method {
            HttpMethod::Get => "Get",
            HttpMethod::Post => "Post",
            HttpMethod::Put => "Put",
            HttpMethod::Delete => "Delete",
        },
        url = url,
        request_content = request_content.unwrap_or_default(),
        return_type = emit_type_name(return_type.to_owned()),
        summary = ""
    )
}
