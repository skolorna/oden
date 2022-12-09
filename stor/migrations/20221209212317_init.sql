CREATE TABLE menus (
  id BLOB PRIMARY KEY,
  title TEXT NOT NULL,
  slug TEXT NOT NULL UNIQUE,
  longitude REAL,
  latitude REAL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW,
  checked_at TIMESTAMP
);
