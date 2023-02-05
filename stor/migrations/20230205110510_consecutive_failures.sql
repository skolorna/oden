ALTER TABLE
  menus
ADD
  COLUMN consecutive_failures INT NOT NULL DEFAULT 0;
