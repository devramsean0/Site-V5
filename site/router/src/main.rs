use std::net::TcpStream;

fn root(mut _stream: &TcpStream) -> String {
    "HTTP/1.1 200 OK\r\n\r\n".to_string()
}

fn main() {
    env_logger::init();
    router::Router::new("127.0.0.1".to_string(), 3000)
        .register_route(router::Route {
            method: "GET".to_string(),
            path: "/".to_string(),
            route_callbacks: router::RouteCallbacks {
                microservice_path: None,
                run_function: Some(root),
            },
        })
        .register_route(router::Route {
            method: "GET".to_string(),
            path: "/favicon.ico".to_string(),
            route_callbacks: router::RouteCallbacks {
                microservice_path: None,
                run_function: Some(root),
            },
        })
        .start();
}
