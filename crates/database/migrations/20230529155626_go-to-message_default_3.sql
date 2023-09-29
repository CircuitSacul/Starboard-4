-- Add migration script here
ALTER TABLE starboards ALTER COLUMN go_to_message SET DEFAULT 3;
