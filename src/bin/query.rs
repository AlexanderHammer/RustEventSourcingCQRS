use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use mongodb::{
    bson::{doc, Document},
    Client as MongoClient, Collection,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Foo {
    pub(crate) is_rust_a_nice_language: bool
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

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get_movie)
            .route("/hey", web::get().to(manual_hello))
    })
        .bind(("localhost", 8080))?
        .run()
        .await
}
