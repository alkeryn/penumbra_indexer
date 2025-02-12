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
async fn main() -> errors::IndexerResult<()>{
    let args = Args::parse();
    utils::set_logging_verbose(args.verbose);

    let node = args.node_addres;
    let postgres_uri = std::env::var("POSTGRES_URI")?;
    let db : Box<dyn db::Db> = Box::new(db::PostgresDB::new(&postgres_uri).await?);
    let penumbra_indexer = PenumbraIndexer::new(db, PenumbraIndexerSettings {
        node,
        concurency: args.concurency
    }).await?;
    log::info!("indexer initialized");

    let penumbra_indexer = std::sync::Arc::new(penumbra_indexer); // useful to query it later

    let pen = penumbra_indexer.clone();
    let sync_task = tokio::spawn(async move {
        pen.auto_sync().await;
    });

    crate::web::start_axum(penumbra_indexer).await?;
    sync_task.await?;
    Ok(())
}
