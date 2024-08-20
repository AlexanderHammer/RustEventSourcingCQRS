mod stock_event;
mod request;

use actix_web::{delete, error, post, put, web, App, HttpResponse, HttpServer, Responder};
use eventstore::{AppendToStreamOptions, Client, EventData, ExpectedRevision, StreamPosition};
use std::error::Error;
use std::str::FromStr;
use futures::StreamExt;
use uuid::Uuid;
use stock_event::StockEvent;
use request::{CreateStockItem, AdjustStockItem, DeleteStockItem, CreateGenericEvent, GenericEvent};

const STREAM_PREFIX: &str = "stockItem";
const MAX_SIZE: usize = 262_144; // max payload size is 256k


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

#[post("/stock-item")]
async fn post_stock_item(es_client: web::Data<Client>, payload: web::Payload) -> impl Responder {
    let body = payload_to_bytes_mut(payload).await?;
    let command = serde_json::from_slice::<CreateStockItem>(&body)?;
    let evt = EventData::json(StockEvent::CREATE.to_string(), &command)?.id(Uuid::new_v4());

    let options =
        AppendToStreamOptions::default().expected_revision(ExpectedRevision::NoStream);

    let stream_name = format!("{}-{}", STREAM_PREFIX, command.part_no);
    let append_result = es_client.append_to_stream(stream_name, &options, evt);

    match append_result.await {
        Ok(_) => Ok(HttpResponse::Accepted()),
        Err(_) => Err(error::ErrorExpectationFailed("Stock item already exists")),
    }
}

#[post("/stock-item/{part_no}/add/{increment}")]
async fn add_amount(es_client: web::Data<Client>, path: web::Path<(String, f64)>) -> impl Responder {
    let (part_no, increment) = path.into_inner();
    let stream_name = format!("{}-{}", STREAM_PREFIX, &part_no);
    let read_stream_options = eventstore::ReadStreamOptions::default()
        .position(StreamPosition::End)
        .max_count(1);

    if let Ok(mut stream) = es_client.read_stream(&*stream_name, &read_stream_options).await {
        while let Ok(Some(event)) = stream.next().await
        {
            let mut _new_total: f64 = 0.0;
            let recorded_event = event.get_original_event();
            if let Ok(event_type) = StockEvent::from_str(recorded_event.event_type.as_str()) {
                match event_type {
                    StockEvent::ADD => match recorded_event.as_json::<AdjustStockItem>() {
                        Ok(event) => _new_total = event.total + increment,
                        Err(error) => return Err(error::ErrorExpectationFailed(error.to_string())),
                    }
                    StockEvent::CREATE => match recorded_event.as_json::<CreateStockItem>() {
                        Ok(event) => _new_total = event.total + increment,
                        Err(error) => return Err(error::ErrorExpectationFailed(error.to_string())),
                    },
                    StockEvent::SET => match recorded_event.as_json::<AdjustStockItem>() {
                        Ok(event) => _new_total = event.total + increment,
                        Err(error) => return Err(error::ErrorExpectationFailed(error.to_string())),
                    },
                    StockEvent::DELETE => {
                        return Err(error::ErrorExpectationFailed("Stock item deleted"));
                    }
                }
            } else {
                return Err(error::ErrorExpectationFailed("Unknown event type"));
            }

            if _new_total < 0.0 {
                return Err(error::ErrorExpectationFailed("New total is less than 0"));
            }

            let command = AdjustStockItem { part_no, increment, total: _new_total };
            let evt = EventData::json(StockEvent::ADD.to_string(), &command)?.id(Uuid::new_v4());
            let options = AppendToStreamOptions::default().
                expected_revision(ExpectedRevision::Exact(recorded_event.revision));
            let append_result = es_client.append_to_stream(&*stream_name, &options, evt);

            return match append_result.await {
                Ok(_) => Ok(HttpResponse::Accepted()),
                Err(_) => Err(error::ErrorExpectationFailed("Append failed")),
            };
        }
    }
    Err(error::ErrorFailedDependency("Stream must exist before adjusting amount"))
}

#[put("/stock-item/{part_no}/set/{set_amount}")]
async fn set_amount(es_client: web::Data<Client>, path: web::Path<(String, f64)>) -> impl Responder {
    let (part_no, set_amount) = path.into_inner();
    let command = AdjustStockItem { part_no, increment: set_amount, total: set_amount };
    let evt = EventData::json(StockEvent::SET.to_string(), &command)?.id(Uuid::new_v4());

    let options = AppendToStreamOptions::default()
        .expected_revision(ExpectedRevision::StreamExists);

    let stream_name = format!("{}-{}", STREAM_PREFIX, command.part_no);
    let append_result = es_client.append_to_stream(stream_name, &options, evt);

    match append_result.await {
        Ok(_) => Ok(HttpResponse::Accepted()),
        Err(_) => Err(error::ErrorExpectationFailed("Stream must exist before setting amount")),
    }
}

#[delete("/stock-item/{part_no}")]
async fn delete_stock_item(es_client: web::Data<Client>, path: web::Path<String>) -> impl Responder {
    let part_no = path.into_inner();
    let command = DeleteStockItem { part_no };
    let evt = EventData::json(StockEvent::DELETE.to_string(), &command)?.id(Uuid::new_v4());
    let options = AppendToStreamOptions::default();
    let stream_name = format!("{}-{}", STREAM_PREFIX, command.part_no);
    let append_result = es_client.append_to_stream(stream_name, &options, evt);

    match append_result.await {
        Ok(_) => Ok(HttpResponse::Accepted()),
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
        Ok(_) => Ok(HttpResponse::Accepted()),
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