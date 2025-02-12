pub fn set_logging_verbose(verbose: u8) {
    match verbose { // TODO don't set it if it has been manually set externally
        0 => std::env::set_var("RUST_LOG", "error"),
        1 => std::env::set_var("RUST_LOG", "warn"),
        2 => std::env::set_var("RUST_LOG", "info"),
        3 =>  {
            std::env::set_var("RUST_LOG", "penumbra_indexer=debug");
        },
        _ => {
            std::env::set_var("RUST_LOG", "penumbra_indexer=debug");
            std::env::set_var("RUST_BACKTRACE", "1");
        }
    }
    env_logger::init();
}
