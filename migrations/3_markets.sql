CREATE TABLE listings (
    market_id BIGINT NOT NULL,
    name VARCHAR(128),
    mean_price INT,
    buy_price INT,
    sell_price INT,
    demand INT,
    demand_bracket INT,
    stock INT,
    stock_bracket INT,
    listed_at timestamp,

    PRIMARY KEY (market_id, name),
    FOREIGN KEY (market_id) REFERENCES stations(market_id)
);
