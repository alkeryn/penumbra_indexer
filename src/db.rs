use async_trait::async_trait;
use crate::errors::IndexerResult;

#[derive(Default, Debug)]
pub struct Block {
    pub nth: usize,
    pub data: serde_json::Value,
}


#[async_trait]
pub trait Db : Send + Sync {
    async fn get_last_block(&self) -> IndexerResult<usize>;
    async fn store_new_blocks(&self, blocks: &[Block]) -> IndexerResult<()>;
    async fn get_blocks(&self, nth: &[usize]) -> IndexerResult<Vec<Block>>;
}

pub fn find_missing_blocks(blocks: Vec<crate::db::Block>, all: &[usize]) -> Vec<usize> {
    let success_blocks : std::collections::HashSet<usize> = blocks.iter().map(|e| e.nth).collect();
    let all_blocks : std::collections::HashSet<usize> = all.iter().map(|i| *i).collect();

    all_blocks.difference(&success_blocks)
        .into_iter()
        .map(|e| *e)
        .collect()
}

pub struct DummyDb {}

pub struct PostgresDB {
    pg: sqlx::postgres::PgPool
}

impl PostgresDB {
    pub async fn new(address: &str) -> IndexerResult<Self> {
        let pg = sqlx::postgres::PgPool::connect(address).await?;
        Ok(
            Self {
                pg
            }
        )
    }
}

use sqlx::Row;
#[async_trait]
impl Db for PostgresDB {
    async fn get_last_block(&self) -> IndexerResult<usize> {
        let id : i64 = sqlx::query("SELECT id FROM blocks ORDER BY id DESC LIMIT 1")
            .fetch_one(&self.pg)
            .await?.get("id");
        Ok(id as usize)
    }
    async fn store_new_blocks(&self, blocks: &[Block]) -> IndexerResult<()> {
        let mut query = sqlx::QueryBuilder::new(
            "INSERT INTO blocks (id, data) "
        );
        query.push_values(blocks, |mut q, block | {
            q.push_bind(block.nth as i64);
            q.push_bind(&block.data);
        });

        query.push(" ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data");
        let query = query.build().execute(&self.pg).await?;
        Ok(())
    }
    async fn get_blocks(&self, nth: &[usize]) -> IndexerResult<Vec<Block>> {
        let mut query = sqlx::QueryBuilder::new(
            "SELECT * from blocks where id in "
        );
        query.push_tuples(nth, |mut b, id| {
            b.push_bind(*id as i64);
        });
        let r : Vec<_> = query
            .build_query_as::<(i64, serde_json::Value)>()
            .fetch_all(&self.pg)
            .await?.iter().map(|e| {
                Block {
                    nth: e.0 as usize,
                    data: e.1.clone()
                }
            }).collect();
        Ok(r)
    }
}

#[async_trait]
impl Db for DummyDb {
    async fn get_last_block(&self) -> IndexerResult<usize> {
        Ok(0)
    }
    async fn store_new_blocks(&self, blocks: &[Block]) -> IndexerResult<()> {
        Ok(())
    }
    async fn get_blocks(&self, nth: &[usize]) -> IndexerResult<Vec<Block>> {
        Ok(Vec::default())
    }
}


// this could be used in the future to layer db's for faster caching, ie ram <> redis <> postgres
pub struct LayeredDB {
    databases: Vec<Box<dyn Db>>
}
