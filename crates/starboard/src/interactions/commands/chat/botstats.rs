use std::time::Duration;

use psutil::{cpu, memory};
use thousands::Separable;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_util::builder::embed::EmbedFieldBuilder;

use errors::StarboardResult;

use crate::{interactions::context::CommandCtx, utils::embed};

fn fmt_b(b: u64) -> String {
    let gb = b as f32 / 10u32.pow(9) as f32;
    format!("{gb:.1} GB")
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "botstats", desc = "Show bot statistics.")]
pub struct BotStats;

impl BotStats {
    pub async fn callback(self, mut ctx: CommandCtx) -> StarboardResult<()> {
        ctx.defer(false).await?;

        // collect system stats
        let mut cpu_stats = cpu::CpuPercentCollector::new().unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;

        let cpu_percent = cpu_stats.cpu_percent().unwrap();
        let cpu_count = cpu::cpu_count_physical();

        let virt = memory::virtual_memory().unwrap();

        // other stats
        let guilds = ctx.bot.cache.guilds.len().separate_with_commas();

        let cached_users = ctx.bot.cache.users.entry_count().separate_with_commas();
        let cached_members = ctx.bot.cache.members.entry_count().separate_with_commas();
        let cached_messages = ctx.bot.cache.messages.entry_count().separate_with_commas();

        #[allow(clippy::uninlined_format_args)]
        let emb = embed::build()
            .title("Starboard's Stats")
            .description(format!(
                "Total Servers: {guilds}\nLast Restart: <t:{}:R>",
                ctx.bot.start.timestamp()
            ))
            .field(EmbedFieldBuilder::new(
                "Cache",
                format!(
                    "Cached Users: {}\nCached Members: {}\nCached Messages: {}",
                    cached_users, cached_members, cached_messages
                ),
            ))
            .field(EmbedFieldBuilder::new(
                "System Stats",
                format!(
                    "CPU: {:.0}% ({} cores)\nMemory: {}/{}",
                    cpu_percent,
                    cpu_count,
                    fmt_b(virt.used()),
                    fmt_b(virt.total()),
                ),
            ))
            .build();
        ctx.respond(ctx.build_resp().embeds([emb]).build()).await?;

        Ok(())
    }
}
