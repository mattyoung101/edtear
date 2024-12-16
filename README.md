# EDTear
This is the "Elite: Dangerous Trade Ear", a service that subscribes to [EDDN](https://github.com/EDCD/EDDN),
and ingests nightly dumps from [EDSM](https://www.edsm.net/en/nightly-dumps), and sends that all to a
PostgreSQL database with PostGIS enabled.

This will hopefully be useful in a trade route calculator I'm planning to make in the future.

Parts of the SQL queries are based on [Galos](https://github.com/nixpulvis/galos/) by Nathan Lilienthal.

## Running
Requires a recent Rust version (I'm using 1.82), PostgresSQL 16+ with PostGIS, and the Rust `sqlx` CLI.

Setup database:

```
sqlx database create
sqlx migrate run
```

Import EDSM data:

```
cargo run --release -- ingest-edsm --systems-json-path systemsPopulated.json --stations-json-path stations.json --url postgres://postgres:password@localhost/edtear
```

Listen to data from EDDN:

```
cargo run --release -- listen --url postgres://postgres:password@localhost/edtear
```

## Running in Docker Compose
Once you've set up the database locally, you can also run EDTear through Docker Compose, which may be useful
for server deployments.

Make a file named `postgres.env` in this directory (and on the server) with the contents:

```
POSTGRES_PASSWORD=<password>
```

On your host system, dump the local database:

```shell
sudo -u postgres pg_dump -Fc edtear > edtear.dump
```

On the server, run `docker compose up -d postgres` to spawn a Postgres instance.

Import the data on the server (should prompt you for your password):

```shell
pg_restore -U postgres -h localhost -p 6543 -d edtear edtear.dump
```

Shutdown the current Postgres: `docker compose down`

And now deploy with the imported data: `docker compose up -d`

## Licence
Copyright (c) 2024 Matt Young.

EDTear is available under the ISC licence.
