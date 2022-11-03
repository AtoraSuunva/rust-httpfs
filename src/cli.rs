use clap::{Parser, ValueEnum, ValueHint};

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Color {
    Always,
    Auto,
    Never,
}

impl Color {
    pub fn init(self) {
        // Set a supports-color override based on the variable passed in.
        match self {
            Color::Always => owo_colors::set_override(true),
            Color::Auto => {}
            Color::Never => owo_colors::set_override(false),
        }
    }
}

// httpfs [-v] [-p PORT] [-d PATH-TO-DIR]

#[derive(Debug, Parser)]
#[clap(version, about)]
pub struct Cli {
    /// Should the output be in color?
    #[clap(long, value_enum, global = true, default_value = "auto")]
    pub color: Color,

    /// Verbosity of the output, -v = Prints the detail of the response such as protocol, status, and headers., -vv = and print request message
    #[clap(short, action = clap::ArgAction::Count)]
    pub verbosity: u8,

    /// Port to listen on, default is 8080
    #[clap(short, long, default_value_t = 8080)]
    pub port: u16,

    /// Path to the directory to serve, default is current working directory
    #[clap(short, long, default_value = ".", value_hint = ValueHint::DirPath)]
    pub dir: String,
}

pub const VERBOSE: u8 = 1;
pub const VERY_VERBOSE: u8 = 2;
