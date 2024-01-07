use std::error::Error;
use eventstore::{Client, RetryOptions, SubscribeToAllOptions};

#[tokio::main]
async fn main(){
    let settings = "esdb://admin:changeit@localhost:2113?tls=false"
    .parse()
    .unwrap();
    let es_client = Client::new(settings).unwrap();
    read_all_events(es_client).await;
}   


async fn read_all_events(client: Client) -> Result<(), Box<dyn Error>>  {
    let retry = RetryOptions::default().retry_forever();
    let options = SubscribeToAllOptions::default().retry_options(retry);
    let mut stream = client.subscribe_to_all(&options).await;

    loop {
        let event = stream.next().await?;
        if event.get_original_event().event_type.starts_with("$") {
            continue;
        }
        print!("Received event: {:?}", event.get_original_event().event_type);
    }
}