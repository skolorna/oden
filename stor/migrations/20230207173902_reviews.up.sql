CREATE TABLE reviews (
  id UUID NOT NULL PRIMARY KEY,
  author UUID NOT NULL,
  menu_id UUID NOT NULL,
  date DATE NOT NULL,
  meal TEXT NOT NULL,
  rating INTEGER NOT NULL,
  comment TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  edited_at TIMESTAMPTZ,
  UNIQUE (author, menu_id, date, meal),
  FOREIGN KEY (menu_id, date, meal) REFERENCES meals(menu_id, date, meal) ON DELETE CASCADE,
  CHECK (rating between 1 and 5)
);
