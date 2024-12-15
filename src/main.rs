use clap::{Parser, Subcommand};
use log::info;
use color_eyre::eyre::Result;
pub mod edsm;
pub mod eddn;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "edtear")]
#[command(
    about = format!("EDTear v{VERSION}: The Elite: Dangerous Trade Ear"),
)]
struct EdTearCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Ingest EDSM nightly dumps
    #[command(arg_required_else_help = true)]
    IngestEdsm {
        #[arg(long)]
        /// Path to populated systems JSON from EDSM
        systems_json_path: std::path::PathBuf,
        #[arg(long)]
        /// Path to stations JSON from EDSM
        stations_json_path: std::path::PathBuf,
    },

    /// Listens to EDDN to receive updated commodity data
    Listen {
    },

    /// Prints version information.
    #[command()]
    Version {},
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = EdTearCli::parse();
    env_logger::init();
    color_eyre::install()?;

    match args.command {
        Commands::Version {} => {
            println!("EDTear v{VERSION}, the Elite: Dangerous Trade Ear");
            println!("Copyright (c) 2025 Matt Young. ISC Licence.");
            Ok(())
        }
        Commands::IngestEdsm { systems_json_path, stations_json_path } => {
            info!("Importing EDSM dump");
            edsm::ingest_edsm(systems_json_path, stations_json_path).await?;
            Ok(())
        },
        Commands::Listen {  } => {
            info!("Listening to EDDN");
            eddn::listen().await?;
            Ok(())
        },

    }
}
