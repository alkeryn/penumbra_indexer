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
    pub async fn get_penumbra_lattest_block_height(&self) -> Result<Option<u64>, tonic::Status> {
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

// fetch_delta(range)
pub struct PenumbraIndexer {
    pen: std::sync::Arc<Penumbra>,
    current_block: std::sync::atomic::AtomicUsize,
    db: Box<dyn crate::db::Db>
}

async fn get_block_json(n: usize, pen: &Penumbra) -> IndexerResult<serde_json::Value> {
    pen.get_block_n(n as i64).await?.to_json()
}

struct BlockResult {
    nth: usize,
    r: IndexerResult<serde_json::Value>
}

use futures::stream::StreamExt;
impl PenumbraIndexer {
    async fn new(node: &str, db: Box<dyn crate::db::Db>) -> IndexerResult<Self> {
        let pen = Penumbra::new(node).await?;
        let current_block = pen.get_penumbra_lattest_block_height().await?.expect("failed to get current_block");
        let current_block = std::sync::atomic::AtomicUsize::new(current_block as usize); // TODO get from db
        let pen = std::sync::Arc::new(pen);
        Ok(
            Self {
                pen,
                current_block,
                db
            }
        )
    }
    async fn fetch_delta(&self, range: std::ops::Range<usize>) -> Vec<BlockResult> {
        let stream = futures::stream::iter(range);
        let (tx,rx) = tokio::sync::mpsc::unbounded_channel::<BlockResult>();

        let pen = self.pen.clone();
        let task = tokio::spawn(async move {
                stream.for_each_concurrent(50, |n| {
                    let pen = &pen;
                    let tx = tx.clone();
                    async move {
                        let b = get_block_json(n, pen).await;
                        tx.send(BlockResult {
                            nth: n,
                            r: b
                        });
                    }
                }).await;
        });
        let blocks : Vec<_> = tokio_stream::wrappers::UnboundedReceiverStream::new(rx).collect().await;
        blocks
    }
}
