use actix_web::{error, post, web, App, HttpResponse, HttpServer, Responder};
use eventstore::{Client, EventData, AppendToStreamOptions};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CreateStockItem {
    pub(crate) part_no: String,
    pub(crate) name: String,
    pub(crate) category: String,
    pub(crate) count: u64
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[post("/stock-item")]
async fn post_stock_item(es_client: web::Data<Client>,  mut payload: web::Payload) -> impl Responder {

    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    let obj = serde_json::from_slice::<CreateStockItem>(&body)?;
    let evt = EventData::json("stock-item-created", &obj)?.id(Uuid::new_v4());

    let options = AppendToStreamOptions::default().expected_revision(eventstore::ExpectedRevision::NoStream);

    es_client
        .append_to_stream(obj.part_no, &options, evt)
        .await
        .unwrap();


    Ok(HttpResponse::Ok())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = "esdb://admin:changeit@localhost:2113?tls=false"
        .parse()
        .unwrap();
    let es_client = Client::new(settings).unwrap();

    HttpServer::new(move || {
        App::new()
            .service(post_stock_item)
            .app_data(web::Data::new(es_client.clone()))
    })
    .bind(("localhost", 8081))?
    .run()
    .await
}
