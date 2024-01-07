use actix_web::{error, post, web, App, HttpResponse, HttpServer, Responder};
use eventstore::{AppendToStreamOptions, Client, EventData};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CreateStockItem {
    pub(crate) part_no: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) category: String,
    pub(crate) count: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CreateGenericEvent {
    pub(crate) stream_name: String,
    pub(crate) prefix: String,
    pub(crate) event_type: String,
    pub(crate) data: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GenericEvent {
    pub(crate) data: HashMap<String, String>,
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[post("/stock-item")]
async fn post_stock_item(es_client: web::Data<Client>, payload: web::Payload) -> impl Responder {
    let body = payload_to_bytes_mut(payload).await?;
    let command = serde_json::from_slice::<CreateStockItem>(&body)?;
    let evt = EventData::json("create-stock-item", &command)?.id(Uuid::new_v4());

    let options =
        AppendToStreamOptions::default().expected_revision(eventstore::ExpectedRevision::NoStream);

    let stream_name = format!("stockItem-{}", command.part_no);
    let append_result = es_client.append_to_stream(stream_name, &options, evt);

    match append_result.await {
        Ok(_) => return Ok(HttpResponse::Accepted()),
        Err(_) => Err(error::ErrorExpectationFailed("Stock item already exists")),
    }
}

#[post("/generic-event")]
async fn post_generic_event(es_client: web::Data<Client>, payload: web::Payload) -> impl Responder {
    let body = payload_to_bytes_mut(payload).await?;
    let command = serde_json::from_slice::<CreateGenericEvent>(&body)?;
    if command.prefix.is_empty() || command.stream_name.is_empty() || command.event_type.is_empty()
    {
        return Err(error::ErrorBadRequest(format!(
            "Prefix, stream name and event type are required: {}, {}, {}",
            command.prefix, command.stream_name, command.event_type
        )));
    }
    let evt = EventData::json(&command.event_type, GenericEvent { data: command.data })?
        .id(Uuid::new_v4());
    let options = AppendToStreamOptions::default();
    let stream_name = format!("{}-{}", command.prefix, command.stream_name);

    let append_result = es_client.append_to_stream(stream_name, &options, evt);

    match append_result.await {
        Ok(_) => return Ok(HttpResponse::Accepted()),
        Err(e) => Err(error::ErrorImATeapot(format!(
            "Error occurred while appending event: {}",
            e.to_string()
        ))),
    }
}

async fn payload_to_bytes_mut(mut payload: web::Payload) -> Result<web::BytesMut, Box<dyn Error>> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err("Payload to large".into());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
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
            .service(post_generic_event)
            .app_data(web::Data::new(es_client.clone()))
    })
    .bind(("localhost", 8081))?
    .run()
    .await
}
