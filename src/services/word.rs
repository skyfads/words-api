use crate::services::ai::init_ai_service;

use serde::{Deserialize, Serialize};
use serde_json;
use std::path::Path;
use tokio::fs;
use tokio::sync::OnceCell;

#[derive(Debug, Deserialize, Serialize)]
pub struct Sentence {
    pub example: String,
    pub meaning: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Word {
    pub dictionary_form: String,
    pub language: String,
    pub definition: String,
    pub sentence: Sentence,
}

pub struct WordService {
    prompt_template: String,
}

impl WordService {
    pub async fn new() -> std::io::Result<Self> {
        let path = Path::new("assets/word_detail_prompt.txt");
        let template = fs::read_to_string(path).await?;
        Ok(WordService {
            prompt_template: template,
        })
    }

    pub async fn get_detail(&self, word: &str) -> Word {
        let prompt = self.prompt_template.replace("{WORD}", word);
        let ai = init_ai_service();
        let answer = ai.chat(&prompt).expect("AI call failed");
        let word_entry: Word = serde_json::from_str(&answer).expect("Failed to parse AI JSON");

        word_entry
    }
}

pub static WORD_SERVICE: OnceCell<WordService> = OnceCell::const_new();

pub async fn init_word_service() -> &'static WordService {
    WORD_SERVICE
        .get_or_init(|| async { WordService::new().await.unwrap() })
        .await
}
