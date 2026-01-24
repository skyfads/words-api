use dotenv::dotenv;

mod services {
    pub mod ai;
    pub mod word;
}

use services::word::init_word_service;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let ws = init_word_service().await;
    let entry = ws.get_detail("running").await;

    println!("Word: {}", entry.dictionary_form);
    println!("Language: {}", entry.language);
    println!("Definition: {}", entry.definition);
    println!("Example: {}", entry.sentence.example);
    println!("Sentence meaning: {}", entry.sentence.meaning);

    Ok(())
}
