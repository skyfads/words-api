use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use std::env;
use tokio::sync::OnceCell;
use tokio_postgres::NoTls;

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
        meaning TEXT
    );
    "#;

    conn.batch_execute(sql).await?;
    Ok(())
}

pub async fn create_language(name: &str) -> Result<i32, tokio_postgres::Error> {
    let conn = get_conn().await;
    let row = conn
        .query_one(
            "INSERT INTO language(name) VALUES($1) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id",
            &[&name],
        )
        .await?;
    Ok(row.get("id"))
}

pub async fn create_word(
    language_id: i32,
    term: &str,
    definition: &str,
) -> Result<i32, tokio_postgres::Error> {
    let conn = get_conn().await;
    let row = conn
        .query_one(
            "INSERT INTO word(language_id, term, definition) VALUES($1, $2, $3) ON CONFLICT(language_id, term) DO UPDATE SET definition = EXCLUDED.definition RETURNING id",
            &[&language_id, &term, &definition],
        )
        .await?;
    Ok(row.get("id"))
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

pub async fn create_sentence(
    word_id: i32,
    example: &str,
    meaning: Option<&str>,
) -> Result<i32, tokio_postgres::Error> {
    let conn = get_conn().await;
    let row = conn
        .query_one(
            "INSERT INTO sentence(word_id, example, meaning) VALUES($1, $2, $3) RETURNING id",
            &[&word_id, &example, &meaning],
        )
        .await?;
    Ok(row.get("id"))
}
