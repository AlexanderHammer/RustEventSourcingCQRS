mod stock_event;

use eventstore::{Client as ESClient, RetryOptions, SubscribeToAllOptions, SubscriptionFilter};
use std::error::Error;
use std::str::FromStr;
use mongodb::{
    bson::{doc, Document},
    Client as MDBClient,
};
use mongodb::options::{ClientOptions, ServerApi, ServerApiVersion};
use stock_event::StockEvent;

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
    if let Err(err) = read_all_events(es_client, mongo_client).await {
        eprintln!("Error while reading events: {}", err);
    }
}

async fn read_all_events(es_client: ESClient, mdb_client: MDBClient) -> Result<(), Box<dyn Error>> {
    let retry = RetryOptions::default().retry_forever();
    let filter = SubscriptionFilter::on_event_type()
        .add_prefix("stockItem")
        .exclude_system_events();
    let options = SubscribeToAllOptions::default()
        .filter(filter)
        .retry_options(retry);
    let mut sub = es_client.subscribe_to_all(&options).await;

    mdb_client.database("admin").run_command(doc! { "ping": 1 }, None).await?;
    println!("Pinged your deployment. You successfully connected to MongoDB!");

    loop {
        let event = sub.next().await?;
        let stream_id = event.get_original_stream_id();
        let revision = event.get_original_event().revision;

        let parse_result = StockEvent::from_str(event.get_original_event().event_type.as_str());

        match parse_result {
            Ok(event_type) => {
                match event_type {
                    StockEvent::ADD => {
                        println!("Creating stock item");
                    }
                    StockEvent::CREATE => {
                        println!("Adding stock item");
                    }
                    StockEvent::SET => {
                        println!("Setting stock item");
                    }
                    StockEvent::DELETE => {
                        println!("Deleting stock item");
                    }
                }
            }
            Err(error) => {
                println!(
                    "Received event {}@{}, {:?}",
                    revision,
                    stream_id,
                    event.get_original_event()
                );
            }
        };
    }
}
