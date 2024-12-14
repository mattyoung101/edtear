use color_eyre::eyre::Result;
use edsm_dumps_model::model::{station::Station, system::SystemWithCoordinates, system_populated::SystemPopulated};
use futures::StreamExt;
use indicatif::ProgressIterator;
use log::{info};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
};

async fn read_systems(systems_json_path: std::path::PathBuf, pool: &Pool<Postgres>) -> Result<()> {
    info!("Parsing systems JSON");
    let system_file = File::open(systems_json_path)?;
    let system_reader = BufReader::new(system_file);
    let systems: Vec<SystemPopulated> = serde_json::from_reader(system_reader)?;
    info!("Parsed {} systems", systems.len());

    // info!("Dumping COPY files to /var/tmp/systems.csv");
    // let out = File::create("/var/tmp/systems.csv").unwrap();
    // let mut buf_out = BufWriter::new(out);
    // for system in &systems {
    //     buf_out.write(
    //         format!(
    //             "{}|{}|{}|POINT({} {} {})\n",
    //             system.id as i64,
    //             system.name,
    //             system.date.naive_utc(),
    //             system.coords.x as f64,
    //             system.coords.y as f64,
    //             system.coords.z as f64
    //         ).as_bytes()
    //     )?;
    // }
    // buf_out.flush()?;

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
