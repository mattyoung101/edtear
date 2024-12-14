CREATE TYPE LandingPad AS (
    large   smallint,
    medium  smallint,
    small   smallint
);

CREATE TABLE stations (
    id BIGINT PRIMARY KEY NOT NULL,
    distance_to_arrival REAL,
    name VARCHAR(128) NOT NULL,
    market_id BIGINT,
    system_id BIGINT,
    landing_pad LandingPad,

    FOREIGN KEY (system_id) REFERENCES systems(id)
)
