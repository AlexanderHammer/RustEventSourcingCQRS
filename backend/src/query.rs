mod request;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use mongodb::{bson::{doc}, Client};
use crate::request::StockItem;



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017/?maxIdleTimeMS=12000".into());
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

#[get("/stock-item/{part_no}")]
async fn get_stock_item(client: web::Data<Client>, path: web::Path<String>) -> impl Responder {
    let part_no = path.into_inner();
    let collection: mongodb::Collection<StockItem> =
        client.database("stock").collection("stockItems");
    let stock_item = collection
        .find_one(doc! { "part_no": part_no })
        .await;

    match stock_item.unwrap() {
        Some(x) => HttpResponse::Ok().json(x),
        None => HttpResponse::NotFound().body("Stock item not found"),
    }
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
