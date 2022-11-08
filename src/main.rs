use crate::cli::Cli;
use crate::httpfs::server::run_server;
use clap::Parser;

mod cli;
mod colorize;
mod filesystem;
mod httpfs;

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    args.color.init();

    // The server runs in a loop unless it hits a completely unrecoverable error.
    // Like "we literally can't serve another client" level bad
    let res = run_server(&args.dir, args.port, args.verbosity).await;

    if res.is_err() {
        // oh no
        eprintln!("{}", res.unwrap_err());
        std::process::exit(1);
    }
}
