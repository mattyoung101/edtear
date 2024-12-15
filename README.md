# EDTear
This is the "Elite: Dangerous Trade Ear", a service that subscribes to [EDDN](https://github.com/EDCD/EDDN),
and ingests nightly dumps from [EDSM](https://www.edsm.net/en/nightly-dumps), and sends that all to a
PostgreSQL database with PostGIS enabled.

This will hopefully be useful in a trade route calculator I'm planning to make in the future.

Parts of the SQL queries are based on [Galos](https://github.com/nixpulvis/galos/) by Nathan Lilienthal.

## Running
Setup database:

```
sqlx database create
sqlx migrate run
```

Import EDSM data:

```
cargo run --release -- ingest-edsm --systems-json-path systemsPopulated.json --stations-json-path stations.json
```

Listen to data from EDDN:

```
cargo run --release -- listen
```


## Licence
Copyright (c) 2025 Matt Young.

EDTear is available under the ISC licence.
