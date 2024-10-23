//! # Router Module
//!
//! This module provides the [Router] struct for managing asynchronous
//! request-response communication using unique identifiers (UUIDs). It
//! leverages the [async_channel] crate for message passing and the [tokio]
//! crate for asynchronous operations.
//!
//! ## Overview
//!
//! The [Router] struct is designed to facilitate the routing of requests and
//! responses between different components of an application. It uses channels
//! to send and receive requests and responses, ensuring that the communication
//! is non-blocking and efficient. Each request is associated with a unique UUID
//! to match it with the corresponding response.
//!
//! Also provided is a default implementation for easy instantiation with
//! pre-configured channel capacities.
use std::{future::Future, sync::Arc, time::Duration};

use async_channel::{bounded, unbounded, Receiver, Sender};
use scc::HashMap;
use uuid::Uuid;

use crate::endpoint::Endpoint;

#[derive(Debug, Clone)]
/// The `Router` struct is responsible for routing requests and responses
/// between different components. It uses channels for communication and
/// maintains a mapping of request IDs to response senders.
///
/// # Type Parameters
/// - `Request`: any type that implements [Send] + [Clone] + 'static
/// - `Response`: any type that implements [Send] + [Clone] + 'static
pub struct Router<Request, Response> {
    /// used by Endpoints to send incoming requests to the router for processing
    registration_sender: Sender<(Request, Sender<Response>)>,
    /// used by the router's registration loop to receiving new requests and
    /// their corresponding response senders
    registration_receiver: Receiver<(Request, Sender<Response>)>,
    /// used by the registration loop to sending requests along with their unique
    /// identifiers to workers
    request_sender: Sender<(Uuid, Request)>,
    /// used by the router's response loop to receive responses along with their
    /// unique identifiers
    request_receiver: Receiver<(Uuid, Request)>,
    /// used by workers to sending responses along with their unique identifiers
    response_sender: Sender<(Uuid, Response)>,
    /// used by the router's response loop to receive responses along with their
    /// unique identifiers
    response_receiver: Receiver<(Uuid, Response)>,
    /// maps unique request IDs to their corresponding response senders
    response_map: Arc<HashMap<Uuid, Sender<Response>>>,
}

/// Asynchronous private function that continuously listens for incoming
/// responses and routes them to the appropriate sender based on the UUID.
///
/// # Arguments
///
/// - `response_receiver`: A receiver channel that receives tuples of UUIDs and
///     responses.
/// - `response_map`: Router's `HashMap` that maps UUIDs to their corresponding
///     response senders.
///
/// # Type Parameters
///
/// - `Response`: The type of the response. It must implement [Send], [Clone],
///     and `'static`,
///
/// # Behavior
///
/// The function runs in an infinite loop, awaiting responses from the
/// `response_receiver`. When a response is received, it attempts to find the
/// corresponding sender in the `response_map` using the UUID. If a sender is
/// found, it sends the response to the sender. If sending the response fails,
/// it logs the error.
async fn response_loop<Response>(
    response_receiver: Receiver<(Uuid, Response)>,
    response_map: Arc<HashMap<Uuid, Sender<Response>>>,
) where
    Response: Send + 'static + Clone,
{
    while let Ok((uuid, response)) = response_receiver.recv().await {
        match response_map.remove_async(&uuid).await {
            Some((_, sender)) => match sender.send(response).await {
                //TODO: Handle error via logging and tracing
                Ok(_) => {
                    println!("Success from resp loop")
                }
                Err(err) => {
                    println!("Error from resp loop : {:?}", err)
                }
            },
            None => {
                println!(
                    "Error from resp loop : No sender found for uuid: {:?}",
                    uuid
                );
            }
        }
    }
}

