CREATE TABLE tests (
  id VARCHAR PRIMARY KEY NOT NULL,
  title VARCHAR NOT NULL,
  body TEXT,
  published BOOLEAN NOT NULL DEFAULT 'f'
)
