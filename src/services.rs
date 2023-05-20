use actix_web::{
  get, post,
  web::{Data, Json, Path},
  Responder, HttpResponse
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow};
use crate::AppState;

#[derive(Serialize, FromRow)]
struct User {
  id: String,
  name: String,
  email: String,
}

#[derive(FromRow)]
struct Token {
  id: String,
  sessionToken: String,
  userId: String,
  expires: NaiveDateTime,
}

#[derive(Serialize, FromRow)]
struct Event {
  id: String,
  date: String,
  description: String,
}

#[derive(Deserialize)]
pub struct CreateEventBody {
    pub date: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct GetUserId {
  pub id: String
}

#[get("/users/{id}")]
pub async fn fetch_users(state: Data<AppState>, path: Path<String>) -> impl Responder {

    let id: String = path.into_inner();

    let result: sqlx::Result<Token> = sqlx::query_as!(
      Token,
      "SELECT * FROM Session WHERE userId = ? AND CURTIME() < expires",
      id,
    )
      .fetch_one(&state.db)
      .await;

    match result {
      Err(_) => {
        match sqlx::query_as::<_, User>("SELECT id, name, email FROM User")
        .fetch_all(&state.db)
        .await {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(_) => HttpResponse::NotFound().json("No users found"),
        }
      }
      Ok(_) => HttpResponse::InternalServerError().json("Invalid request")
    }

}

#[get("/users/{author_email}/events")]
pub async fn fetch_user_events(state: Data<AppState>, path: Path<String>) -> impl Responder {

  let author_email: String = path.into_inner();
  match sqlx::query_as::<_, Event>(
    "SELECT id, date, description FROM Event WHERE authorEmail = $1"
  )
    .bind(author_email)
    .fetch_all(&state.db)
    .await {
      Ok(events) => HttpResponse::Ok().json(events),
      Err(_) => HttpResponse::NotFound().json("No events found"),
    }
}

#[post("/users/{author_email}/events")]
pub async fn create_user_event(state: Data<AppState>, path: Path<String>, body: Json<CreateEventBody>) -> impl Responder {
  let author_email: String = path.into_inner();
  match sqlx::query_as::<_, Event>(
    "INSERT INTO Event (description, date, authorEmail) VALUES ($1, $2, $3) RETURNING id, date ,description"
  )
    .bind(body.description.to_string())
    .bind(body.date.to_string())
    .bind(author_email)
    .fetch_one(&state.db)
    .await {
      Ok(event) => HttpResponse::Ok().json(event),
      Err(_) => HttpResponse::InternalServerError().json("Failed to create event"),
    }
}