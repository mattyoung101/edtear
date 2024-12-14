use color_eyre::eyre::{Ok, Result};
use edsm_dumps_model::model::{station::Station, system::SystemWithCoordinates};
use futures::StreamExt;
use log::info;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;
use std::{fs::File, io::BufReader};

async fn read_systems(systems_json_path: std::path::PathBuf, pool: &Pool<Postgres>) -> Result<()> {
    info!("Parsing systems JSON");
    let system_file = File::open(systems_json_path)?;
    let system_reader = BufReader::new(system_file);
    let systems: Vec<SystemWithCoordinates> = serde_json::from_reader(system_reader)?;
    info!("Parsed {} systems", systems.len());

    info!("Inserting into PostgreSQL");
    let transaction = pool.begin().await?;
    let progress = Arc::new(indicatif::ProgressBar::new(
        systems.len().try_into().unwrap(),
    ));
    futures::stream::iter(systems)
        .for_each(|system| {
            let progress = Arc::clone(&progress); // Share the same progress bar across tasks
            async move {
                sqlx::query!(
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
                .await
                .unwrap();
                progress.inc(1);
            }
        })
        .await;
    progress.finish();

    info!("Committing transaction");
    transaction.commit().await?;

    Ok(())
}

async fn read_stations(
    stations_json_path: std::path::PathBuf,
    pool: &Pool<Postgres>,
) -> Result<()> {
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
