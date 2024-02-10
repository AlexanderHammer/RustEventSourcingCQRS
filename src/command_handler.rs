mod stock_event;
mod request;

use eventstore::{Client as ESClient, RetryOptions, SubscribeToAllOptions, SubscriptionFilter};
use std::error::Error;
use std::str::FromStr;
use mongodb::{bson::{doc}, Client as MDBClient};
use mongodb::options::{ClientOptions, ServerApi, ServerApiVersion};
use stock_event::StockEvent;
use request::{CreateStockItem, AdjustStockItem, DeleteStockItem};

#[tokio::main]
async fn main() {
    // Create a new client options object and parse the MongoDB URI
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let mut client_options = ClientOptions::parse_async(uri).await.unwrap();
    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    // Create a new client and connect to the server
    let mongo_client = MDBClient::with_options(client_options).unwrap();

    // Create a new EventStoreDB client
    let settings = "esdb://admin:changeit@localhost:2113?tls=false"
        .parse()
        .unwrap();
    let es_client = ESClient::new(settings).unwrap();
    if let Err(err) = read_all_events(&es_client, &mongo_client).await {
        eprintln!("Error while reading events: {}", err);
    }
}

async fn read_all_events(es_client: &ESClient, mdb_client: &MDBClient) -> Result<(), Box<dyn Error>> {
    let retry = RetryOptions::default().retry_forever();
    let filter = SubscriptionFilter::on_event_type()
        .add_prefix("stockItem")
        .exclude_system_events();
    let options = SubscribeToAllOptions::default()
        .filter(filter)
        .retry_options(retry);
    let mut sub = es_client.subscribe_to_all(&options).await;

    loop {
        let event = sub.next().await?;

       print_event(&event);

        match StockEvent::from_str(event.get_original_event().event_type.as_str()) {
            Ok(event_type) => {
                match event_type {
                    StockEvent::CREATE => {
                        match event.get_original_event().as_json() {
                            Ok(x) => create(mdb_client, x).await.unwrap_or_else(|e| {
                                eprintln!("Error while creating stock item: {}", e);
                            }),
                            Err(_) => print_event(&event),
                        };
                    }
                    StockEvent::ADD => {
                        match event.get_original_event().as_json() {
                            Ok(x) => add(mdb_client, x).await.unwrap_or_else(|e| {
                                eprintln!("Error while adding amount to stock item: {}", e);
                            }),
                            Err(_) => print_event(&event),
                        };
                    }
                    StockEvent::SET => {
                        match event.get_original_event().as_json() {
                            Ok(x) => set(mdb_client, x).await.unwrap_or_else(|e| {
                                eprintln!("Error while setting new amount for stock item: {}", e);
                            }),
                            Err(_) =>print_event(&event),
                        };
                    }
                    StockEvent::DELETE => {
                        match event.get_original_event().as_json() {
                            Ok(x) => delete(mdb_client, x).await.unwrap_or_else(|e| {
                                eprintln!("Error while deleting stock item: {}", e);
                            }),
                            Err(_) => print_event(&event),
                        };
                    }
                }
            }
            Err(_) => {
                /*println!(
                    "Received event {}@{}, {:?}",
                    revision,
                    stream_id,
                    event.get_original_event()
                );*/
            }
        };
    }
}

async fn create(mdb_client: &MDBClient, _event: CreateStockItem) -> Result<(), Box<dyn Error>> {
    let collection: mongodb::Collection<CreateStockItem> = mdb_client.database("stock").collection("stockItems");
    let ct = collection.count_documents(doc! { "part_no": doc! { "$regex": &_event.part_no } }, None).await?;
    if ct > 0 {
        return Err("Stock item already exists".into());
    }
    collection.insert_one(&_event, None).await?;
    Ok(())
}

async fn add(mdb_client: &MDBClient, _event: AdjustStockItem) -> Result<(), Box<dyn Error>> {
    Ok(())
}

async fn set(mdb_client: &MDBClient, _event: AdjustStockItem) -> Result<(), Box<dyn Error>> {
    Ok(())
}

async fn delete(mdb_client: &MDBClient, _event: DeleteStockItem) -> Result<(), Box<dyn Error>> {
    Ok(())
}

fn print_event(event: &eventstore::ResolvedEvent) {
    println!(
        "Received event {}@{}, {:?}",
        event.get_original_event().revision,
        event.get_original_event().stream_id,
        event.get_original_event()
    );
}
