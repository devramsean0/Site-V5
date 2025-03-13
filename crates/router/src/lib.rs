use log::{debug, info};
use std::{
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};

mod cache_db;

#[derive(PartialEq, Debug)]
pub struct RouteCallbacks {
    pub microservice_path: Option<String>,
    pub run_function: Option<fn(&TcpStream) -> String>,
}

#[derive(PartialEq, Debug)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub route_callbacks: RouteCallbacks,
}

pub struct Router {
    routes: Vec<Route>,
    port: i32,
    host: String,
    db: rusqlite::Connection,
}

impl Default for RouteCallbacks {
    fn default() -> RouteCallbacks {
        RouteCallbacks {
            microservice_path: Default::default(),
            run_function: Default::default(),
        }
    }
}

impl Router {
    pub fn new(host: String, port: i32) -> Self {
        info!("Establishing Cache DB");
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        cache_db::run_migrations(&conn);
        info!("Creating router struct");
        Router {
            routes: vec![],
            port,
            host,
            db: conn
        }
    }

    pub fn register_route(&mut self, route: Route) -> &mut Self {
        info!("Registering {} route {}", route.method, route.path);
        self.routes.push(route);
        self
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let buf_reader = BufReader::new(&stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|line| line.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();
        debug!("Raw HTTP request: {:?}", http_request);
        let split_path = http_request[0].split(" ").collect::<Vec<_>>();
        info!("Recieved Request to {} {}", split_path[0], split_path[1]);
        for route in &self.routes {
            if route.method == split_path[0] && route.path == split_path[1] {
                debug!("Route found for {} {}", route.method, route.path);
                let cache_lookup = cache_db::retrieve_web_cache(&self.db, route.method.clone(), route.path.clone());
                if cache_lookup.len() > 0 {
                    debug!("Cache hit");
                    stream.write(cache_lookup[0].body.as_bytes()).unwrap();
                    info!("Cache hit for {} {}", route.method, route.path);
                } else {
                    if route.route_callbacks.microservice_path.is_some() {
                        debug!("Microservice path found");
                        // Call microservice
                    } else if route.route_callbacks.run_function.is_some() {
                        debug!("Run function found");
                        let return_data = route.route_callbacks.run_function.unwrap()(&stream);
                        stream.write(return_data.as_bytes()).unwrap();
                        cache_db::insert_web_cache(&self.db, route.method.clone(), route.path.clone(), return_data);
                    }
                }
            }
        }
    }

    pub fn start(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.host.as_str(), self.port)).unwrap();
        debug!("Bound TCP Listener to {}:{}", self.host, self.port);
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            debug!("Connection established");
            self.handle_connection(stream);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new_router() {
        let router = Router::new("0.0.0.0".to_string(), 3000);
        assert_eq!(router.routes, vec![]);
        assert_eq!(router.host, "0.0.0.0".to_string());
        assert_eq!(router.port, 3000);
    }

    #[test]
    fn test_route_registered() {
        let mut router = Router::new("0.0.0.0".to_string(), 3000);
        router.register_route(Route {
            method: "GET".to_string(),
            path: "/".to_string(),
            route_callbacks: Default::default(),
        });
        assert_eq!(router.routes[0].method, "GET".to_string());
        assert_eq!(router.routes[0].path, "/".to_string());
    }
}
