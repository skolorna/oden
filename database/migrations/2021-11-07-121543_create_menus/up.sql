-- Your SQL goes here
CREATE TABLE menus (
  id UUID PRIMARY KEY,
  title TEXT NOT NULL,
  slug TEXT NOT NULL UNIQUE,
  updated_at TIMESTAMPTZ
);
