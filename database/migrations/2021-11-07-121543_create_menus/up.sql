-- Your SQL goes here
CREATE TABLE menus (
  id SERIAL PRIMARY KEY,
  title TEXT NOT NULL,
  slug TEXT NOT NULL UNIQUE,
  updated_at TIMESTAMPTZ
);