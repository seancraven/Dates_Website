-- Add migration script here
ALTER TABLE users ADD COLUMN auth BOOLEAN DEFAULT false;
