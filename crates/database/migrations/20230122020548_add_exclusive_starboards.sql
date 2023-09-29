CREATE TABLE exclusive_groups (
    id SERIAL NOT NULL,
    guild_id BIGINT NOT NULL,
    name TEXT NOT NULL,

    FOREIGN KEY (guild_id) REFERENCES guilds (guild_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    PRIMARY KEY (id)
);

ALTER TABLE starboards ADD COLUMN exclusive_group INT;
ALTER TABLE starboards ADD COLUMN exclusive_group_priority SMALLINT NOT NULL DEFAULT 0;
ALTER TABLE starboards ADD CONSTRAINT fk_starboards_exclusive_group
    FOREIGN KEY (exclusive_group) REFERENCES exclusive_groups (id)
        MATCH SIMPLE
        ON DELETE SET NULL
        ON UPDATE CASCADE;

CREATE UNIQUE INDEX exclusive_groups__guild_id_name ON exclusive_groups USING BTREE (guild_id, name);
