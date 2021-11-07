-- Your SQL goes here
CREATE TABLE days (
  id TEXT PRIMARY KEY,
  date DATE NOT NULL,
  meals TEXT NOT NULL,
  menu_id TEXT NOT NULL,
  FOREIGN KEY (menu_id) REFERENCES menus(id) ON DELETE CASCADE
);