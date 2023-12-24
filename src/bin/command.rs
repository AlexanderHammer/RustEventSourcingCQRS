use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use eventstore::{Client as ESClient, EventData};
use mongodb::{
    bson::{doc, Document},
    Client as MongoClient, Collection,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Foo {
    pub(crate) is_rust_a_nice_language: bool
}

#[post("/language-vote")]
async fn post_language_vote() -> impl Responder {
    // Creates a client settings for a single node configuration.
    let settings = "esdb://admin:changeit@localhost:2113?tls=false"
        .parse()
        .unwrap();
    let es_client = ESClient::new(settings).unwrap();

    let payload = Foo {
        is_rust_a_nice_language: true,
    };

    // It is not mandatory to use JSON as a data format however EventStoreDB
    // provides great additional value if you do so.
    let evt = EventData::json("language-poll", &payload).unwrap();

    es_client
        .append_to_stream("language-stream", &Default::default(), evt)
        .await
        .unwrap();

    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(post_language_vote)
    })
    .bind(("localhost", 8081))?
    .run()
    .await
}
