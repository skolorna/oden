-- Your SQL goes here
CREATE TABLE days (
  id SERIAL PRIMARY KEY,
  date DATE NOT NULL,
  meals BYTEA NOT NULL,
  menu_id SERIAL NOT NULL,
  FOREIGN KEY (menu_id) REFERENCES menus(id) ON DELETE CASCADE,
  UNIQUE (date, menu_id)
);