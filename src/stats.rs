use color_eyre::eyre::Result;
use comfy_table::{presets::UTF8_FULL, Table};
use log::info;
use num2english::NumberToEnglish;
use num_format::{Locale, ToFormattedString};
use sqlx::postgres::PgPoolOptions;

/// Round to significant digits (rather than digits after the decimal).
///
/// Not implemented for `f32`, because such an implementation showed precision
/// glitches (e.g. `precision_f32(12300.0, 2) == 11999.999`), so for `f32`
/// floats, convert to `f64` for this function and back as needed.
///
/// Examples:
/// ```
///   precision_f64(1.2300, 2)                      // 1.2<f64>
///   precision_f64(1.2300_f64, 2)                  // 1.2<f64>
///   precision_f64(1.2300_f32 as f64, 2)           // 1.2<f64>
///   precision_f64(1.2300_f32 as f64, 2) as f32    // 1.2<f32>
/// ```
/// Source: https://stackoverflow.com/a/76572321/5007892
fn precision_f64(x: f64, decimals: u32) -> f64 {
    if x == 0. || decimals == 0 {
        0.
    } else {
        let shift = decimals as i32 - x.abs().log10().ceil() as i32;
        let shift_factor = 10_f64.powi(shift);

        (x * shift_factor).round() / shift_factor
    }
}

pub async fn display_stats(url: String) -> Result<()> {
    info!("Setting up PostgreSQL pool on {}", url);
    let var_name = PgPoolOptions::new();
    let pool = var_name.max_connections(8).connect(&url).await?;

    info!("Computing statistics");
    let num_systems_future = sqlx::query!("SELECT COUNT(*) FROM systems;").fetch_one(&pool);
    let num_stations_future = sqlx::query!("SELECT COUNT(*) FROM stations;").fetch_one(&pool);
    let num_listings_future = sqlx::query!("SELECT COUNT(*) FROM listings;").fetch_one(&pool);
    let unique_stations_future =
        sqlx::query!("SELECT COUNT(DISTINCT market_id) FROM listings;").fetch_one(&pool);
    let latest_listings_future = sqlx::query!(
        r#"
        WITH latest_listings AS (
            SELECT
                market_id,
                name,
                MAX(listed_at) AS latest_listed_at
            FROM
                listings
            WHERE stock > 0
            GROUP BY
                market_id, name
        )
        SELECT
            l.market_id,
            l.name,
            l.mean_price,
            l.buy_price,
            l.sell_price,
            l.demand,
            l.demand_bracket,
            l.stock,
            l.stock_bracket,
            l.listed_at
        FROM
            listings l
        INNER JOIN
            latest_listings ll
        ON
            l.market_id = ll.market_id
            AND l.name = ll.name
            AND l.listed_at = ll.latest_listed_at
        WHERE l.stock > 0;
    "#
    )
    .fetch_all(&pool);
    let most_expensive_future = sqlx::query!(
        r#"
        SELECT stat.name as station_name, sys.name as sys_name, list.buy_price as buy_price,
            list.stock as stock, list.name as commodity_name
        FROM listings list
        INNER JOIN stations stat ON list.market_id = stat.market_id
        INNER JOIN systems sys ON stat.id = sys.id
        WHERE stock > 0
        ORDER BY list.buy_price DESC
        LIMIT 1;
    "#
    )
    .fetch_one(&pool);
    let most_numerous_future = sqlx::query!(
        r#"
        SELECT stat.name as station_name, sys.name as sys_name, list.buy_price as buy_price,
            list.stock as stock, list.name as commodity_name
        FROM listings list
        INNER JOIN stations stat ON list.market_id = stat.market_id
        INNER JOIN systems sys ON stat.id = sys.id
        ORDER BY list.stock DESC
        LIMIT 1;
    "#
    )
    .fetch_one(&pool);

    #[cfg_attr(rustfmt, rustfmt_skip)]
    let (
            num_systems,
            num_stations,
            num_listings,
            unique_stations,
            latest_listings,
            most_expensive,
            most_numerous
        )
    = futures::join!(
        num_systems_future,
        num_stations_future,
        num_listings_future,
        unique_stations_future,
        latest_listings_future,
        most_expensive_future,
        most_numerous_future
    );

    let unique_stations_count = unique_stations.unwrap().count.unwrap();
    let total_stations_count = num_stations.unwrap().count.unwrap();

    let mut total_galaxy_cost: u64 = 0;
    if let Ok(group) = latest_listings {
        for listing in group {
            total_galaxy_cost += listing.buy_price as u64 * listing.stock as u64;
        }
    }
    let exp = most_expensive.unwrap();
    let num = most_numerous.unwrap();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    // table.apply_modifier(UTF8_ROUND_CORNERS);
    table.set_header(vec!["Statistic", "Value"]);
    table.add_row(vec![
        "Systems",
        &num_systems
            .unwrap()
            .count
            .unwrap()
            .to_formatted_string(&Locale::en_AU),
    ]);
    table.add_row(vec![
        "Stations",
        &total_stations_count.to_formatted_string(&Locale::en_AU),
    ]);
    table.add_row(vec![
        "Listings",
        &num_listings
            .unwrap()
            .count
            .unwrap()
            .to_formatted_string(&Locale::en_AU),
    ]);
    table.add_row(vec![
        "Unique stations tracked",
        format!(
            "{} ({}%)",
            unique_stations_count.to_formatted_string(&Locale::en_AU),
            (unique_stations_count as f32) / (total_stations_count as f32) * 100.0
        )
        .as_str(),
    ]);
    table.add_row(vec![
        "Cost to buy the entire galaxy",
        &format!(
            "${}\n({})",
            total_galaxy_cost.to_formatted_string(&Locale::en_AU),
            (precision_f64(total_galaxy_cost as f64, 1) as u64).to_english()
        ),
    ]);
    table.add_row(vec![
        "Most expensive item",
        &format!(
            "{}\nFrom {} in {}\nCost ${}, units available {}",
            &exp.commodity_name,
            &exp.station_name,
            &exp.sys_name,
            &exp.buy_price.to_formatted_string(&Locale::en_AU),
            &exp.stock.to_formatted_string(&Locale::en_AU)
        ),
    ]);
    table.add_row(vec![
        "Most numerous item",
        &format!(
            "{}\nFrom {} in {}\nCost ${}, units available {}",
            &num.commodity_name,
            &num.station_name,
            &num.sys_name,
            &num.buy_price.to_formatted_string(&Locale::en_AU),
            &num.stock.to_formatted_string(&Locale::en_AU)
        ),
    ]);

    // listings (last 24 hours)
    // unique stations updated (last 24 hours)
    // top 5 stations with most listings (i.e. the most visited ones)
    // number of unique commodities tracked
    // total number of commodities tracked

    println!("{table}");
    Ok(())
}
