CREATE TYPE supplier AS ENUM ('skolmaten', 'sodexo', 'mpi', 'kleins', 'sabis', 'matilda');

CREATE TABLE menus (
  id UUID PRIMARY KEY,
  title TEXT NOT NULL,
  supplier supplier NOT NULL,
  supplier_reference TEXT NOT NULL,
  longitude FLOAT8,
  latitude FLOAT8,
  osm_id TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  checked_at TIMESTAMPTZ
);

CREATE TABLE days (
  menu_id UUID NOT NULL REFERENCES menus (id) ON DELETE CASCADE,
  date DATE NOT NULL,
  meals TEXT[] NOT NULL,
  PRIMARY KEY (menu_id, date)
);
