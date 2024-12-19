use color_eyre::eyre::Result;
use log::{info, warn};
use sqlx::postgres::PgPoolOptions;

// Ingests data from the Elite: Dangerous Data Network

pub async fn listen(url: String) -> Result<()> {
    info!("Setting up PostgreSQL pool on {}", url);
    let var_name = PgPoolOptions::new();
    let pool = var_name
        .max_connections(8)
        .connect(&url)
        .await?;

    for env in eddn::subscribe(eddn::URL) {
        match env {
            Ok(envelope) => {
                if let eddn::Message::Commodity(commodity) = envelope.message {
                    let market = commodity.event;
                    info!(
                        "Received market for {} in {}",
                        market.station_name, market.system_name
                    );

                    let market_id = market.market_id as i64;

                    // insert into the DB
                    let transaction = pool.begin().await?;

                    let time = envelope.header.gateway_timestamp.naive_utc();

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
                            Ok(_) => {}
                            Err(error) => {
                                warn!("Failed to insert commodity {}: {}", commodity.name, error);
                                // cancel the update, makes logging clearer (maybe clear this for
                                // prod)
                                break;
                            }
                        }
                    }

                    transaction.commit().await?;
                }
            }
            Err(error) => {
                warn!("Failed to receive EDDN message: {}", error);
            }
        }
    }

    Ok(())
}
