ALTER TABLE games ADD COLUMN status TEXT CHECK (status IS NULL OR status IN ('playing','completed','backlog','dropped'));
ALTER TABLE games ADD COLUMN rating INTEGER CHECK (rating IS NULL OR (rating BETWEEN 1 AND 5));
