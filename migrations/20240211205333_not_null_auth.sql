-- Add migration script here
ALTER TABLE users ALTER COLUMN auth SET NOT NULL;
