CREATE TABLE systems (
    id BIGINT PRIMARY KEY NOT NULL UNIQUE,
    name VARCHAR(128) NOT NULL,
    date TIMESTAMP NOT NULL,
    coords geometry(POINTZ) NOT NULL
);

CREATE INDEX systems_position_idx ON systems USING GIST (coords gist_geometry_ops_nd);
