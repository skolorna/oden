BEGIN;

CREATE TABLE meals (
  menu_id UUID NOT NULL REFERENCES menus(id) ON DELETE CASCADE,
  date DATE NOT NULL,
  meal TEXT NOT NULL,
  PRIMARY KEY (menu_id, date, meal)
);

INSERT INTO
  meals (menu_id, date, meal)
SELECT
  menu_id,
  date,
  unnest(meals)
FROM
  days ON CONFLICT DO NOTHING;

DROP TABLE days;

COMMIT;
