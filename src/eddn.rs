use std::{
    process::exit, sync::{atomic::AtomicU32, Arc, Mutex}, thread, time::Duration
};

use color_eyre::eyre::Result;
use lazy_static::lazy_static;
use log::{error, info, warn};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tokio::sync::watch;

// Ingests data from the Elite: Dangerous Data Network

lazy_static! {
    /// Keeps track of the number of items added in the last hour
    static ref WATCHDOG_ADDED: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
}

/// Returns whether or not the given market ID has listings in the last 1 hour
async fn has_listings_1h(market_id: i64, pool: &Pool<Postgres>) -> bool {
    let result = sqlx::query!(
        r#"
        SELECT * FROM listings
        WHERE market_id = $1
        AND listed_at >= NOW() - INTERVAL '1 hour'
        LIMIT 1;
    "#,
        market_id
    )
    .fetch_one(pool)
    .await;
    result.is_ok()
}

/// Runs every 1 hour to ensure that the application is still receiving data. If it is not, quits
/// the whole application (Docker will restart us)
fn watchdog() {
    loop {
        thread::sleep(Duration::from_hours(1));
        let mut count = WATCHDOG_ADDED.lock().unwrap();
        if *count > 0 {
            // watchdog passed, reset the count and wait again
            info!("Watchdog OK");
            *count = 0;
        } else {
            error!("Watchdog FAILED! 0 items added in the last hour. Quitting now!");
            // docker will restart us (hopefully)
            exit(1);
        }
    }
}

pub async fn listen(url: String) -> Result<()> {
    info!("Setting up PostgreSQL pool on {}", url);
    let var_name = PgPoolOptions::new();
    let pool = var_name.max_connections(8).connect(&url).await?;

    info!("Starting watchdog");
    thread::spawn(|| {
        watchdog();
    });

    for env in eddn::subscribe(eddn::URL) {
        match env {
            Ok(envelope) => {
                if let eddn::Message::Commodity(commodity) = envelope.message {
                    let market = commodity.event;
                    let market_id = market.market_id;

                    // check if this system already has a record in the last 1 hour (to save disk
                    // space)
                    if has_listings_1h(market_id, &pool).await {
                        info!(
                            "Station {} in {} already updated in the last 1 hour, skipping",
                            market.station_name, market.system_name
                        );
                        continue;
                    }

                    // insert into the DB
                    let transaction = pool.begin().await?;
                    let time = envelope.header.gateway_timestamp.naive_utc();
                    let mut success = true;

                    for commodity in market.commodities {
                        let result = sqlx::query!(
                            r#"
                            INSERT INTO listings (
                                market_id,
                                name,
                                mean_price,
                                buy_price,
                                sell_price,
                                demand,
                                demand_bracket,
                                stock,
                                stock_bracket,
                                listed_at
                            ) VALUES (
                                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
                            );
                        "#,
                            market_id,
                            commodity.name,
                            commodity.mean_price,
                            commodity.buy_price,
                            commodity.sell_price,
                            commodity.demand,
                            commodity.demand_bracket,
                            commodity.stock,
                            commodity.stock_bracket,
                            time
                        )
                        .execute(&pool)
                        .await;

                        match result {
                            Ok(_) => {
                                let mut count = WATCHDOG_ADDED.lock().unwrap();
                                *count += 1;
                            }
                            Err(error) => {
                                warn!(
                                    "Failed to insert commodity {} for station {} in {}: {}",
                                    commodity.name, market.station_name, market.system_name, error
                                );
                                // cancel the entire update
                                success = false;
                                break;
                            }
                        }
                    }
                    transaction.commit().await?;

                    if success {
                        info!(
                            "Inserted market data for {} in {}",
                            market.station_name, market.system_name
                        );
                    }
                }
            }
            Err(error) => {
                warn!("Failed to receive EDDN message: {}", error);
            }
        }
    }

    Ok(())
}
