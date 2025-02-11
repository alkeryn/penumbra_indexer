use tech_test::penumbra::*;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let node = "https://grpc.penumbra.silentvalidator.com";
    let pen = Penumbra::new(node).await?;
    let block = pen.get_penumbra_lattest_block_height().await?.unwrap();
    println!("{}", block);
    let b = pen.get_block_n(block as i64).await?;
    println!("{}", b.to_json()?);
    Ok(())
}
