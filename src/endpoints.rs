mod task;
use crate::task::*;

// List all the tasks in JSON Format and return via HTTP Protocol
#[get("/tasks")]
pub async fn list() -> HttpResponse {
    // TODO find the last 50 tweets and return them

    let tweets = Tweets { results: vec![] };

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(tweets)
}

// Create a task using JSON Request body
#[post("/tasks")]
pub async fn list() -> HttpResponse {
    // TODO find the last 50 tweets and return them

    let tweets = Tweets { results: vec![] };

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(tweets)
}

// Destructive update(replacement) the task with id using JSON Request body
#[put("/tasks/{id}")]
pub async fn list() -> HttpResponse {
    // TODO find the last 50 tweets and return them

    let tweets = Tweets { results: vec![] };

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(tweets)
}

// Partial update the task with id using JSON Request body
#[patch("/tasks/{id}")]
pub async fn list() -> HttpResponse {
    // TODO find the last 50 tweets and return them

    let tweets = Tweets { results: vec![] };

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(tweets)
}

// Delete a particular task with id
#[delete("/tasks/{id}")]
pub async fn delete(path: Path<(String,)>) -> HttpResponse {
    // TODO delete tweet by ID
    // in any case return status 204

    HttpResponse::NoContent()
        .content_type(APPLICATION_JSON)
        .await
        .unwrap()
}