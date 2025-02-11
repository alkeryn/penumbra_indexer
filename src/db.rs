use async_trait::async_trait;

#[allow(unused)]
pub struct Block {
    nth: usize,
    data: serde_json::Value,
}
#[async_trait]
pub trait Db {
    async fn get_last_block() -> usize;
    async fn store_new_block(&self, block: Block);
    async fn get_block(&self, nth: usize);
}


pub struct DummyDb {
}
