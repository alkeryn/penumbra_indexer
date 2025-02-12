async fn execute_script(script: &str, pg: &sqlx::PgPool) -> penumbra_indexer::errors::IndexerResult<()> {
    let v : Vec<_> = script.split_terminator(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    for query in v {
        sqlx::query(query).execute(pg).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> penumbra_indexer::errors::IndexerResult<()> {
    let postgres_uri = std::env::var("POSTGRES_URI")?;
    let pg = sqlx::postgres::PgPool::connect(&postgres_uri).await?;
    execute_script(include_str!("../../db/postgres.sql"), &pg).await?;
    Ok(())
}
