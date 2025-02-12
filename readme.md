# Intro:

## how to run it:

just do `cargo run -- -c 100 --vvv`

### concurency:

blocks can be downloaded **concurently** (concurency being an argument that can be set with `-c`).

## penumbra grpc:
so there is the `Penumbra` struct in penumbra.rs, its purpose is to wrap around the grpc connection and make the various
penumbra requests, like :

- `get_penumbra_latest_block_height` which get the block height
- `get_block_n` which get the block n's content

## database:
then we have the `Db` trait in db.rs, the trait is an abstraction over various structs that can implement it.
that way the database could be changed easily, you just need to make a new struct for a new database type, like
sqlite for example and impl the trait for it, and the rest of the code will be able to use it.

here is the trait's definition which should be pretty self explanatory:

```rust
#[async_trait]
pub trait Db : Send + Sync {
    async fn get_last_block(&self) -> IndexerResult<usize>;
    async fn store_new_blocks(&self, blocks: &[Block]) -> IndexerResult<()>;
    async fn get_blocks(&self, nth: &[usize]) -> IndexerResult<Vec<Block>>;
}
```

## Penumbra indexer:
then there is the `PenumbraIndexer` in penumbra.rs, as its name suggest, its role is to index the chain, it is a wrapper
around `Penumbra` and a `Box<dyn Db>`, meaning it'll use `Penumbra` to get onchain data and use any struct that impl the `Db`
trait to save and query the data to the database, it could be anything, even ram.

the indexer has a `update_task` function that basically fetch the latest block height, download all the missing blocks from the latest synced
and save them to database.

the `auto_sync` function is then calling `update_task` in a loop, waiting 5 seconds after each time.

### Errors:

most function return an `IndexerResult<T>` which is a wrappper type for `Result<T, ErrorWrapper>`, errors are neatly wrapped within a local error type.

## Web server:

I'm using `axum`, the code in `web.rs` is pretty self explanatory.\
the function handler has access to an `Arc<PenumbraIndexer>` which allows it to query the db through it.\
it'll listen on port 8080, you can just do `curl localhost:8080 | jq` to get the last 10 blocks well formated,\
i recomend filtering the output though so you can do something like `curl localhost:8080  | jq '.[] | .nth , .block.header.time'`

## logger:

you can enable all loggin function using `-vvvv`, you can see how i manage it in `utils.rs`

# Good to know:

the file `descriptor.bin` is necessary to be able to conver the grpc responses to json using `prost_reflection`
you can do `cargo --bin=gen` to create it, it relies on the proto files from the penumbra github repo.
you need protobuf to be installed to be able to run the gen.

the database can be initialized using `cargo --bin=initdb` be sure to set the postgres_uri appropriately in .envrc and to then source the file before
running the command, be sure to provide a valid database string.

the program starts from the latest block on the chain, it would be a trivial change to make it start from the latest block in the db
or the first if none is found.

# Improvment suggestions:

because this is a technical test and not remunerated, some things have not been made, some of which i'd usually do for more serious projects.

- `PenumbraIndexer` could try to fetch and store a block when it is asked for a block that's not found in the db
- Retries upon failure to get a block
- `LayeredDB` in db.rs could be implemented, this would allow to pipeline database, for example redis and postgres, trying the faster one first.\
the rest of the code would not need to be changed.
- more error handling, and more verbose error reporting
- timeouts
- and generally you can just use `rg TODO` to find all the things that could improved i commented on
- database integrity checks
- adding more cli arguments, like bind address, update rate etc...
- automatic filling of holes in the database
- remove `#[allow(unused)]` in lib.rs and remove all unused warns.
- things could be cleaner, but again this is a technical test, not paid labor.

# cmd arguments

```
Usage: penumbra_indexer [OPTIONS]

Options:
  -n <NODE_ADDRES>               what grpc node to use [default: https://grpc.penumbra.silentvalidator.com]
  -v, --verbose...               verbosity
  -c, --concurency <CONCURENCY>  [default: 50]
  -h, --help                     Print help
```

# End:

if you have futher questions or want this readme to be updated futher let me know.
