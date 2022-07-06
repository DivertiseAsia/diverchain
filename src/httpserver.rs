use std::net::TcpListener;
use std::net::TcpStream;
use std::io::{Read,Write};
use std::thread;

use actix_web::{web, get, post, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world?!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn handle_by_actix() -> std::io::Result<()> {
  HttpServer::new(|| {
      App::new()
          .service(hello)
          .service(echo)
          .route("/hey", web::get().to(manual_hello))
  })
  .bind(("0.0.0.0", 7878))?
  .run()
  .await
}

pub fn start_server() {

    println!("Spinning up HTTP server...\n");
    
    thread::spawn(move || {
      //Fire and forget
      handle_by_actix();
    });
}
