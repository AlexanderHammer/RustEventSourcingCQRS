mod stock_event;

use eventstore::{Client, RetryOptions, SubscribeToAllOptions, SubscriptionFilter};
use std::error::Error;
use std::str::FromStr;
use stock_event::StockEvent;

#[tokio::main]
async fn main() {
    let settings = "esdb://admin:changeit@localhost:2113?tls=false"
        .parse()
        .unwrap();
    let es_client = Client::new(settings).unwrap();
    if let Err(err) = read_all_events(es_client).await {
        eprintln!("Error while reading events: {}", err);
    }
}

async fn read_all_events(client: Client) -> Result<(), Box<dyn Error>> {
    let retry = RetryOptions::default().retry_forever();
    let filter = SubscriptionFilter::on_event_type()
        .add_prefix("stockItem")
        .exclude_system_events();
    let options = SubscribeToAllOptions::default()
        .filter(filter)
        .retry_options(retry);
    let mut sub = client.subscribe_to_all(&options).await;

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
