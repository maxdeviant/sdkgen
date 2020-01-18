use apidoc::Route;

fn main() {
    let json = include_str!("../../../../../serve_apidoc/api_data.json");

    let routes: Vec<Route> = serde_json::from_str(json).expect("Failed to deserialize API data");

    println!("{:?}", routes);

    println!("Hello, world!");
}
