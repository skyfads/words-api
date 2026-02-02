use axum::{
    extract::{Json, Path, Query},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::services::db;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateWordRequest {
    pub language: String,
    pub term: String,
    pub definition: String,
}

#[derive(Serialize)]
pub struct SentenceResponse {
    pub id: i32,
    pub example: String,
    pub meaning: Option<String>,
}

#[derive(Serialize)]
pub struct WordResponse {
    pub id: i32,
    pub language: String,
    pub term: String,
    pub definition: String,
}

#[derive(Serialize)]
pub struct WordResponseWithSentences {
    pub id: i32,
    pub language: String,
    pub term: String,
    pub definition: String,
    pub sentences: Vec<SentenceResponse>,
}

pub async fn get_word(
    Path((language, term)): Path<(String, String)>,
) -> Result<Json<WordResponseWithSentences>, StatusCode> {
    let word = db::get_word(&language, &term)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match word {
        Some((id, term, definition)) => {
            let sentences_data = db::get_sentences_by_word(id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let sentences = sentences_data
                .into_iter()
                .map(|(s_id, example, meaning)| SentenceResponse {
                    id: s_id,
                    example,
                    meaning,
                })
                .collect();

            Ok(Json(WordResponseWithSentences {
                id,
                language,
                term,
                definition,
                sentences,
            }))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_all_words(
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<WordResponse>>, StatusCode> {
    // Default to 20 items per page starting at 0
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    let words_data = db::get_all_words(limit, offset)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = words_data
        .into_iter()
        .map(|(id, language, term, definition)| WordResponse {
            id,
            language,
            term,
            definition,
        })
        .collect();

    Ok(Json(response))
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
