use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use mongodb::{
    bson::{doc, Document},
    Client,
};

#[get("/stock-item/{part_no}")]
async fn get_stock_item(client: web::Data<Client>, path: web::Path<(String)>) -> impl Responder {
    let (part_no) = path.into_inner();
    let collection: mongodb::Collection<Document> =
        client.database("stock").collection("stockItems");
    let stock_item = collection
        .find_one(doc! { "part_no": part_no }, None)
        .await;

    match stock_item.unwrap () {
        Some (x) =>  HttpResponse::Ok().body(x.to_string()),
        None => HttpResponse::NotFound().body("Stock item not found"),
    }
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
            .service(get_stock_item)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
