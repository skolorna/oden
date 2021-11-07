-- Your SQL goes here
CREATE TABLE meals (
  id BYTEA PRIMARY KEY,
  date DATE NOT NULL,
  value TEXT NOT NULL,
  menu_id SERIAL NOT NULL,
  FOREIGN KEY (menu_id) REFERENCES menus(id) ON DELETE CASCADE
);