use crate::cli::Cli;
use crate::http_server::run_server;
use clap::Parser;

mod cli;
mod colorize;
mod http_connection;
mod http_message;
mod http_parse;
mod http_parse_error;
mod http_server;

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    args.color.init();

    let res = run_server(&args.dir, args.port, args.verbosity).await;

    if res.is_err() {
        // oh no
        eprintln!("{}", res.unwrap_err());
        std::process::exit(1);
    }
}
