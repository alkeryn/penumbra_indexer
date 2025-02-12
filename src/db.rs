use async_trait::async_trait;
use crate::errors::IndexerResult;

#[derive(Default)]
pub struct Block {
    pub nth: usize,
    pub data: serde_json::Value,
}


#[async_trait]
pub trait Db {
    async fn get_last_block(&self) -> IndexerResult<usize>;
    async fn store_new_blocks(&self, block: &[Block]) -> IndexerResult<()>;
    async fn get_blocks(&self, nth: &[usize]) -> IndexerResult<Block>;
}


pub struct DummyDb {}

pub struct PostgresDB {
    db: sqlx::postgres::PgPool
}

impl PostgresDB {
    pub async fn new(address: &str) -> IndexerResult<Self> {
        let db = sqlx::postgres::PgPool::connect(address).await?;
        Ok(
            Self {
                db
            }
        )
    }
}

#[async_trait]
impl Db for PostgresDB {
    async fn get_last_block(&self) -> IndexerResult<usize> {
        Ok(0)
    }
    async fn store_new_blocks(&self, block: &[Block]) -> IndexerResult<()> {
        Ok(())
    }
    async fn get_blocks(&self, nth: &[usize]) -> IndexerResult<Block> {
        Ok(Block::default())
    }
}

#[async_trait]
impl Db for DummyDb {
    async fn get_last_block(&self) -> IndexerResult<usize> {
        Ok(0)
    }
    async fn store_new_blocks(&self, block: &[Block]) -> IndexerResult<()> {
        Ok(())
    }
    async fn get_blocks(&self, nth: &[usize]) -> IndexerResult<Block> {
        Ok(Block::default())
    }
}


// this could be used in the future to layer db's for faster caching, ie ram <> redis <> postgres
pub struct LayeredDB {
    databases: Vec<Box<dyn Db>>
}
