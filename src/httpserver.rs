use std::sync::Mutex;
use std::sync::Arc;
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::{Read,Write};
use std::thread;

use crate::task::*;

use actix_web::{web, get, post, App, HttpResponse, HttpServer, Responder};

struct AppState {
  map: Arc<Mutex<MapContainer>>,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world?!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/tasks")]
pub async fn tasks(data: web::Data<AppState>) -> HttpResponse {
  let map = &data.map;
  let tasks = &map.lock().unwrap().tasks;
  let resp = format!("Task count:{}", tasks.keys().len());
  HttpResponse::Ok().body(resp)
}

async fn kill_me() -> impl Responder {
    std::process::exit(0);
    HttpResponse::Ok().body("RIP")
}

#[actix_web::main]
async fn handle_by_actix(map: Arc<Mutex<MapContainer>>) -> std::io::Result<()> {
  
  HttpServer::new(move || {
      App::new()
            .app_data(web::Data::new(AppState {
              map: map.clone(),
          }))
          .service(hello)
          .service(echo)
          .service(tasks)
          .route("/kill", web::get().to(kill_me))
  })
  .bind(("0.0.0.0", 7878))?
  .run()
  .await
}

pub fn start_server(map: Arc<Mutex<MapContainer>>) {

    println!("Spinning up HTTP server...\n");
    
    thread::spawn(move || {
      //Fire and forget
      handle_by_actix(map).unwrap();
    });
}
