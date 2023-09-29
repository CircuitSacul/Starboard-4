-- 0 = None, 1 = Link, 2 = Button
ALTER TABLE starboards ADD COLUMN go_to_message SMALLINT NOT NULL DEFAULT 2;
UPDATE starboards SET go_to_message = 1 WHERE jump_to_message is true;
UPDATE starboards SET go_to_message = 0 WHERE jump_to_message is false;
ALTER TABLE starboards DROP COLUMN jump_to_message;
