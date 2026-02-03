use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use bb8_postgres::tokio_postgres::Row;
use std::env;
use tokio::sync::OnceCell;
use tokio_postgres::NoTls;
use tokio_postgres::error::SqlState;

pub type PgPool = Pool<PostgresConnectionManager<NoTls>>;

pub static DB_POOL: OnceCell<PgPool> = OnceCell::const_new();

pub async fn init_db() -> &'static PgPool {
    DB_POOL
        .get_or_init(|| async {
            let uri = env::var("POSTGRES_URI").expect("POSTGRES_URI must be set");
            let config: tokio_postgres::Config = uri.parse().expect("Invalid POSTGRES_URI");
            let mgr = PostgresConnectionManager::new(config, NoTls);

            Pool::builder()
                .max_size(10)
                .build(mgr)
                .await
                .expect("Failed to create Postgres pool")
        })
        .await
}

pub async fn get_conn() -> bb8::PooledConnection<'static, PostgresConnectionManager<NoTls>> {
    init_db()
        .await
        .get()
        .await
        .expect("Failed to get DB connection")
}

pub async fn run_migrations() -> Result<(), tokio_postgres::Error> {
    let conn = get_conn().await;

    let sql = r#"
    CREATE TABLE IF NOT EXISTS language (
        id SERIAL PRIMARY KEY,
        name VARCHAR(50) NOT NULL UNIQUE
    );

    CREATE TABLE IF NOT EXISTS word (
        id SERIAL PRIMARY KEY,
        language_id INT NOT NULL REFERENCES language(id) ON DELETE RESTRICT,
        term VARCHAR(255) NOT NULL,
        definition TEXT NOT NULL,
        UNIQUE(language_id, term)
    );

    CREATE TABLE IF NOT EXISTS sentence (
        id SERIAL PRIMARY KEY,
        word_id INT NOT NULL REFERENCES word(id) ON DELETE CASCADE,
        example TEXT NOT NULL,
        meaning TEXT,
        UNIQUE(word_id, example)
    );
    "#;

    conn.batch_execute(sql).await?;
    Ok(())
}

pub async fn create_language(name: &str) -> Result<i32, tokio_postgres::Error> {
    let conn = get_conn().await;

    if let Some(row) = conn
        .query_opt("SELECT id FROM language WHERE name = $1", &[&name])
        .await?
    {
        return Ok(row.get("id"));
    }

    match conn
        .query_one(
            "INSERT INTO language(name) VALUES($1) RETURNING id",
            &[&name],
        )
        .await
    {
        Ok(row) => Ok(row.get("id")),
        Err(e) => {
            if e.code() == Some(&SqlState::UNIQUE_VIOLATION) {
                let row = conn
                    .query_one("SELECT id FROM language WHERE name = $1", &[&name])
                    .await?;
                Ok(row.get("id"))
            } else {
                Err(e)
            }
        }
    }
}

pub async fn create_word(
    language_id: i32,
    term: &str,
    definition: &str,
) -> Result<i32, tokio_postgres::Error> {
    let conn = get_conn().await;

    if let Some(row) = conn
        .query_opt(
            "SELECT id FROM word WHERE language_id = $1 AND term = $2",
            &[&language_id, &term],
        )
        .await?
    {
        return Ok(row.get("id"));
    }

    match conn
        .query_one(
            "INSERT INTO word(language_id, term, definition)
             VALUES($1, $2, $3)
             RETURNING id",
            &[&language_id, &term, &definition],
        )
        .await
    {
        Ok(row) => Ok(row.get("id")),

        Err(e) => {
            if e.code() == Some(&tokio_postgres::error::SqlState::UNIQUE_VIOLATION) {
                let row = conn
                    .query_one(
                        "SELECT id FROM word WHERE language_id = $1 AND term = $2",
                        &[&language_id, &term],
                    )
                    .await?;
                Ok(row.get("id"))
            } else {
                Err(e)
            }
        }
    }
}

pub async fn get_word(
    language: &str,
    term: &str,
) -> Result<Option<(i32, String, String)>, tokio_postgres::Error> {
    let conn = get_conn().await;

    let row_opt = conn
        .query_opt(
            r#"
            SELECT w.id, w.term, w.definition
            FROM word w
            JOIN language l ON w.language_id = l.id
            WHERE l.name = $1 AND w.term = $2
            "#,
            &[&language, &term],
        )
        .await?;

    Ok(row_opt.map(|row| {
        (
            row.get::<_, i32>("id"),
            row.get::<_, String>("term"),
            row.get::<_, String>("definition"),
        )
    }))
}

pub async fn get_all_words(
    limit: i64,
    offset: i64,
) -> Result<Vec<(i32, String, String, String)>, tokio_postgres::Error> {
    let conn = get_conn().await;
    let rows = conn
        .query(
            r#"
            SELECT w.id, l.name as language, w.term, w.definition 
            FROM word w
            JOIN language l ON w.language_id = l.id
            ORDER BY w.id ASC
            LIMIT $1 OFFSET $2
            "#,
            &[&limit, &offset],
        )
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.get("id"),
                row.get("language"),
                row.get("term"),
                row.get("definition"),
            )
        })
        .collect())
}

pub async fn delete_word(id: i32) -> Result<(), tokio_postgres::Error> {
    let conn = get_conn().await;
    conn.execute("DELETE FROM word WHERE id = $1", &[&id])
        .await?;
    Ok(())
}

pub async fn create_sentence(
    word_id: i32,
    example: &str,
    meaning: Option<&str>,
) -> Result<i32, tokio_postgres::Error> {
    let conn = get_conn().await;

    if let Some(row) = conn
        .query_opt(
            "SELECT id FROM sentence WHERE word_id = $1 AND example = $2",
            &[&word_id, &example],
        )
        .await?
    {
        return Ok(row.get("id"));
    }

    match conn
        .query_one(
            "INSERT INTO sentence(word_id, example, meaning)
             VALUES($1, $2, $3)
             RETURNING id",
            &[&word_id, &example, &meaning],
        )
        .await
    {
        Ok(row) => Ok(row.get("id")),

        Err(e) => {
            if e.code() == Some(&tokio_postgres::error::SqlState::UNIQUE_VIOLATION) {
                let row = conn
                    .query_one(
                        "SELECT id FROM sentence WHERE word_id = $1 AND example = $2",
                        &[&word_id, &example],
                    )
                    .await?;
                Ok(row.get("id"))
            } else {
                Err(e)
            }
        }
    }
}

pub async fn get_sentences_by_word(
    word_id: i32,
) -> Result<Vec<(i32, String, Option<String>)>, tokio_postgres::Error> {
    let conn = get_conn().await;
    let rows: Vec<Row> = conn
        .query(
            "SELECT id, example, meaning FROM sentence WHERE word_id = $1",
            &[&word_id],
        )
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.get("id"), row.get("example"), row.get("meaning")))
        .collect())
}

pub async fn get_sentences_by_word_ids(
    word_ids: &[i32],
) -> Result<Vec<(i32, i32, String, Option<String>)>, tokio_postgres::Error> {
    let conn = get_conn().await;
    let rows = conn
        .query(
            "SELECT id, word_id, example, meaning FROM sentence WHERE word_id = ANY($1)",
            &[&word_ids],
        )
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.get("id"),
                row.get("word_id"),
                row.get("example"),
                row.get("meaning"),
            )
        })
        .collect())
}
