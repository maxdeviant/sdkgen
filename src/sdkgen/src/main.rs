use apidoc;
use sdkgen_core::{HttpMethod, Route};

fn convert_http_method(method: apidoc::HttpMethod) -> HttpMethod {
    match method {
        apidoc::HttpMethod::Get => HttpMethod::Get,
        apidoc::HttpMethod::Post => HttpMethod::Post,
        apidoc::HttpMethod::Put => HttpMethod::Put,
        apidoc::HttpMethod::Delete => HttpMethod::Delete,
    }
}

fn convert_route(route: apidoc::Route) -> Route {
    Route {
        name: route.name,
        method: convert_http_method(route.method),
        url: route.url,
        group: route.group,
        version: route.version,
        url_parameters: vec![],
        payload_type: None,
        return_type: None,
    }
}

fn main() {
    let json = include_str!("../../../../../serve_apidoc/api_data.json");

    let routes: Vec<apidoc::Route> =
        serde_json::from_str(json).expect("Failed to deserialize API data");

    let routes: Vec<Route> = routes.into_iter().map(convert_route).collect();

    println!("{:?}", routes);

    println!("Hello, world!");
}
