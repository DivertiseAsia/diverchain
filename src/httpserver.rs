use std::sync::Mutex;
use std::sync::Arc;
// use std::net::TcpListener;
// use std::net::TcpStream;
// use std::io::{Read,Write};
use std::thread;

use crate::task::*;

use actix_web::{web, get, post, put, patch, delete,  web::Json, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;

struct AppState {
    map: Arc<Mutex<MapContainer>>,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world?!")
}

#[post("/echo")]
async fn echo(item: Json<MyObj>) -> Json<String> {
    return Json("hello world".to_string());
}  


#[get("/tasks")]
async fn tasks(data: web::Data<AppState>) -> impl Responder {
    let mut json_vec: Vec<Task> = Vec::new();
    let map = &data.map;
    let tasks = &map.lock().unwrap().tasks;

    for (_task_id, task) in tasks {
        println!("HASDHASDHASDHAS");
        json_vec.push(task.clone());
    }

    return web::Json(json_vec);
}

#[post("/tasks")]
async fn create_task(req_body: Json<Task>, data: web::Data<AppState>) -> impl Responder {
    let map = &data.map;
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    tasks_map.insert(req_body.id.as_ref().unwrap().clone(), req_body.into_inner());
    HttpResponse::Ok().body("Successfully added task")
}

#[put("/tasks/{input_id}")]
async fn destructive_update(req_body: Json<Task>, data: web::Data<AppState>, info: web::Path<String>) -> impl Responder {
    let map = &data.map;
    let input_id = info.into_inner();
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let removed: Option<Task> = tasks_map.remove(&input_id); 

    match removed {
        | Some(_) => {
            let mut task_item = req_body.into_inner();
            task_item.id = Some(input_id.clone());
            tasks_map.insert(input_id, task_item);
            HttpResponse::Ok().body("Successfully replaced task")
        }

        | None => {
            HttpResponse::Ok().body("Failed to replace task: task with given id not found")  
        }
    }
}
    


#[patch("/tasks/{input_id}")]
async fn partial_update(req_body: Json<Task>, data: web::Data<AppState>, info: web::Path<String>) -> impl Responder {
    let map = &data.map;
    let input_id = info.into_inner();
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let editable: Option<&mut Task> = tasks_map.get_mut(&input_id); 
    
    match editable {
        | Some(task_edit) => {
            let mut task_item = req_body.into_inner();
            
            task_edit.id = task_item.id;
            task_edit.content = task_item.content;
            task_edit.vote = task_item.vote;
            task_edit.deadline = task_item.deadline;
            task_edit.status = task_item.status;
            task_edit.voted = task_item.voted;
            task_edit.creator = task_item.creator;
            task_edit.detail = task_item.detail;
            task_edit.comments = task_item.comments;

            HttpResponse::Ok().body("Successfully updated task")
        }

        | None => {
            HttpResponse::Ok().body("Failed to replace task: task with given id not found")
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

#[get("/tasks/{task_id}/comments")]
async fn get_task_comments(data: web::Data<AppState>, info: web::Path<String>) -> impl Responder {
    let map = &data.map;
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let input_id = info.into_inner();
    let chosen_task = tasks_map.get(&input_id);

    match chosen_task {
        | Some(task) => {
            let comments = &task.comments;
            let serialized: Result<String, serde_json::Error> = serde_json::to_string(&comments);

            match serialized {  
                | Ok(ref _val) => {
                    let comments_json = serialized.unwrap();
                    HttpResponse::Ok().body(comments_json)
                } 
                | Err(_) => {
                    HttpResponse::Ok().body("Failed to serialize HashMap")
                }
            }
        }

        | None => {
            HttpResponse::Ok().body("Failed to find task with given ID")
        }
    }
}

#[get("/tasks/{task_id}/user_comments")]
async fn get_task_comment_for_user(req_body: Json<UserInfo>, data: web::Data<AppState>, info: web::Path<String>) -> impl Responder {
    let map = &data.map;
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let task_id = info.into_inner();
    let user_id = &req_body.user_id;

    let chosen_task = tasks_map.get(&task_id);

    match chosen_task {
        | Some(task) => {
            let comments = &task.comments;
            let user_comment = comments.get(user_id);

            match user_comment {  
                | Some(comment) => {
                    HttpResponse::Ok().body(comment.to_string())
                } 
                | None => {
                    HttpResponse::Ok().body("Input user has not left a comment on this task")
                }
            }
        }

        | None => {
            HttpResponse::Ok().body("Failed to find task with given ID")
        }
    }
}

#[post("/tasks/{task_id}/comments")]
async fn post_task_comment(req_body: Json<NewComment>, data: web::Data<AppState>, info: web::Path<String>) -> impl Responder {
    let map = &data.map;
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let task_id = info.into_inner();

    let user_id = &req_body.user_id;
    let comment_string = &req_body.comment;

    let editable: Option<&mut Task> = tasks_map.get_mut(&task_id); 

    match editable {
        | Some(task) => {
            let comments = &mut task.comments;
            comments.insert(user_id.to_string(), comment_string.to_string());
            HttpResponse::Ok().body("Successfully added comment")
        }

        | None => {
            HttpResponse::Ok().body("Failed to find task with given ID")
        }
    }
}

#[put("/tasks/{task_id}/comments")]
async fn put_task_comment(req_body: Json<NewComment>, data: web::Data<AppState>, info: web::Path<String>) -> impl Responder {
    let map = &data.map;
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let user_id = &req_body.user_id;
    let comment_string = &req_body.comment;

    let task_id = info.into_inner();
    let editable: Option<&mut Task> = tasks_map.get_mut(&task_id); 

    match editable {
        | Some(task) => {
            let comments = &mut task.comments;
            let removed: Option<String> = comments.remove(user_id); 

            match removed {
                | Some(_) => {
                    comments.insert(user_id.to_string(), comment_string.to_string());
                    HttpResponse::Ok().body("Successfully replaced comment")
                }

                | None => {
                    HttpResponse::Ok().body("Given user has not left a comment")
                }
            }
        }

        | None => {
            HttpResponse::Ok().body("Failed to find task with given ID")
        }
    }
}

#[patch("/tasks/{task_id}/comments")]
async fn patch_task_comment(req_body: Json<NewComment>, data: web::Data<AppState>, info: web::Path<String>) -> impl Responder {
    let map = &data.map;
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let user_id = &req_body.user_id;
    let comment_string = &req_body.comment;

    let task_id = info.into_inner();
    let editable: Option<&mut Task> = tasks_map.get_mut(&task_id); 

    match editable {
        | Some(task) => {
            let comments = &mut task.comments;
            let editable: Option<&mut String> = comments.get_mut(user_id); 

            match editable {
                | Some(content) => {
                    *content = comment_string.to_string();
                    HttpResponse::Ok().body("Successfully replaced comment")
                }

                | None => {
                    HttpResponse::Ok().body("Given user has not left a comment")
                }
            }
        }

        | None => {
            HttpResponse::Ok().body("Failed to find task with given ID")
        }
    }
}

#[delete("/tasks/{task_id}/comments")]
async fn delete_task_comment(req_body: Json<NewComment>, data: web::Data<AppState>, info: web::Path<String>) -> impl Responder {
    let map = &data.map;
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let user_id = &req_body.user_id;
    let comment_string = &req_body.comment;

    let task_id = info.into_inner();
    let editable: Option<&mut Task> = tasks_map.get_mut(&task_id); 

    match editable {
        | Some(task) => {
            let comments = &mut task.comments;
            let removed: Option<String> = comments.remove(user_id); 

            match removed {
                | Some(_) => {
                    HttpResponse::Ok().body("Successfully removed comment")
                }

                | None => {
                    HttpResponse::Ok().body("Given user has not left a comment")
                }
            }
        }

        | None => {
            HttpResponse::Ok().body("Failed to find task with given ID")
        }
    }
}

#[post("/tasks/{task_id}/vote")]
async fn post_add_vote(req_body: Json<NewVote>, data: web::Data<AppState>, info: web::Path<String>) -> impl Responder {
    let map = &data.map;
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let user_id = &req_body.user_id;

    let task_id = info.into_inner();
    let editable: Option<&mut Task> = tasks_map.get_mut(&task_id); 

    match editable {
        | Some(task) => {
            let voters = &mut task.voted; 

            if voters.contains_key(user_id) {
                HttpResponse::Ok().body("Cannot vote twice on same task")

            } else {
                voters.insert(user_id.to_string(), None);
                task.vote = task.vote + 1;
                HttpResponse::Ok().body("Successfully added vote")
            }
        }

        | None => {
            HttpResponse::Ok().body("Failed to find task with given ID")
        }
    }
}

#[post("/tasks/{task_id}/unvote")]
async fn post_unvote(req_body: Json<NewVote>, data: web::Data<AppState>, info: web::Path<String>) -> impl Responder {
    let map = &data.map;
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let user_id = &req_body.user_id;

    let task_id = info.into_inner();
    let editable: Option<&mut Task> = tasks_map.get_mut(&task_id); 

    match editable {
        | Some(task) => {
            let voters = &mut task.voted; 

            if voters.contains_key(user_id) {
                voters.remove(&user_id.to_string());
                task.vote = task.vote - 1;
                HttpResponse::Ok().body("Successfully removed vote")
            } else {
                HttpResponse::Ok().body("User does not a vote on this task")
            }
        }

        | None => {
            HttpResponse::Ok().body("Failed to find task with given ID")
        }
    }
}

#[post("/tasks/{task_id}/validate")]
async fn post_add_validation(req_body: Json<NewValidation>, data: web::Data<AppState>, info: web::Path<String>) -> impl Responder {
    let map = &data.map;
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let user_id = &req_body.user_id;
    let signature_string = &req_body.signature;

    let task_id = info.into_inner();
    let editable: Option<&mut Task> = tasks_map.get_mut(&task_id); 

    match editable {
        | Some(task) => {
            let voters = &mut task.voted; 

            if voters.contains_key(user_id) {
                if voters.get(user_id) == None {
                    voters.insert(user_id.to_string(), Some(signature_string.to_string()));
                    HttpResponse::Ok().body("User successfully validated task")
                } else {
                    HttpResponse::Ok().body("User already validated this task")
                }
                    
            } else {
                HttpResponse::Ok().body("User has not voted on given task")
            }
        }

        | None => {
            HttpResponse::Ok().body("Failed to find task with given ID")
        }
    }
}

#[post("/tasks/{task_id}/unvalidate")]
async fn post_remove_validation(req_body: Json<NewValidation>, data: web::Data<AppState>, info: web::Path<String>) -> impl Responder {
    let map = &data.map;
    let mut maps = (*map).lock().unwrap();
    let tasks_map = &mut maps.tasks;

    let user_id = &req_body.user_id;
    let signature_string = &req_body.signature;

    let task_id = info.into_inner();
    let editable: Option<&mut Task> = tasks_map.get_mut(&task_id); 

    match editable {
        | Some(task) => {
            let voters = &mut task.voted; 

            if voters.contains_key(user_id) {
                if voters.get(user_id) == None {
                    HttpResponse::Ok().body("User has not validated task yet")
                } else {
                    voters.insert(user_id.to_string(), None);
                    HttpResponse::Ok().body("Successfully removed validation of user")
                }
            } else {
                HttpResponse::Ok().body("User does not a vote on this task")
            }
        }

        | None => {
            HttpResponse::Ok().body("Failed to find task with given ID")
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
        let cors = Cors::permissive();

        App::new()
                .app_data(web::Data::new(AppState {
                map: map.clone(),
            }))
            .app_data(web::JsonConfig::default().limit(4096))
            .wrap(cors)
            .service(hello)
            .service(echo)
            .service(tasks)
            .service(create_task)
            .service(destructive_update)
            .service(partial_update)
            .service(delete_task)
            .service(get_task_comments)
            .service(get_task_comment_for_user)
            .service(post_task_comment)
            .service(put_task_comment)
            .service(patch_task_comment)
            .service(delete_task_comment)
            .service(post_add_vote)
            .service(post_unvote)
            .service(post_add_validation)
            .service(post_remove_validation)
            .route("/kill", web::get().to(kill_me))
    })
    .workers(8)
    .bind(("localhost", 7878))?
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
