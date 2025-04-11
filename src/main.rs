use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use log::info;
pub mod eddn;
pub mod edsm;
pub mod spansh;
pub mod stats;

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
        #[arg(long, env = "DATABASE_URL")]
        /// Postgres connection URL. Recommended: postgres://postgres:password@localhost/edtear
        url: String,
    },

    /// Ingests Spansh's nightly dumps to insert landing pad sizes. Must be run after ingest-edsm.
    #[command(arg_required_else_help = true)]
    IngestSpansh {
        /// Path to galaxy stations JSON from Spansh's website
        #[arg(long)]
        galaxy_stations_json_path: std::path::PathBuf,
        #[arg(long, env = "DATABASE_URL")]
        /// Postgres connection URL. Recommended: postgres://postgres:password@localhost/edtear
        url: String,
    },

    /// Listens to EDDN to receive updated commodity data
    Listen {
        #[arg(long, env = "DATABASE_URL")]
        /// Postgres connection URL. Recommended: postgres://postgres:password@localhost/edtear
        url: String,
    },

    /// Displays statistics for a connected database
    Stats {
        #[arg(long, env = "DATABASE_URL")]
        /// Postgres connection URL. Recommended: postgres://postgres:password@localhost/edtear
        url: String,
    },

    /// Prints version information.
    #[command()]
    Version {},
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv_override()?;
    let args = EdTearCli::parse();
    env_logger::init();
    color_eyre::install()?;

    match args.command {
        Commands::Version {} => {
            println!("EDTear v{VERSION}, the Elite: Dangerous Trade Ear");
            println!("Copyright (c) 2024 Matt Young. ISC Licence.");
            Ok(())
        }
        Commands::IngestEdsm {
            systems_json_path,
            stations_json_path,
            url,
        } => {
            info!("Importing EDSM dump");
            edsm::ingest_edsm(systems_json_path, stations_json_path, url).await?;
            Ok(())
        }
        Commands::Listen { url } => {
            info!("Listening to EDDN");
            eddn::listen(url).await?;
            Ok(())
        }
        Commands::Stats { url } => {
            stats::display_stats(url).await?;
            Ok(())
        }
        Commands::IngestSpansh {
            galaxy_stations_json_path,
            url,
        } => {
            info!("Importing Spansh dump");
            spansh::read_spansh_stations(galaxy_stations_json_path, url).await?;
            Ok(())
        },
    }
}
