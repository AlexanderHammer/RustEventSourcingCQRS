use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use eventstore::{Client as ESClient, EventData};
use mongodb::{
    bson::{doc, Document},
    Client as MongoClient, Collection,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Foo {
    is_rust_a_nice_language: bool,
}

#[get("/")]
async fn get_movie() -> impl Responder {
    // Replace the placeholder with your Atlas connection string
    let uri = "mongodb://localhost:27017";
    // Create a new client and connect to the server
    let mongo_client = MongoClient::with_uri_str(uri).await.unwrap();
    // Get a handle on the movies collection
    let database = mongo_client.database("sample_mflix");
    let my_coll: Collection<Document> = database.collection("movies");
    // Find a movie based on the title value
    let my_movie = my_coll
        .find_one(doc! { "title": "The Perils of Pauline" }, None)
        .await;
    // Print the document
    println!("Found a movie:\n{:#?}", my_movie);

    HttpResponse::Ok().body("Hello world!")
}

#[post("/event-store")]
async fn event_store() -> impl Responder {
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

    let mut stream = es_client
        .read_stream("language-stream", &Default::default())
        .await
        .unwrap();

    while let Some(event) = stream.next().await.unwrap() {
        let event = event.get_original_event().as_json::<Foo>().unwrap();

        // Do something productive with the result.
        println!("{:?}", event);
    }

    HttpResponse::Ok().body("Hey there!")
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get_movie)
            .service(event_store)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
