mod stock_event;
mod request;

use actix_web::{delete, error, post, put, web, App, HttpResponse, HttpServer, Responder};
use eventstore::{AppendToStreamOptions, Client, EventData};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{error::Error};
use uuid::Uuid;
use stock_event::StockEvent;
use request::{CreateStockItem, AdjustStockItem, DeleteStockItem, CreateGenericEvent, GenericEvent};



const STREAM_PREFIX: &str = "stockItem";
const MAX_SIZE: usize = 262_144; // max payload size is 256k


#[post("/stock-item")]
async fn post_stock_item(es_client: web::Data<Client>, payload: web::Payload) -> impl Responder {
    let body = payload_to_bytes_mut(payload).await?;
    let command = serde_json::from_slice::<CreateStockItem>(&body)?;
    let evt = EventData::json(StockEvent::CREATE.to_string(), &command)?.id(Uuid::new_v4());

    let options =
        AppendToStreamOptions::default().expected_revision(eventstore::ExpectedRevision::NoStream);

    let stream_name = format!("{}-{}", STREAM_PREFIX, command.part_no);
    let append_result = es_client.append_to_stream(stream_name, &options, evt);

    match append_result.await {
        Ok(_) => return Ok(HttpResponse::Accepted()),
        Err(_) => Err(error::ErrorExpectationFailed("Stock item already exists")),
    }
}

#[post("/stock-item/add/{part_no}/{count}")]
async fn add_amount(
    es_client: web::Data<Client>,
    path: web::Path<(String, u64)>,
) -> impl Responder {
    let (part_no, count) = path.into_inner();
    let command = AdjustStockItem { part_no, count };
    let evt = EventData::json(StockEvent::ADD.to_string(), &command)?.id(Uuid::new_v4());

    let options = AppendToStreamOptions::default()
        .expected_revision(eventstore::ExpectedRevision::StreamExists);

    let stream_name = format!("{}-{}", STREAM_PREFIX, command.part_no);
    let append_result = es_client.append_to_stream(stream_name, &options, evt);

    match append_result.await {
        Ok(_) => return Ok(HttpResponse::Accepted()),
        Err(_) => Err(error::ErrorExpectationFailed("Who knows")),
    }
}

#[put("/stock-item/set/{part_no}/{count}")]
async fn set_amount(
    es_client: web::Data<Client>,
    path: web::Path<(String, u64)>,
) -> impl Responder {
    let (part_no, count) = path.into_inner();
    let command = AdjustStockItem { part_no, count };
    let evt = EventData::json(StockEvent::SET.to_string(), &command)?.id(Uuid::new_v4());

    let options = AppendToStreamOptions::default()
        .expected_revision(eventstore::ExpectedRevision::StreamExists);

    let stream_name = format!("{}-{}", STREAM_PREFIX, command.part_no);
    let append_result = es_client.append_to_stream(stream_name, &options, evt);

    match append_result.await {
        Ok(_) => return Ok(HttpResponse::Accepted()),
        Err(_) => Err(error::ErrorExpectationFailed("Who knows")),
    }
}

#[delete("/stock-item/{part_no}")]
async fn delete_stock_item(
    es_client: web::Data<Client>,
    path: web::Path<String>,
) -> impl Responder {
    let part_no = path.into_inner();
    let command = DeleteStockItem { part_no };
    let evt = EventData::json(StockEvent::DELETE.to_string(), &command)?.id(Uuid::new_v4());

    let options = AppendToStreamOptions::default()
        .expected_revision(eventstore::ExpectedRevision::StreamExists);

    let stream_name = format!("{}-{}", STREAM_PREFIX, command.part_no);
    let append_result = es_client.append_to_stream(stream_name, &options, evt);

    match append_result.await {
        Ok(_) => return Ok(HttpResponse::Accepted()),
        Err(_) => Err(error::ErrorExpectationFailed("Who knows")),
    }
}

#[post("/generic-event")]
async fn post_generic_event(es_client: web::Data<Client>, payload: web::Payload) -> impl Responder {
    let body = payload_to_bytes_mut(payload).await?;
    let command = serde_json::from_slice::<CreateGenericEvent>(&body)?;
    if command.stream_prefix.is_empty()
        || command.stream_name.is_empty()
        || command.event_type.is_empty()
    {
        return Err(error::ErrorBadRequest(format!(
            "Stream prefix, stream name and event type are required: {}, {}, {}",
            command.stream_prefix, command.stream_name, command.event_type
        )));
    }
    let evt = EventData::json(&command.event_type, GenericEvent { data: command.data })?
        .id(Uuid::new_v4());
    let options = AppendToStreamOptions::default();
    let stream_name = format!("{}-{}", command.stream_prefix, command.stream_name);

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
            .service(add_amount)
            .service(delete_stock_item)
            .service(set_amount)
            .app_data(web::Data::new(es_client.clone()))
    })
        .bind(("localhost", 8081))?
        .run()
        .await
}
