use penumbra::util::tendermint_proxy::v1::{
    GetStatusRequest,
    GetBlockByHeightRequest,
    GetBlockByHeightResponse,
    tendermint_proxy_service_client::TendermintProxyServiceClient
};
use penumbra::Message;

pub trait Tjs {
    fn to_json(self) -> BoxRes<serde_json::Value>;
}
fn get_descriptor_pool() -> BoxRes<prost_reflect::DescriptorPool> {
    let pool = prost_reflect::DescriptorPool::decode(include_bytes!("../descriptor.bin").as_slice())?;
    Ok(pool)
}

impl Tjs for GetBlockByHeightResponse {
    fn to_json(self) -> BoxRes<serde_json::Value> {
        let v = self.encode_to_vec();
        let pool = get_descriptor_pool()?;
        let des = pool.get_message_by_name("penumbra.util.tendermint_proxy.v1.GetBlockByHeightResponse").expect("couldn't get descriptor");

        let e = prost_reflect::DynamicMessage::decode(des, v.as_slice())?;
        let json = serde_json::to_value(e)?;
        Ok(json)
    }
}

pub struct Penumbra {
    tendermint_client : TendermintProxyServiceClient<tonic::transport::Channel>
}
type BoxRes<T> = Result<T, Box<dyn std::error::Error>>;
impl Penumbra {
    pub async fn new(node: &str) -> BoxRes<Self> {
        let client = TendermintProxyServiceClient::connect(node.to_owned()).await?;
        Ok(
            Self {
                tendermint_client: client
            }
        )
    }
    pub async fn get_penumbra_lattest_block_height(&self) -> BoxRes<Option<u64>> {
        let mut client = self.tendermint_client.clone();
        let r = client.get_status(GetStatusRequest{})
            .await?.into_inner()
            .sync_info.map(|e| e.latest_block_height);
        Ok(r)
    }
    pub async fn get_block_n(&self, n: i64) -> BoxRes<GetBlockByHeightResponse> {
        let mut client = self.tendermint_client.clone();
        let e = client.get_block_by_height(GetBlockByHeightRequest {
            height: n
        }).await?.into_inner();
        Ok(e)
    }
}
