use async_trait::async_trait;
use crate::errors::IndexerResult;

#[allow(unused)]
pub struct Block {
    nth: usize,
    data: serde_json::Value,
}


#[async_trait]
pub trait Db {
    async fn get_last_block(&self) -> IndexerResult<usize>;
    async fn store_new_block(&self, block: Block) -> IndexerResult<()>;
    async fn get_block(&self, nth: usize) -> IndexerResult<Block>;
}


pub struct DummyDb {
}

pub struct PostgresDB {
    db: sqlx::postgres::PgPool
}
