use dotenv::dotenv;

mod services {
    pub mod ai;
    pub mod db;
    pub mod word;
}

use services::db;
use services::word::init_word_service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    db::init_db().await;
    db::run_migrations()
        .await
        .expect("Failed to run migrations");

    let ws = init_word_service().await;
    let entry = ws.get_detail("running").await;

    println!("Word: {}", entry.dictionary_form);
    println!("Language: {}", entry.language);
    println!("Definition: {}", entry.definition);
    println!("Example: {}", entry.sentence.example);
    println!("Sentence meaning: {}", entry.sentence.meaning);

    let lang_id = db::create_language(&entry.language).await.unwrap();
    let exist = db::get_word(&entry.language, &entry.dictionary_form).await?;
    if exist.is_some() {
        println!(
            "Word '{}' already exists for language '{}'",
            entry.dictionary_form, entry.language
        );
        std::process::exit(0);
    }

    let word_id = db::create_word(lang_id, &entry.dictionary_form, &entry.definition)
        .await
        .unwrap();

    let _sentence_id = db::create_sentence(
        word_id,
        &entry.sentence.example,
        Some(&entry.sentence.meaning),
    )
    .await
    .unwrap();

    println!("Entry saved to the database!");

    Ok(())
}
