use penumbra_indexer::penumbra::*;
use penumbra_indexer::*;
use clap::Parser;
#[derive(Parser)]
struct Args {
    #[arg(short, default_value = "https://grpc.penumbra.silentvalidator.com")]
    /// what grpc node to use
    node_addres: String,

    /// verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(short, long, default_value = "50")]
    concurency: usize
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = Args::parse();
    utils::set_logging_verbose(args.verbose);

    let node = args.node_addres;
    let db : Box<dyn db::Db> = Box::new(db::DummyDb {});
    let penumbra_indexer = PenumbraIndexer::new(db, PenumbraIndexerSettings {
        node: node,
        concurency: args.concurency
    }).await?;
    log::info!("indexer initialized");

    let sync_task = tokio::spawn(async move {
        penumbra_indexer.auto_sync().await;
    });
    // penumbra_indexer.update_task().await?;
    // let pen = Penumbra::new(&node).await?;
    // let block = pen.get_penumbra_lattest_block_height().await?.unwrap();
    // let b = pen.get_block_n(block as i64).await?;
    // println!("{}", b.to_json()?);
    sync_task.await?;
    Ok(())
}
