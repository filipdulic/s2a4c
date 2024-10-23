use std::{sync::Arc, time::Duration};

use actix_web::{
    get,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use async_channel::{Receiver, Sender};
use sync2async4coms::router::Router;
use uuid::Uuid;

async fn sleep_ms(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

// Simple worker that waits for 200ms before responding
async fn worker(receiver: Receiver<(Uuid, String)>, sender: Sender<(Uuid, String)>) {
    while let Ok((uuid, request)) = receiver.recv().await {
        sleep_ms(200).await;
        sender
            .send((uuid, format!("Response to request: {}", request)))
            .await
            .unwrap();
    }
}
// actix web hello world service
// uses router endpoint to handle requests
// times out after the specified duration in miliseconds
#[get("/{timeout}-{name}")]
async fn hello(
    router: Data<Router<String, String>>,
    request: web::Path<(u64, String)>,
) -> impl Responder {
    let (timeout, name) = request.into_inner();
    let timeout = Duration::from_millis(timeout);
    let response = router
        .endpoint(Some(timeout))
        .handle_request(format!(
            "\nHello {name}\nTimeOut Set to {}ms\n",
            timeout.as_millis()
        ))
        .await;
    match response {
        Ok(response) => HttpResponse::Ok().body(response),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // instantiate a router
    let router = Router::default();
    // tokio spawn workers
    router.tokio_spawn_workers(4, worker);
    // tokio spawn router loops
    router.tokio_spawn();
    // create actix web data from router
    let data = Data::from(Arc::new(router));
    // create actix web app
    HttpServer::new(move || App::new().app_data(Data::clone(&data)).service(hello))
        .bind(("0.0.0.0", 8080))
        .unwrap()
        .run()
        .await
}
