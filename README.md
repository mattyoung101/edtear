# EDTear
This is the "Elite: Dangerous Trade Ear", a service that subscribes to [EDDN](https://github.com/EDCD/EDDN),
and ingests nightly dumps from [EDSM](https://www.edsm.net/en/nightly-dumps), and sends that all to a
PostgreSQL database with PostGIS enabled.

This will hopefully be useful in a trade route calculator I'm planning to make in the future.

## Running
Setup database:

```
sqlx database create
sqlx migrate run
```

Import EDSM data:

```
cargo run -- ingest-edsm --systems-json-path systemsWithCoordinates.json --stations-json-path stations.json
```

## Licence
Copyright (c) 2025 Matt Young.

EDTear is available under the ISC licence.
