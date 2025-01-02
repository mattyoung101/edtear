// Imports Spansh's data dumps (https://spansh.co.uk/dumps) to determine station landing pads
// Specifically: https://downloads.spansh.co.uk/galaxy_stations.json.gz

use color_eyre::eyre::Result;
use indicatif::ProgressIterator;
use log::{info, warn};
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, Acquire, Pool, Postgres};
use std::{fs::File, io::BufReader};

// This is a very condensed schema to only process the landing pad data

#[derive(Deserialize, Debug)]
struct SpanshLandingPads {
    large: i32,
    medium: i32,
    small: i32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SpanshStation {
    name: String,
    landing_pads: Option<SpanshLandingPads>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SpanshSystem {
    name: String,
    stations: Vec<SpanshStation>,
}

pub async fn read_spansh_stations(
    galaxy_stations_json_path: std::path::PathBuf,
    url: String,
) -> Result<()> {
    info!("Setting up PostgreSQL pool");
    let var_name = PgPoolOptions::new();
    let pool = var_name.max_connections(64).connect(&url).await?;

    info!("Parsing Spansh dump");
    let galaxy_file = File::open(galaxy_stations_json_path)?;
    let galaxy_reader = BufReader::new(galaxy_file);
    let systems: Vec<SpanshSystem> = serde_json::from_reader(galaxy_reader)?;
    info!("Parsed {} systems", systems.len());

    // let trans = pool.begin().await?;

    for system in systems.iter().progress() {
        for station in &system.stations {
            if let Some(landing_pads) = &station.landing_pads {
                let mut insert = String::new();
                if landing_pads.small > 0 {
                    insert += "s,"; // small
                }
                if landing_pads.medium > 0 {
                    insert += "m,"; // medium
                }
                if landing_pads.large > 0 {
                    insert += "l"; // large
                }

                let system_id_result =
                    sqlx::query!("SELECT id FROM systems WHERE name = $1;", system.name)
                        .fetch_one(&pool)
                        .await;

                match system_id_result {
                    Ok(system_id) => {
                        let result = sqlx::query!(
                            r#"
                            UPDATE stations SET landing_pad = $1
                            WHERE name = $2 AND system_id = $3;
                        "#,
                            insert,
                            station.name,
                            system_id.id
                        )
                        .execute(&pool)
                        .await;
                        if result.is_err() {
                            warn!(
                                "Failed to insert station {} in {}: {}",
                                station.name,
                                system.name,
                                result.unwrap_err()
                            );
                        }
                    }
                    Err(error) => {
                        // warn!(
                        //     "Unable to locate ID for system name {}: {}",
                        //     system.name, error
                        // );
                        continue;
                    }
                }
            }
        }
    }

    // trans.commit().await?;

    Ok(())
}
