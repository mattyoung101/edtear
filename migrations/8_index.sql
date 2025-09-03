CREATE INDEX idx_listings_market_name_listedat
    ON listings (market_id, name, listed_at DESC);
