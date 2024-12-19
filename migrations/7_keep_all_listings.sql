ALTER TABLE listings DROP CONSTRAINT listings_pkey;

-- This allows us to keep track of all commodities
ALTER TABLE listings ADD CONSTRAINT listings_pkey PRIMARY KEY (market_id, name, listed_at);
