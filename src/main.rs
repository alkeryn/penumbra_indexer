use tech_test::penumbra::*;
use clap::Parser;
#[derive(Parser)]
struct Args {
    #[arg(short, default_value = "https://grpc.penumbra.silentvalidator.com")]
    node_addres: String
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = Args::parse();
    let node = args.node_addres;
    let pen = Penumbra::new(&node).await?;
    let block = pen.get_penumbra_lattest_block_height().await?.unwrap();
    println!("{}", block);
    let b = pen.get_block_n(block as i64).await?;
    println!("{}", b.to_json()?);
    Ok(())
}
