use openapiv3::{OpenAPI as OpenApi, Operation, PathItem, ReferenceOr};
use sdkgen_core::{HttpMethod, Route};
use serde_yaml;

pub fn from_yaml(openapi_yaml: &str) -> serde_yaml::Result<Vec<Route>> {
    let openapi: OpenApi = serde_yaml::from_str(openapi_yaml)?;

    Ok(from_openapi(openapi))
}

fn from_openapi(openapi: OpenApi) -> Vec<Route> {
    let mut routes = Vec::new();

    for (path, reference_or_path_item) in openapi.paths.into_iter() {
        match reference_or_path_item {
            ReferenceOr::Reference { .. } => {
                println!("Unhandled reference at path: '{}'", &path);
                continue;
            }
            ReferenceOr::Item(path_item) => {
                let routes_for_path = path_to_routes(path, path_item);

                routes.extend(routes_for_path);
            }
        }
    }

    routes
}

fn path_to_routes(path: String, path_item: PathItem) -> Vec<Route> {
    let get_route = path_item
        .get
        .map(|operation| operation_to_route(path.clone(), HttpMethod::Get, operation));

    let post_route = path_item
        .post
        .map(|operation| operation_to_route(path.clone(), HttpMethod::Post, operation));

    let put_route = path_item
        .put
        .map(|operation| operation_to_route(path.clone(), HttpMethod::Put, operation));

    vec![get_route, post_route, put_route]
        .into_iter()
        .filter_map(|x| x)
        .collect()
}

fn operation_to_route(path: String, method: HttpMethod, operation: Operation) -> Route {
    Route {
        name: operation
            .operation_id
            .expect(&format!("No operation ID for {}", &path)),
        url: path.clone(),
        method,
        group: path,
        version: "".into(),
        url_parameters: vec![],
        payload_type: None,
        return_type: None,
    }
}
