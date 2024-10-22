//! # Sync2Async4Coms Library
//!
//! This library provides utilities for managing asynchronous communication in Rust applications. It includes
//! modules for handling endpoints and routing requests and responses using unique identifiers (UUIDs).
//!
//! ## Modules
//!
//! - `endpoint`: Provides the `Endpoint` struct and `EndpointError` enum for handling asynchronous communication
//!   with a timeout mechanism.
//! - `router`: Provides the `Router` struct for managing asynchronous request-response communication using UUIDs.
//!
//! ## Overview
//!
//! The `sync2async4coms` library is designed to facilitate asynchronous communication in Rust applications. It
//! leverages the `async_channel` crate for message passing and the `tokio` crate for handling asynchronous
//! operations and timeouts. The library provides a robust and efficient way to manage request-response patterns
//! with unique identifiers and optional timeouts.
//!
//! ## Usage
//!
//! To use the `sync2async4coms` library, you need to include it as a dependency in your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! sync2async4coms = "0.1.0"
//! ```
//!
//! You can then import the necessary modules and structs in your code:
//!
//! ```rust
//! use sync2async4coms::endpoint::{Endpoint, EndpointError};
//! use sync2async4coms::router::Router;
//! ```
//!
//! ## Example
//!
//! Here is a simple example demonstrating how to use the `Endpoint` and `Router` structs:
//!
//! ```rust
//! use async_channel::{bounded, Sender};
//! use tokio::time::Duration;
//! use sync2async4coms::endpoint::{Endpoint, EndpointError};
//! use sync2async4coms::router::Router;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), EndpointError> {
//!     // Create an endpoint with a timeout
//!     let (sender, receiver) = bounded(100);
//!     let endpoint = Endpoint::<String, String>::new(sender, Some(Duration::from_secs(5)));
//!
//!     // Create a router
//!     let router: Router<String, String> = Router::default();
//!
//!     // Example usage of the endpoint and router
//!     // ...
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - Asynchronous request-response handling
//! - Unique identifier (UUID) based routing
//! - Optional timeout for requests
//! - Comprehensive error handling
//!
//! ## Dependencies
//!
//! - `async_channel` for asynchronous message passing
//! - `tokio` for handling asynchronous operations and timeouts
//! - `uuid` for generating unique identifiers
//! - `thiserror` for error handling

pub mod endpoint;
pub mod router;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::router::Router;
    use async_channel::{Receiver, Sender};
    use tokio::time::Duration;
    use uuid::Uuid;

    use actix_web::get;
    use actix_web::web::Data;
    use actix_web::{web, App, HttpResponse, HttpServer, Responder};

    async fn worker_200ms(receiver: Receiver<(Uuid, String)>, sender: Sender<(Uuid, String)>) {
        while let Ok((uuid, request)) = receiver.recv().await {
            tokio::time::sleep(Duration::from_millis(200)).await;
            let response = format!("Response to request: {}", request);
            sender.send((uuid, response)).await.unwrap();
        }
    }
    #[get("/{timeout}")]
    async fn hello(
        router: Data<Router<String, String>>,
        request: web::Path<u64>,
    ) -> impl Responder {
        let timeout = Duration::from_millis(request.into_inner());
        let response = router
            .endpoint(Some(timeout))
            .handle_request("Hello World".to_string())
            .await;
        match response {
            Ok(response) => HttpResponse::Ok().body(response),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    }

    #[tokio::test]
    async fn test_endpoint_and_router_with_timeout() {
        // Create a Router
        let router: Router<String, String> = Router::default();

        // Spawn a worker to handle requests
        router.tokio_spawn_workers(4, worker_200ms);
        router.tokio_spawn();

        let data = Data::from(Arc::new(router));
        // spawn server
        tokio::spawn(async move {
            let _ = HttpServer::new(move || App::new().app_data(Data::clone(&data)).service(hello))
                .bind(("0.0.0.0", 8080))
                .unwrap()
                .run()
                .await;
        });
        tokio::time::sleep(Duration::from_millis(300)).await;
        // execute 200ms worker response with 250 ms timeout
        let response = reqwest::get("http://127.0.0.1:8080/250").await.unwrap();
        assert_eq!(response.status().as_u16(), 200);
        assert_eq!(
            response.text().await.unwrap(),
            "Response to request: Hello World"
        );
        // execute 200ms worker response with 150 ms timeout
        let response = reqwest::get("http://127.0.0.1:8080/150").await.unwrap();
        assert_eq!(response.status().as_u16(), 500);
    }
}
