ALTER TABLE listings ALTER COLUMN name SET NOT NULL;
ALTER TABLE listings ALTER COLUMN mean_price SET NOT NULL;
ALTER TABLE listings ALTER COLUMN buy_price SET NOT NULL;
ALTER TABLE listings ALTER COLUMN sell_price SET NOT NULL;
ALTER TABLE listings ALTER COLUMN demand SET NOT NULL;
ALTER TABLE listings ALTER COLUMN demand_bracket SET NOT NULL;
ALTER TABLE listings ALTER COLUMN stock SET NOT NULL;
ALTER TABLE listings ALTER COLUMN stock_bracket SET NOT NULL;
ALTER TABLE listings ALTER COLUMN listed_at SET NOT NULL;