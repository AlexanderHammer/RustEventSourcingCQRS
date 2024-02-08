use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use mongodb::{
    bson::{doc, Document},
    Client,
};

#[get("/")]
async fn get_movie(client: web::Data<Client>) -> impl Responder {
    let collection: mongodb::Collection<Document> =
        client.database("sample_mflix").collection("movies");
    // Find a movie based on the title value
    let my_movie = collection
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
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let client = Client::with_uri_str(uri).await.expect("failed to connect");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(get_movie)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
