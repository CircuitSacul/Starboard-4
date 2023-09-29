CREATE TABLE IF NOT EXISTS guilds (
    guild_id BIGINT NOT NULL,
    premium_end TIMESTAMPTZ,

    PRIMARY KEY (guild_id)
);

CREATE TABLE IF NOT EXISTS users (
    user_id BIGINT NOT NULL,
    is_bot BOOLEAN NOT NULL,
    credits INTEGER NOT NULL DEFAULT 0,
    -- total cents donated excluding patreon
    donated_cents BIGINT NOT NULL DEFAULT 0,
    patreon_status SMALLINT NOT NULL DEFAULT 0,

    PRIMARY KEY (user_id)
);

CREATE TABLE IF NOT EXISTS patrons (
    patreon_id VARCHAR(64) NOT NULL,
    discord_id BIGINT,
    last_patreon_total_cents BIGINT NOT NULL DEFAULT 0,

    FOREIGN KEY (discord_id) REFERENCES users (user_id)
        MATCH SIMPLE
        ON DELETE RESTRICT
        ON UPDATE CASCADE,

    PRIMARY KEY (patreon_id)
);

CREATE TABLE IF NOT EXISTS members (
    user_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    xp REAL NOT NULL DEFAULT 0,
    autoredeem_enabled BOOLEAN NOT NULL DEFAULT false,

    FOREIGN KEY (user_id) REFERENCES users (user_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,
    FOREIGN KEY (guild_id) REFERENCES guilds (guild_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    PRIMARY KEY (user_id, guild_id)
);

CREATE TABLE IF NOT EXISTS starboards (
    id SERIAL NOT NULL,
    name TEXT NOT NULL,
    channel_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    webhook_id BIGINT,
    premium_locked BOOLEAN NOT NULL DEFAULT false,

    -- style
    display_emoji TEXT DEFAULT '⭐',
    ping_author BOOLEAN NOT NULL DEFAULT false,
    use_server_profile BOOLEAN NOT NULL DEFAULT true,
    extra_embeds BOOLEAN NOT NULL DEFAULT true,
    use_webhook BOOLEAN NOT NULL DEFAULT false,

    -- embed style
    color INTEGER,
    jump_to_message BOOLEAN NOT NULL DEFAULT true,
    attachments_list BOOLEAN NOT NULL DEFAULT true,
    replied_to BOOLEAN NOT NULL DEFAULT true,

    -- requirements
    required SMALLINT NOT NULL DEFAULT 3,
    required_remove SMALLINT NOT NULL DEFAULT 0,
    upvote_emojis TEXT[] NOT NULL DEFAULT '{"⭐"}',
    downvote_emojis TEXT[] NOT NULL DEFAULT '{}',
    self_vote BOOLEAN NOT NULL DEFAULT false,
    allow_bots BOOLEAN NOT NULL DEFAULT true,
    require_image BOOLEAN NOT NULL DEFAULT false,
    older_than BIGINT NOT NULL DEFAULT 0,
    newer_than BIGINT NOT NULL DEFAULT 0,

    -- behaviour
    enabled BOOLEAN NOT NULL DEFAULT true,
    autoreact_upvote BOOLEAN NOT NULL DEFAULT true,
    autoreact_downvote BOOLEAN NOT NULL DEFAULT true,
    remove_invalid_reactions BOOLEAN NOT NULL DEFAULT true,
    link_deletes BOOLEAN NOT NULL DEFAULT false,
    link_edits BOOLEAN NOT NULL DEFAULT true,
    private BOOLEAN NOT NULL DEFAULT false,
    xp_multiplier REAL NOT NULL DEFAULT 1.0,
    cooldown_enabled BOOLEAN NOT NULL DEFAULT false,
    cooldown_count SMALLINT NOT NULL DEFAULT 5,
    cooldown_period SMALLINT NOT NULL DEFAULT 5,

    UNIQUE (guild_id, name),

    FOREIGN KEY (guild_id) REFERENCES guilds (guild_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS overrides (
    id SERIAL NOT NULL,
    guild_id BIGINT NOT NULL,
    name TEXT NOT NULL,
    starboard_id INTEGER NOT NULL,
    channel_ids BIGINT[] NOT NULL DEFAULT '{}',
    overrides JSON NOT NULL DEFAULT '{}',

    FOREIGN KEY (guild_id) REFERENCES guilds (guild_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,
    FOREIGN KEY (starboard_id) REFERENCES starboards (id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS permroles (
    role_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    obtain_xproles BOOLEAN,
    give_votes BOOLEAN,
    receive_votes BOOLEAN,

    FOREIGN KEY (guild_id) REFERENCES guilds (guild_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    PRIMARY KEY (role_id)
);

CREATE TABLE IF NOT EXISTS permrole_starboards (
    permrole_id BIGINT NOT NULL,
    starboard_id INTEGER NOT NULL,
    give_votes BOOLEAN,
    receive_votes BOOLEAN,

    FOREIGN KEY (starboard_id) REFERENCES starboards (id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,
    FOREIGN KEY (permrole_id) REFERENCES permroles (role_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    PRIMARY KEY (permrole_id, starboard_id)
);

CREATE TABLE IF NOT EXISTS autostar_channels (
    id SERIAL NOT NULL,
    name TEXT NOT NULL,
    channel_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    premium_locked BOOLEAN NOT NULL DEFAULT false,
    emojis TEXT[] NOT NULL DEFAULT '{}',
    min_chars SMALLINT NOT NULL DEFAULT 0,
    max_chars SMALLINT,
    require_image BOOLEAN NOT NULL DEFAULT false,
    delete_invalid BOOLEAN NOT NULL DEFAULT false,

    UNIQUE (guild_id, name),

    FOREIGN KEY (guild_id) REFERENCES guilds (guild_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS xproles (
    role_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    required SMALLINT NOT NULL,

    FOREIGN KEY (guild_id) REFERENCES guilds (guild_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    PRIMARY KEY (role_id)
);

CREATE TABLE IF NOT EXISTS posroles (
    role_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    max_members INTEGER NOT NULL,

    FOREIGN KEY (guild_id) REFERENCES guilds (guild_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    PRIMARY KEY (role_id)
);

CREATE TABLE IF NOT EXISTS messages (
    message_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL,
    author_id BIGINT NOT NULL,
    is_nsfw BOOLEAN NOT NULL,
    forced_to INTEGER[] NOT NULL DEFAULT '{}',
    trashed BOOLEAN NOT NULL DEFAULT false,
    trash_reason VARCHAR,
    frozen BOOLEAN NOT NULL DEFAULT false,

    FOREIGN KEY (guild_id) REFERENCES guilds (guild_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,
    FOREIGN KEY (author_id) REFERENCES users (user_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    PRIMARY KEY (message_id)
);

CREATE TABLE IF NOT EXISTS starboard_messages (
    message_id BIGINT NOT NULL,
    starboard_id INTEGER NOT NULL,
    starboard_message_id BIGINT NOT NULL,
    last_known_point_count SMALLINT NOT NULL DEFAULT 0,

    FOREIGN KEY (message_id) REFERENCES messages (message_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,
    FOREIGN KEY (starboard_id) REFERENCES starboards (id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    PRIMARY KEY (message_id, starboard_id)
);

CREATE TABLE IF NOT EXISTS votes (
    message_id BIGINT NOT NULL,
    starboard_id INTEGER NOT NULL,
    user_id BIGINT NOT NULL,
    target_author_id BIGINT NOT NULL,
    is_downvote BOOLEAN NOT NULL,

    FOREIGN KEY (message_id) REFERENCES messages (message_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,
    FOREIGN KEY (starboard_id) REFERENCES starboards (id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users (user_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,
    FOREIGN KEY (target_author_id) REFERENCES users (user_id)
        MATCH SIMPLE
        ON DELETE CASCADE
        ON UPDATE CASCADE,

    PRIMARY KEY (message_id, starboard_id, user_id)
);

-- indexes
CREATE INDEX IF NOT EXISTS patrons__discord_id ON patrons USING BTREE ((discord_id));

CREATE INDEX IF NOT EXISTS aschannels__guild_id_name ON autostar_channels USING BTREE ((guild_id), (name));
CREATE INDEX IF NOT EXISTS aschannels__channel_id ON autostar_channels USING BTREE ((channel_id));

CREATE INDEX IF NOT EXISTS guilds__premium_end ON guilds USING BTREE ((premium_end));

CREATE INDEX IF NOT EXISTS messages__trashed ON messages USING BTREE (trashed)
    WHERE trashed=true;

CREATE INDEX IF NOT EXISTS members__guild_id ON members USING BTREE ((guild_id));
CREATE INDEX IF NOT EXISTS members__user_id ON members USING BTREE ((user_id));
CREATE INDEX IF NOT EXISTS members__autoredeem_enabled ON members USING BTREE ((autoredeem_enabled))
    WHERE autoredeem_enabled=true;
CREATE INDEX IF NOT EXISTS members__guild_id_xp ON members USING BTREE ((guild_id), (xp))
    WHERE xp > 0;

CREATE UNIQUE INDEX IF NOT EXISTS overrides__guild_id_name ON overrides USING BTREE ((guild_id), (name));
CREATE INDEX IF NOT EXISTS overrides__starboard_id ON overrides USING BTREE ((starboard_id));
CREATE INDEX IF NOT EXISTS overrides__channel_ids ON overrides USING GIN ((channel_ids));

CREATE UNIQUE INDEX IF NOT EXISTS sb_messages__sb_message_id ON starboard_messages USING BTREE ((starboard_message_id));
CREATE INDEX IF NOT EXISTS sb_messages__last_known_point_count ON starboard_messages USING BTREE ((last_known_point_count));
CREATE INDEX IF NOT EXISTS sb_messages__starboard_id ON starboard_messages USING BTREE ((starboard_id));

CREATE INDEX IF NOT EXISTS permroles__guild_id ON permroles USING BTREE ((guild_id));

CREATE UNIQUE INDEX IF NOT EXISTS posroles__guild_id_max_members ON posroles USING BTREE ((guild_id), (max_members));

CREATE INDEX IF NOT EXISTS starboards__guild_id_name ON starboards USING BTREE ((guild_id), (name));
CREATE INDEX IF NOT EXISTS starboards__channel_id ON starboards USING BTREE ((channel_id));

CREATE INDEX IF NOT EXISTS xproles__guild_id ON xproles USING BTREE ((guild_id));

CREATE INDEX IF NOT EXISTS votes__starboard_id ON votes USING BTREE ((starboard_id));
CREATE INDEX IF NOT EXISTS votes__user_id ON votes USING BTREE ((user_id));
CREATE INDEX IF NOT EXISTS votes__message_id ON votes USING BTREE ((message_id));
CREATE INDEX IF NOT EXISTS votes__target_author_id ON votes USING BTREE ((target_author_id));
CREATE INDEX IF NOT EXISTS votes__is_downvote ON votes USING BTREE ((is_downvote));
