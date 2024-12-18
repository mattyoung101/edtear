ALTER TABLE stations DROP COLUMN landing_pad;

ALTER TABLE stations ADD COLUMN landing_pad varchar(16);
