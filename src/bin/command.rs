use actix_web::{error, post, web, App, HttpResponse, HttpServer, Responder};
use eventstore::{Client as ESClient, EventData};
use futures::StreamExt;
use mongodb::{
    bson::{doc, Document},
    Client as MongoClient, Collection,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct LanguageVote {
    pub(crate) lang: LanguageKind,
}

#[derive(Serialize, Deserialize, Debug)]
enum LanguageKind {
    JavaScript,
    CSharp,
    Rust,
    CPlusPlus,
    C,
    Python,
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[post("/language-vote")]
async fn post_language_vote(mut payload: web::Payload) -> impl Responder {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.unwrap();
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    let obj = serde_json::from_slice::<LanguageVote>(&body)?;
    let evt = EventData::json("language-poll", &obj)?;

    let settings = "esdb://admin:changeit@localhost:2113?tls=false"
        .parse()
        .unwrap();
    let es_client = ESClient::new(settings).unwrap();

    es_client
        .append_to_stream("language-stream", &Default::default(), evt)
        .await
        .unwrap();

    Ok(HttpResponse::Ok().body(serde_json::to_string_pretty(&obj).unwrap()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || App::new().service(post_language_vote))
        .bind(("localhost", 8081))?
        .run()
        .await
}
