use std::sync::Mutex;
use std::sync::Arc;
// use std::net::TcpListener;
// use std::net::TcpStream;
// use std::io::{Read,Write};
use std::thread;

use crate::task::*;

use actix_web::{web, get, post, put, patch, delete, App, HttpResponse, HttpServer, Responder};


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
async fn tasks(data: web::Data<AppState>) -> impl Responder {
    let mut json_vec: Vec<Task> = Vec::new();
    let map = &data.map;
    let tasks = &map.lock().unwrap().tasks;

    for (_task_id, task) in tasks {
        json_vec.push(task.clone());
    }

    // json_vec.push(Task {
    //   id: "a".to_string(),
    //   name: "b".to_string(),
    //   detail: "c".to_string(),
    //   duedate: "d".to_string(),
    //   owner: "e".to_string(),
    //   total_vote: 32,
    // });
    // let resp = format!("Task count:{}", tasks.keys().len());
    // HttpResponse::Ok().body(resp)
    return web::Json(json_vec);
}

#[post("/tasks")]
async fn create_task(req_body: String, data: web::Data<AppState>) -> impl Responder {
    println!("ASDOASD");
    let map = &data.map;
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let deserialized: Result<Task, serde_json::Error> = serde_json::from_str(&req_body); 
    println!("{:?}", deserialized.as_ref().unwrap());

    match deserialized {
        | Ok(ref _val) => {
            let task_item = deserialized.unwrap();
            tasks_map.insert(task_item.id.clone(), task_item);
            HttpResponse::Ok().body("Successfully added task")
        } 

        | Error => {
            HttpResponse::Ok().body("Failed to add task due to invalid input")    
        }
    }
}

#[put("/tasks/{input_id}")]
async fn destructive_update(req_body: String, data: web::Data<AppState>, info: web::Path<String>) -> impl Responder {
    let map = &data.map;
    let input_id = info.into_inner();
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let removed: Option<Task> = tasks_map.remove(&input_id); 
    let deserialized: Result<Task, serde_json::Error> = serde_json::from_str(&req_body); 
    
    match deserialized {
        | Ok(ref _val) => {
            match removed {
                | Some(_) => {
                    let mut task_item = deserialized.unwrap();
                    task_item.id = input_id;
                    tasks_map.insert(task_item.id.clone(), task_item);
                    HttpResponse::Ok().body("Successfully replaced task")
                }
        
                | None => {
                    HttpResponse::Ok().body("Failed to replace task: task with given id not found")
                }
            }
        } 

        | Error => {
            HttpResponse::Ok().body("Failed to replace task due to invalid input")    
        }
    }
}

#[patch("/tasks/{input_id}")]
async fn partial_update(req_body: String, data: web::Data<AppState>, info: web::Path<String>) -> impl Responder {
    let map = &data.map;
    let input_id = info.into_inner();
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let editable: Option<&mut Task> = tasks_map.get_mut(&input_id); 
    let deserialized: Result<Task, serde_json::Error> = serde_json::from_str(&req_body); 
    
    match deserialized {
        | Ok(ref _val) => {
            match editable {
                | Some(task_edit) => {
                    let task_item = deserialized.unwrap();

                    task_edit.id = task_item.id;
                    task_edit.name = task_item.name;
                    task_edit.detail = task_item.detail;
                    task_edit.duedate = task_item.duedate;
                    task_edit.owner = task_item.owner;
                    task_edit.total_vote = task_item.total_vote;

                    HttpResponse::Ok().body("Successfully updated task")
                }
        
                | None => {
                    HttpResponse::Ok().body("Failed to replace task: task with given id not found")
                }
            }
        } 

        | Error => {
            HttpResponse::Ok().body("Failed to replace task due to invalid input")    
        }
    }
}

#[delete("/tasks/{input_id}")]
async fn delete_task(data: web::Data<AppState>, info: web::Path<String>) -> impl Responder {
    let map = &data.map;
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let input_id = info.into_inner();
    let removed: Option<Task> = tasks_map.remove(&input_id); 

    match removed {
        | Some(_) => {
            HttpResponse::Ok().body("Successfully removed task")
        }

        | None => {
            HttpResponse::Ok().body("Failed to remove task: task with given id not found")
        }
    }
}


#[allow(unreachable_code)]
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
            .service(create_task)
            .route("/kill", web::get().to(kill_me))
    })
    .workers(8)
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
