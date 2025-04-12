# EDTear
This is the "Elite: Dangerous Trade Ear", a service that subscribes to [EDDN](https://github.com/EDCD/EDDN),
and ingests nightly dumps from [EDSM](https://www.edsm.net/en/nightly-dumps) and
[Spansh](https://spansh.co.uk/dumps), and sends all that to a PostgreSQL database with PostGIS enabled. It
also allows you to see some interesting statistics about the Elite: Dangerous trade universe, once the data
has been collected.

EDTear is used to create the data for [Kural](https://github.com/mattyoung101/kural), my other project to
build a high-performance trade-route calculator based on integer linear programming.

Parts of the SQL queries are based on [Galos](https://github.com/nixpulvis/galos/) by Nathan Lilienthal.

## Running
Requires a recent Rust version (I'm using 1.82), PostgresSQL 16+ with PostGIS, and the Rust `sqlx` CLI.

Setup the database:

```
sqlx database create
sqlx migrate run
```

Import EDSM data (download it [here](https://www.edsm.net/en/nightly-dumps)):

```
cargo run --release -- ingest-edsm --systems-json-path systemsPopulated.json --stations-json-path stations.json --url postgres://postgres:password@localhost/edtear
```

Import Spansh's data (for landing pad sizes, which are not part of EDSM; download it [here](https://spansh.co.uk/dumps)):

```
cargo run --release -- ingest-spansh --systems --galaxy-stations-json-path stations.json --url postgres://postgres:password@localhost/edtear
```

Listen to data from EDDN:

```
cargo run --release -- listen --url postgres://postgres:password@localhost/edtear
```

See some interesting statistics about the data, once collected:

```
cargo run --release -- stats --url postgres://postgres:password@localhost/edtear
```

If no url is supplied EDTear will default to the environment variable `DATABASE_URL` supplied within `.env`:
```
DATABASE_URL=postgres://postgres:password@localhost/edtear
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
Copyright (c) 2024-2025 Matt Young.

EDTear is available under the ISC licence.

## Special thanks
Special thanks to Nathan Lilienthal, EDSM, Spansh, and all the commanders who generously provide data to EDDN.
