CREATE TABLE IF NOT EXISTS dates (
id UUID NOT NULL,
name VARCHAR NOT NULL,
count_ INT NOT NULL,
status INT NOT NULL,
day TIMESTAMPTZ,
description VARCHAR, 
PRIMARY KEY(id)
);

