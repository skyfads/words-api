use axum::{
    extract::{Json, Path},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::services::db;

#[derive(Deserialize)]
pub struct CreateWordRequest {
    pub language: String,
    pub term: String,
    pub definition: String,
}

#[derive(Serialize)]
pub struct WordResponse {
    pub id: i32,
    pub language: String,
    pub term: String,
    pub definition: String,
}

pub async fn get_word(
    Path((language, term)): Path<(String, String)>,
) -> Result<Json<WordResponse>, StatusCode> {
    let word = db::get_word(&language, &term)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match word {
        Some((id, term, definition)) => Ok(Json(WordResponse {
            id,
            language,
            term,
            definition,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_word(
    Json(payload): Json<CreateWordRequest>,
) -> Result<Json<WordResponse>, StatusCode> {
    let lang_id = db::create_language(&payload.language)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let word_id = db::create_word(lang_id, &payload.term, &payload.definition)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(WordResponse {
        id: word_id,
        language: payload.language,
        term: payload.term,
        definition: payload.definition,
    }))
}

pub async fn delete_word(Path(id): Path<i32>) -> Result<StatusCode, StatusCode> {
    db::delete_word(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
