use owo_colors::OwoColorize;
use tokio::net::TcpListener;

use crate::{cli::VERBOSE, colorize::MColorize, httpfs::connection::handle_connection};

pub type UnrecoverableError = Box<dyn std::error::Error>;

pub async fn run_server(
    directory: &str,
    port: u16,
    verbosity: u8,
) -> Result<(), UnrecoverableError> {
    println!(
        "Starting Server: Serving directory {} on port {}",
        directory.out_color(|t| t.blue()),
        port.out_color(|t| t.green()),
    );

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

    loop {
        let (stream, addr) = listener.accept().await?;

        if verbosity >= VERBOSE {
            println!("Connection from {}", addr.out_color(|t| t.bright_yellow()));
        }

        let dir = directory.to_owned();

        tokio::spawn(async move {
            handle_connection(stream, &dir, verbosity).await;
        });
    }
}
