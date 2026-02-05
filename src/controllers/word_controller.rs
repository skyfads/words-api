use axum::{
    extract::{Json, Path, Query},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::extra::normalize_input;
use crate::services::db;
use crate::services::word::init_word_service;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateWordRequest {
    pub term: String,
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
    pub sentences: Vec<SentenceResponse>,
}

pub async fn get_word(
    Path((language, term)): Path<(String, String)>,
) -> Result<Json<WordResponse>, StatusCode> {
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

            Ok(Json(WordResponse {
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
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    let words_data = db::get_all_words(limit, offset)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if words_data.is_empty() {
        return Ok(Json(vec![]));
    }

    let word_ids: Vec<i32> = words_data.iter().map(|(id, ..)| *id).collect();

    let all_sentences = db::get_sentences_by_word_ids(&word_ids)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut sentences_map: HashMap<i32, Vec<SentenceResponse>> = HashMap::new();
    for (s_id, w_id, example, meaning) in all_sentences {
        sentences_map
            .entry(w_id)
            .or_default()
            .push(SentenceResponse {
                id: s_id,
                example,
                meaning,
            });
    }

    let response = words_data
        .into_iter()
        .map(|(id, language, term, definition)| {
            let sentences = sentences_map.remove(&id).unwrap_or_default();
            WordResponse {
                id,
                language,
                term,
                definition,
                sentences,
            }
        })
        .collect();

    Ok(Json(response))
}

pub async fn create_word(
    Json(payload): Json<CreateWordRequest>,
) -> Result<Json<WordResponse>, StatusCode> {
    let clean_term = normalize_input(&payload.term);

    if clean_term.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let word_service = init_word_service().await;
    let ai_word = word_service.get_detail(&clean_term).await;
    let lang_id = db::create_language(&ai_word.language)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let word_id = db::create_word(lang_id, &ai_word.dictionary_form, &ai_word.definition)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let sentence_id = db::create_sentence(
        word_id,
        &ai_word.sentence.example,
        Some(&ai_word.sentence.meaning),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(WordResponse {
        id: word_id,
        language: ai_word.language,
        term: ai_word.dictionary_form,
        definition: ai_word.definition,
        sentences: vec![SentenceResponse {
            id: sentence_id,
            example: ai_word.sentence.example,
            meaning: Some(ai_word.sentence.meaning),
        }],
    }))
}

pub async fn delete_word(Path(id): Path<i32>) -> Result<StatusCode, StatusCode> {
    db::delete_word(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
