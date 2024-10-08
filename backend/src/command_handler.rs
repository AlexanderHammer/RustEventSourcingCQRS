mod request;
mod stock_event;

use crate::request::StockItem;
use eventstore::{
    Client as ESClient, DeleteStreamOptions, ResolvedEvent, RetryOptions, SubscribeToAllOptions,
    SubscriptionFilter,
};
use mongodb::bson::Bson;
use mongodb::options::{ClientOptions, ServerApi, ServerApiVersion};
use mongodb::{bson::doc, Client as MDBClient, Collection};
use request::{AdjustStockItem, CreateStockItem, DeleteStockItem};
use std::error::Error;
use std::str::FromStr;
use stock_event::StockEvent;

const STREAM_PREFIX: &str = "stockItem";
const DATABASE_NAME: &str = "stock";
const COLLECTION_NAME: &str = "stockItems";
const D_ID: &str = "part_no";
const MONGODB_URI: &str = "mongodb://localhost:27017/?maxIdleTimeMS=12000";
const EVENTSTORE_URI: &str = "esdb://admin:changeit@localhost:2113?tls=false";

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Create a new client options object and parse the MongoDB URI
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| MONGODB_URI.into());
    let mut client_options = ClientOptions::parse(uri.as_str()).await.unwrap();
    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    // Create a new client and connect to the server
    let mongo_client = MDBClient::with_options(client_options).unwrap();

    // Create a new EventStoreDB client
    let settings = EVENTSTORE_URI.parse().unwrap();
    let es_client = ESClient::new(settings).unwrap();
    let collection: Collection<StockItem> = mongo_client
        .database(DATABASE_NAME)
        .collection(COLLECTION_NAME);
    if let Err(err) = read_all_events(&es_client, &collection).await {
        tracing::error!("Error while reading events: {}", err);
    }
}

async fn read_all_events(
    es_client: &ESClient,
    collection: &Collection<StockItem>,
) -> Result<(), Box<dyn Error>> {
    let retry = RetryOptions::default().retry_forever();
    let filter = SubscriptionFilter::on_event_type()
        .add_prefix(STREAM_PREFIX)
        .exclude_system_events();
    let options = SubscribeToAllOptions::default()
        .filter(filter)
        .retry_options(retry);
    let mut sub = es_client.subscribe_to_all(&options).await;

    loop {
        let event = sub.next().await?;

        if let Ok(event_type) = StockEvent::from_str(event.get_original_event().event_type.as_str())
        {
            let revision = event.get_original_event().revision;
            match event_type {
                StockEvent::CREATE => match event.get_original_event().as_json() {
                    Ok(create_event) => create(&collection, create_event, revision)
                        .await
                        .unwrap_or_else(|e| {
                            tracing::error!("Error while creating stock item, error: {}", e)
                        }),
                    Err(_) => print_event(&event),
                },
                StockEvent::ADD => match event.get_original_event().as_json() {
                    Ok(adjust_event) => adjust(&collection, adjust_event, revision)
                        .await
                        .unwrap_or_else(|e| {
                            tracing::error!("Error while adding amount to stock item: {}", e)
                        }),
                    Err(_) => print_event(&event),
                },
                StockEvent::SET => match event.get_original_event().as_json() {
                    Ok(adjust_event) => set(&collection, adjust_event, revision)
                        .await
                        .unwrap_or_else(|e| {
                            tracing::error!("Error while setting new amount for stock item: {}", e)
                        }),
                    Err(_) => print_event(&event),
                },
                StockEvent::DELETE => match event.get_original_event().as_json() {
                    Ok(delete_event) => delete(
                        &es_client,
                        &collection,
                        delete_event,
                        &event.get_original_stream_id(),
                    )
                    .await
                    .unwrap_or_else(|e| tracing::error!("Error while deleting stock item: {}", e)),
                    Err(_) => print_event(&event),
                },
            }
        };
    }
}

async fn create(
    collection: &Collection<StockItem>,
    _event: CreateStockItem,
    revision: u64,
) -> Result<(), Box<dyn Error>> {
    let filter = doc! { D_ID: doc! { "$regex": &_event.part_no } };
    let stock_item_doc = StockItem {
        part_no: _event.part_no,
        name: _event.name,
        description: _event.description,
        category: _event.category,
        total: _event.total,
        revision,
    };
    let ct = collection.count_documents(filter).await?;
    if ct > 0 {
        collection.insert_one(stock_item_doc).await?;
    } else {
        collection.insert_one(stock_item_doc).await?;
    }
    Ok(())
}

async fn adjust(
    collection: &Collection<StockItem>,
    _event: AdjustStockItem,
    revision: u64,
) -> Result<(), Box<dyn Error>> {
    let filter = doc! { D_ID: &_event.part_no };
    let update =
        doc! { "$set": doc! {"total": _event.total, "revision": Bson::Int64(revision as i64) } };
    collection.update_one(filter, update).await?;
    Ok(())
}

async fn set(
    collection: &Collection<StockItem>,
    _event: AdjustStockItem,
    revision: u64,
) -> Result<(), Box<dyn Error>> {
    let filter = doc! { D_ID: &_event.part_no };
    let update =
        doc! { "$set": doc! {"total": &_event.total, "revision": Bson::Int64(revision as i64)} };
    collection.update_one(filter, update).await?;
    Ok(())
}

async fn delete(
    es_client: &ESClient,
    collection: &Collection<StockItem>,
    _event: DeleteStockItem,
    stream_name: &str,
) -> Result<(), Box<dyn Error>> {
    let filter = doc! { D_ID: &_event.part_no };
    collection.delete_one(filter).await?;
    es_client
        .delete_stream(stream_name, &DeleteStreamOptions::default())
        .await?;
    Ok(())
}

fn print_event(event: &ResolvedEvent) {
    tracing::info!(
        "Errored received event {}@{}, {:?}",
        event.get_original_event().revision,
        event.get_original_event().stream_id,
        event.get_original_event()
    );
}
