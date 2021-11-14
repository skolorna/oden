-- Your SQL goes here
CREATE TABLE days (
  date DATE NOT NULL,
  meals BYTEA NOT NULL,
  menu_id SERIAL NOT NULL,
  PRIMARY KEY (date, menu_id),
  FOREIGN KEY (menu_id) REFERENCES menus(id) ON DELETE CASCADE
);
