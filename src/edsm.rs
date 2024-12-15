use color_eyre::eyre::Result;
use edsm_dumps_model::model::{
    station::Station, system::SystemWithCoordinates, system_populated::SystemPopulated,
};
use futures::StreamExt;
use indicatif::ProgressIterator;
use log::info;
use log::warn;
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
};

// Not all stations in the document are valid, so we need to skip errors
// Reference: https://stackoverflow.com/a/73654367/5007892

async fn read_systems(systems_json_path: std::path::PathBuf, pool: &Pool<Postgres>) -> Result<()> {
    info!("Parsing systems JSON");
    let system_file = File::open(systems_json_path)?;
    let system_reader = BufReader::new(system_file);
    let systems: Vec<SystemPopulated> = serde_json::from_reader(system_reader)?;
    info!("Parsed {} systems", systems.len());

    let chunks = systems.chunks(2048);
    info!("Will insert {} chunks", chunks.len());

    info!("Inserting into PostgreSQL");
    // we batch into chunks, so that we don't lose all our progress if we CTRL+C it
    for chunk in chunks.progress() {
        let transaction = pool.begin().await?;

        futures::stream::iter(chunk)
            .for_each(|system| async move {
                let _ = sqlx::query!(
                    r#"
                        INSERT INTO systems (
                            id, name, date, coords
                        ) VALUES (
                            $1, $2, $3, ST_MakePoint($4, $5, $6)
                        );
                        "#,
                    system.id as i64,
                    system.name,
                    system.date.naive_utc(),
                    system.coords.x as f64,
                    system.coords.y as f64,
                    system.coords.z as f64
                )
                .execute(pool)
                .await;
            })
            .await;

        transaction.commit().await?;
    }

    Ok(())
}

async fn read_stations(
    stations_json_path: std::path::PathBuf,
    pool: &Pool<Postgres>,
) -> Result<()> {
    info!("Parsing stations JSON");
    let stations_file = File::open(stations_json_path)?;
    let stations_reader = BufReader::new(stations_file);
    let stations: Vec<Station> = serde_json::from_reader(stations_reader)?;
    info!("Parsed {} stations", stations.len());

    let chunks = stations.chunks(2048);
    info!("Will insert {} chunks", chunks.len());

    info!("Inserting into PostgreSQL");
    // we batch into chunks, so that we don't lose all our progress if we CTRL+C it
    for chunk in chunks.progress() {
        let transaction = pool.begin().await?;

        futures::stream::iter(chunk)
            .for_each(|station| async move {
                let result = sqlx::query!(
                    r#"
                    INSERT INTO stations (
                        id, distance_to_arrival, name, market_id, system_id, landing_pad
                    ) VALUES (
                        $1, $2, $3, $4, $5, NULL
                    );
                "#,
                    station.id as i64,
                    station.distance_to_arrival,
                    station.name,
                    station.market_id.map(|v| v as i64),
                    station.system_id.map(|v| v as i64),
                )
                .execute(pool)
                .await;

                // match result {
                //     Ok(_) => {
                //         info!("Succeeded to insert id {} name {}", station.id, station.name);
                //     }
                //     Err(error) => {
                //         // these errors are mostly caused by player carriers, which we will just
                //         // ignore (we won't use them for trading anyway)
                //         if !error.to_string().contains("duplicate") {
                //             warn!("Failed to insert id {} name {}: {}", station.id, station.name, error);
                //         }
                //     }
                // }
            })
            .await;

        transaction.commit().await?;
    }

    Ok(())
}

pub async fn ingest_edsm(
    systems_json_path: std::path::PathBuf,
    stations_json_path: std::path::PathBuf,
) -> Result<()> {
    info!("Setting up PostgreSQL pool");
    let var_name = PgPoolOptions::new();
    let pool = var_name
        .max_connections(64)
        .connect("postgres://postgres:password@localhost/edtear")
        .await?;

    read_systems(systems_json_path, &pool).await?;
    read_stations(stations_json_path, &pool).await?;

    return Ok(());
}
