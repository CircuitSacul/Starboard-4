use std::sync::Arc;

use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::InteractionChannel, user::User,
};

use database::{DbMessage, Starboard, StarboardMessage, StarboardOverride};
use errors::StarboardResult;

use crate::{
    client::bot::StarboardBot,
    core::{
        embedder::{builder::BuiltStarboardEmbed, Embedder},
        premium::is_premium::is_guild_premium,
        starboard::config::StarboardConfig,
    },
    get_guild_id,
    interactions::context::CommandCtx,
    utils::{id_as_i64::GetI64, into_id::IntoId},
};

pub fn get_post_query(
    starboard_id: i32,
    allow_nsfw: bool,
    channel: Option<i64>,
    author: Option<i64>,
    min_points: Option<i16>,
    max_points: Option<i16>,
) -> sqlx::QueryBuilder<'static, sqlx::Postgres> {
    let init_query = r#"
    SELECT * FROM starboard_messages
    WHERE EXISTS (
        SELECT * FROM messages
            WHERE message_id=starboard_messages.message_id
            AND trashed=false
    "#;
    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(init_query);

    // subquery
    if !allow_nsfw {
        builder.push(" AND is_nsfw=false");
    }

    if let Some(channel) = channel {
        builder.push(" AND channel_id=").push_bind(channel);
    }
    if let Some(author) = author {
        builder.push(" AND author_id=").push_bind(author);
    }

    // outer query
    builder.push(") AND starboard_id=").push_bind(starboard_id);

    if let Some(min_points) = min_points {
        builder
            .push(" AND last_known_point_count >= ")
            .push_bind(min_points);
    }
    if let Some(max_points) = max_points {
        builder
            .push(" AND last_known_point_count <= ")
            .push_bind(max_points);
    }

    builder
}

pub async fn get_config(
    bot: &StarboardBot,
    sb: Starboard,
    channel_id: i64,
) -> StarboardResult<StarboardConfig> {
    let overrides =
        StarboardOverride::list_by_starboard_and_channels(&bot.db, sb.id, &[channel_id]).await?;
    Ok(StarboardConfig::new(sb, &[channel_id], overrides)?)
}

pub async fn get_embedder(
    bot: Arc<StarboardBot>,
    config: Arc<StarboardConfig>,
    orig_sql_msg: DbMessage,
    msg: StarboardMessage,
) -> StarboardResult<Option<Embedder>> {
    let orig_msg = bot
        .cache
        .fog_message(
            &bot,
            orig_sql_msg.channel_id.into_id(),
            orig_sql_msg.message_id.into_id(),
        )
        .await?;
    let Some(orig_msg) = orig_msg.into_option() else {
        return Ok(None);
    };

    let ref_msg = if let Some(ref_msg_id) = orig_msg.referenced_message {
        bot.cache
            .fog_message(&bot, orig_sql_msg.channel_id.into_id(), ref_msg_id)
            .await?
            .into_option()
    } else {
        None
    };

    let is_premium = is_guild_premium(&bot, config.starboard.guild_id, true).await?;
    let embedder = Embedder {
        bot,
        points: msg.last_known_point_count as i32,
        config,
        orig_message: Some(orig_msg).into(),
        referenced_message: ref_msg,
        orig_sql_message: Arc::new(orig_sql_msg),
        is_premium,
    };

    Ok(Some(embedder))
}

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "random",
    desc = "Get a random starboard post.",
    dm_permission = false
)]
pub struct RandomPost {
    /// The starboard to get a random post from.
    #[command(autocomplete = true)]
    starboard: String,

    /// Only show messages with at least this many points.
    #[command(rename = "min-points", max_value = 32767, min_value = -32767)]
    min_points: Option<i64>,
    /// Only show messages with at most this many points.
    #[command(rename = "max-points", max_value = 32767, min_value = -32767)]
    max_points: Option<i64>,
    /// Only show messages that were sent in this channel.
    channel: Option<InteractionChannel>,
    /// Only show messages sent by this user.
    author: Option<User>,
    /// Whether to allow messages from NSFW starboards.
    #[command(rename = "allow-nsfw")]
    allow_nsfw: Option<bool>,
}

impl RandomPost {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        let guild_id = get_guild_id!(ctx);
        let guild_id_i64 = guild_id.get_i64();

        let Some(sb) = Starboard::get_by_name(&ctx.bot.db, &self.starboard, guild_id_i64).await?
        else {
            ctx.respond_str(
                &format!("Starboard '{}' does not exist.", self.starboard),
                true,
            )
            .await?;
            return Ok(());
        };
        if sb.settings.private {
            ctx.respond_str(
                "That starboard is private, so you cannot get random messages from it.",
                true,
            )
            .await?;
            return Ok(());
        }

        if self.allow_nsfw == Some(true) {
            let nsfw = ctx
                .bot
                .cache
                .fog_channel_nsfw(
                    &ctx.bot,
                    guild_id,
                    ctx.interaction.channel.as_ref().unwrap().id,
                )
                .await?
                .unwrap();
            if !nsfw {
                ctx.respond_str(
                    "You can't allow NSFW messages here, since this channel isn't NSFW.",
                    true,
                )
                .await?;
                return Ok(());
            }
        }

        let mut builder = get_post_query(
            sb.id,
            self.allow_nsfw.unwrap_or(false),
            self.channel.map(|ch| ch.id.get_i64()),
            self.author.map(|user| user.id.get_i64()),
            self.min_points.map(|v| v as i16),
            self.max_points.map(|v| v as i16),
        );
        builder.push(" ORDER BY random()");
        let msg: Option<StarboardMessage> = builder
            .build_query_as()
            .fetch_optional(&ctx.bot.db.pool)
            .await?;
        let Some(msg) = msg else {
            ctx.respond_str("Nothing to show.", true).await?;
            return Ok(());
        };

        let orig_msg = DbMessage::get(&ctx.bot.db, msg.message_id).await?.unwrap();
        let config = get_config(&ctx.bot, sb, orig_msg.channel_id).await?;
        let config = Arc::new(config);
        let embedder = get_embedder(ctx.bot.clone(), config, orig_msg, msg)
            .await?
            .unwrap();

        let built = embedder.build(false, false).await?;
        let built = match built {
            BuiltStarboardEmbed::Partial(_) => unreachable!("didn't get full embed"),
            BuiltStarboardEmbed::Full(built) => built,
        };

        let data = ctx
            .build_resp()
            .content(built.top_content)
            .embeds(built.embeds)
            .components(built.components)
            .build();
        ctx.respond(data).await?;

        Ok(())
    }
}
