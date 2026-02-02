use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::controllers::word_controller;

pub fn word_routes() -> Router {
    Router::new()
        .route("/words/{language}/{term}", get(word_controller::get_word))
        .route("/words", post(word_controller::create_word))
        .route("/words/{id}", delete(word_controller::delete_word))
}
