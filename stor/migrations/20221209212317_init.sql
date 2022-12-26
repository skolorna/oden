CREATE TABLE menus (
  id BLOB PRIMARY KEY,
  title TEXT NOT NULL,
  supplier TEXT NOT NULL,
  supplier_reference TEXT NOT NULL,
  longitude REAL,
  latitude REAL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW,
  checked_at TIMESTAMP
);

CREATE TABLE days (
  menu_id BLOB NOT NULL REFERENCES menus (id) ON DELETE CASCADE,
  date DATE NOT NULL,
  meals BLOB NOT NULL,
  PRIMARY KEY (menu_id, date)
)
