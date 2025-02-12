use crate::errors::*;
use penumbra::util::tendermint_proxy::v1::{
    GetStatusRequest,
    GetBlockByHeightRequest,
    GetBlockByHeightResponse,
    tendermint_proxy_service_client::TendermintProxyServiceClient
};
use penumbra::Message;

pub trait Tjs {
    fn to_json(self) -> IndexerResult<serde_json::Value>;
}

static DESCRIPTOR_POOL : std::sync::OnceLock<prost_reflect::DescriptorPool> = std::sync::OnceLock::new();
fn get_descriptor_pool() -> prost_reflect::DescriptorPool {
    let pool = DESCRIPTOR_POOL.get_or_init(|| {
        prost_reflect::DescriptorPool::decode(
            include_bytes!("../descriptor.bin").as_slice()
            ).expect("couldn't load descriptor")
    }).clone();
    pool
}


impl Tjs for GetBlockByHeightResponse {
    fn to_json(self) -> IndexerResult<serde_json::Value> {
        let v = self.encode_to_vec();
        let pool = get_descriptor_pool();
        let des = pool.get_message_by_name("penumbra.util.tendermint_proxy.v1.GetBlockByHeightResponse").expect("couldn't get descriptorpool");

        let e = prost_reflect::DynamicMessage::decode(des, v.as_slice())?;
        let json = serde_json::to_value(e)?;
        Ok(json)
    }
}

pub struct Penumbra {
    tendermint_client : TendermintProxyServiceClient<tonic::transport::Channel>
}

impl Penumbra {
    pub async fn new(node: &str) -> Result<Self, tonic::transport::Error> {
        let client = TendermintProxyServiceClient::connect(node.to_owned()).await?;
        Ok(
            Self {
                tendermint_client: client
            }
        )
    }
    pub async fn get_penumbra_latest_block_height(&self) -> Result<Option<u64>, tonic::Status> {
        let mut client = self.tendermint_client.clone();
        let r = client.get_status(GetStatusRequest{})
            .await?.into_inner()
            .sync_info.map(|e| e.latest_block_height);
        Ok(r)
    }
    pub async fn get_block_n(&self, n: i64) -> Result<GetBlockByHeightResponse, tonic::Status> {
        let mut client = self.tendermint_client.clone();
        let e = client.get_block_by_height(GetBlockByHeightRequest {
            height: n
        }).await?.into_inner();
        Ok(e)
    }
}

// TODO add timeouts and retry
async fn get_block_json(n: usize, pen: &Penumbra) -> IndexerResult<serde_json::Value> {
    pen.get_block_n(n as i64).await?.to_json()
}

#[derive(Debug)]
pub struct BlockResult {
    nth: usize,
    r: IndexerResult<crate::db::Block>
}

// fetch_delta(range)
pub struct PenumbraIndexer {
    pen: std::sync::Arc<Penumbra>,
    current_block: tokio::sync::Mutex<usize>,
    db: Box<dyn crate::db::Db>,
    settings: PenumbraIndexerSettings
}

pub struct PenumbraIndexerSettings {
    pub node: String,
    pub concurency: usize
}

use futures::stream::StreamExt;
impl PenumbraIndexer {
    pub async fn new(db: Box<dyn crate::db::Db>, settings: PenumbraIndexerSettings) -> IndexerResult<Self> {
        let pen = Penumbra::new(&settings.node).await?;
        let current_block = pen.get_penumbra_latest_block_height().await?.expect("failed to get current_block") - 1; // TODO get from db
        let current_block = tokio::sync::Mutex::new(current_block as usize);
        let pen = std::sync::Arc::new(pen);
        Ok(
            Self {
                pen,
                current_block,
                db,
                settings
            }
        )
    }
    pub async fn fetch_delta(&self, range: std::ops::Range<usize>) -> Vec<BlockResult> {
        let stream = futures::stream::iter(range);
        self.fetch_stream(stream).await
    }

    pub async fn fetch_blocks_db(&self, blocks: Vec<usize>) -> IndexerResult<Vec<crate::db::Block>> {
        let success_blocks = self.db.get_blocks(&blocks).await?;
        let missing_blocks = crate::db::find_missing_blocks(&success_blocks, &blocks);
        // TODO fetch the missing blocks
        Ok(success_blocks)
    }

    pub async fn fetch_stream(&self, stream: impl futures::Stream<Item = usize> + Send + 'static) -> Vec<BlockResult> {
        let (tx,rx) = tokio::sync::mpsc::unbounded_channel::<BlockResult>();

        let pen = self.pen.clone();
        let concurency = self.settings.concurency;
        let task = tokio::spawn(async move {
            stream.for_each_concurrent(concurency, |n| {
                let pen = &pen;
                let tx = tx.clone();
                async move {
                    let b = get_block_json(n, pen).await;
                    match b {
                        Ok(_) => log::info!("downloaded block {}", n),
                        Err(_) => log::error!("failed to download block {}", n)
                    }
                    tx.send(BlockResult {
                        nth: n,
                        r: b.map(|b| crate::db::Block {
                            nth: n,
                            data: b
                        })
                    }).expect("failed to send to channel");
                }
            }).await;
        });
        // TODO it is possible to print them in order
        let blocks : Vec<_> = tokio_stream::wrappers::UnboundedReceiverStream::new(rx).collect().await;
        let _ = task.await;
        blocks
    }

    pub async fn update_task(&self) -> IndexerResult<()> {
        let mut current_block = self.current_block.lock().await;
        let current_height = self.pen.get_penumbra_latest_block_height().await?.expect("can't get block height") as usize;
        if current_height > *current_block {
            let range = *current_block+1..current_height+1;
            let new_blocks = self.fetch_delta(range).await;
            let success_blocks : Vec<_> = new_blocks.into_iter().filter_map(|b| {
                match b.r {
                    Ok(data) => Some(data),
                    Err(_) => {
                        // TODO handle failure
                        None
                    }
                }
            }).collect();
            self.db.store_new_blocks(&success_blocks).await?;
            *current_block = current_height;
        }
        Ok(())
    }

    pub async fn get_latest_block(&self) -> usize {
        *self.current_block.lock().await
    }

    pub async fn auto_sync(&self) {
        loop {
            let r = self.update_task().await;
            match r {
                Ok(_) => {},
                Err(e) => log::error!("{}", e)
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await; // update every 5 sec
        }
    }
}
