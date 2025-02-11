use async_trait::async_trait;
use crate::errors::BoxRes;

#[allow(unused)]
pub struct Block {
    nth: usize,
    data: serde_json::Value,
}

#[async_trait]
pub trait Db {
    async fn get_last_block(&self) -> BoxRes<usize>;
    async fn store_new_block(&self, block: Block) -> BoxRes<()>;
    async fn get_block(&self, nth: usize) -> BoxRes<Block>;
}


pub struct DummyDb {
}
