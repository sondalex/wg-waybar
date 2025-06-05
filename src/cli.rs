use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to the wireguard configuration file
    pub config: String,
    /// Signal to use
    #[arg(long, default_value_t = 9)]
    pub signal: i32,

    /// Enable debug output
    #[arg(short, long)]
    pub debug: bool,

    /// State filename
    #[arg(long, default_value="status.json")]
    pub state_filename: String, 

    /// Port for wireguard connection
    #[arg(long, default_value_t = 40077)]
    pub port: u32,


    #[command(subcommand)]
    pub command: Option<Commands>,

    }

#[derive(Subcommand)]
pub enum Commands {
    /// Toggle the vpn (switch state)
    Toggle,
}