/// Asynchronous private function that continuously listens for incoming
/// registration requests and maps them to unique UUIDs.
///
/// # Arguments
///
/// - `registration_receiver`: A receiver channel that receives tuples of
///     requests and their corresponding response senders.
/// - `response_map`: router's `HashMap` that maps UUIDs to their corresponding
///     response senders.
/// - `request_sender`: A sender channel that sends tuples of UUIDs and
///     requests.
///
/// # Type Parameters
///
/// - `Request`: The type of the request. It must implement [Send], [Clone],
///     and `'static`,
/// - `Response`: The type of the response. It must implement [Send], [Clone],
///     and `'static`,
///
/// # Behavior
///
/// The function runs in an infinite loop, awaiting registration requests from
/// the `registration_receiver`. When a request is received, it generates a new
/// UUID, maps the UUID to the response sender in the `response_map`, and sends
/// the UUID and request to the `request_sender`. If inserting into the
/// `response_map` fails (e.g., if the key already exists), it handles the error
/// appropriately.
async fn registration_loop<Request, Response>(
    registration_receiver: Receiver<(Request, Sender<Response>)>,
    response_map: Arc<HashMap<Uuid, Sender<Response>>>,
    request_sender: Sender<(Uuid, Request)>,
) where
    Request: Send + 'static + Clone,
    Response: Send + 'static + Clone,
{
    while let Ok((request, response_sink)) = registration_receiver.recv().await {
        // insert can fail if key already exists, unlikly but handled.
        let mut uuid = Uuid::new_v4();
        while response_map
            .insert_async(uuid, response_sink.clone())
            .await
            //.await
            .is_err()
        {
            uuid = Uuid::new_v4();
        }
        let request_sender = request_sender.clone();
        tokio::spawn(async move {
            //TODO: Handle error via logging and tracing
            match request_sender.send((uuid, request)).await {
                Ok(_) => {
                    println!("Success from reg loop")
                }
                Err(err) => {
                    println!("Error from reg loop : {:?}", err)
                }
            };
        });
    }
}

impl<Request, Response> Default for Router<Request, Response>
where
    Request: Send + 'static + Clone,
    Response: Send + 'static + Clone,
{
    fn default() -> Self {
        Self::bounded(Some(100), Some(100), Some(100))
    }
}

impl<Request, Response> Router<Request, Response>
where
    Request: Send + 'static + Clone,
    Response: Send + 'static + Clone,
{
    /// Creates a new instance of the `Router` struct with bounded or unbounded
    /// channels based on the provided sizes.
    ///
    /// # Arguments
    ///
    /// - `registration_channel_size`: An optional size for the registration
    ///     channel. If `None`, an unbounded channel is created.
    /// - `request_channel_size`: An optional size for the request channel. If
    ///     `None`, an unbounded channel is created.
    /// - `response_channel_size`: An optional size for the response channel. If
    ///     `None`, an unbounded channel is created.
    ///
    /// # Returns
    ///
    /// Returns a new instance of the `Router` struct with the specified channel
    /// sizes and an empty response map.
    pub fn bounded(
        registration_channel_size: Option<usize>,
        request_channel_size: Option<usize>,
        response_channel_size: Option<usize>,
    ) -> Self {
        let (registration_sender, registration_receiver) = match registration_channel_size {
            Some(b) => bounded(b),
            None => unbounded(),
        };
        let (request_sender, request_receiver) = match request_channel_size {
            Some(b) => bounded(b),
            None => unbounded(),
        };
        let (response_sender, response_receiver) = match response_channel_size {
            Some(b) => bounded(b),
            None => unbounded(),
        };
        let response_map = Arc::new(HashMap::new());
        Self {
            registration_sender,
            registration_receiver,
            request_sender,
            request_receiver,
            response_sender,
            response_receiver,
            response_map,
        }
    }
    /// Creates a new [Endpoint] instance using the router's registration sender
    /// and an optional timeout.
    ///
    /// # Arguments
    ///
    /// - `timeout`: An optional [Duration] specifying the timeout for the
    ///   [Endpoint]. If `None`, no timeout is applied.
    ///
    /// # Returns
    ///
    /// Returns a new instance of the [Endpoint] struct configured with the
    /// router's registration sender and the specified timeout.
    pub fn endpoint(&self, timeout: Option<Duration>) -> Endpoint<Request, Response> {
        Endpoint::new(self.registration_sender.clone(), timeout)
    }
    pub fn tokio_spawn(&self) -> tokio::task::JoinHandle<()> {
        let temp = self.clone();
        tokio::spawn(async move { temp.run().await })
    }

    pub fn tokio_spawn_workers<F>(
        &self,
        num_workers: usize,
        worker_fn: impl Fn(Receiver<(Uuid, Request)>, Sender<(Uuid, Response)>) -> F,
    ) -> Vec<tokio::task::JoinHandle<()>>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let mut handles = Vec::new();
        for _ in 0..num_workers {
            handles.push(tokio::spawn(worker_fn(
                self.request_receiver.clone(),
                self.response_sender.clone(),
            )));
        }
        handles
    }
    pub async fn run(&self) {
        let response_loop = tokio::spawn(response_loop(
            self.response_receiver.clone(),
            self.response_map.clone(),
        ));
        let registration_loop = tokio::spawn(registration_loop(
            self.registration_receiver.clone(),
            self.response_map.clone(),
            self.request_sender.clone(),
        ));
        let _ = tokio::join!(response_loop, registration_loop);
    }
}
