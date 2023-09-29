use std::{
    collections::HashMap,
    fmt::Write,
    time::{Duration, Instant},
};

use sqlx::{postgres::PgRow, Column, Executor, Row, ValueRef};
use twilight_model::id::{
    marker::{ChannelMarker, MessageMarker},
    Id,
};

use errors::StarboardResult;

use crate::{
    cache::models::message::CachedMessage, client::bot::StarboardBot, concat_format,
    owner::code_block::parse_code_blocks,
};

pub async fn run_sql(
    bot: &StarboardBot,
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    message: &CachedMessage,
    is_edit: bool,
) -> StarboardResult<()> {
    let to_edit = if !is_edit {
        bot.http.create_typing_trigger(channel_id).await?;
        None
    } else {
        bot.cache.responses.get(&message_id)
    };

    let mut rollback = false;

    let blocks = parse_code_blocks(message.content.strip_prefix("star sql").unwrap());
    let mut results = Vec::new();

    let mut tx = bot.db.pool.begin().await?;
    for (code, meta) in &blocks {
        if meta.get("rollback").map_or(true, |v| v.parse().unwrap()) {
            rollback = true;
        }

        let return_results = meta.get("return").is_some();
        let total_execs = meta.get("runs").map_or(1, |v| v.parse().unwrap()).max(1);

        let mut result = None;
        let mut err = None;
        let mut execution_times = Vec::new();
        for _ in 0..total_execs {
            let elapsed = if return_results {
                let start = Instant::now();
                let rows = tx.fetch_all(code.as_str()).await;

                let elapsed = start.elapsed();
                match rows {
                    Ok(rows) => {
                        result.replace(Some(rows.into_iter().map(row_to_json).collect()));
                    }
                    Err(why) => {
                        err = Some(why.to_string());
                    }
                }
                elapsed
            } else {
                let start = Instant::now();
                let ret = tx.execute(code.as_str()).await;
                if let Err(why) = ret {
                    err = Some(why.to_string());
                }
                start.elapsed()
            };
            execution_times.push(elapsed);
        }

        let result = SqlResult {
            execution_times,
            inspect: result.unwrap_or(None),
            err,
            tag: meta.get("tag").unwrap_or(&"?query?").to_string(),
        };
        results.push(result);
    }

    let mut final_result = String::new();

    if rollback || !bot.config.development {
        tx.rollback().await?;
    } else {
        tx.commit().await?;
        final_result.push_str("Committed.\n\n");
    }

    for result in results {
        final_result.push_str(&concat_format!(
            "Query {} ran {} times, " <- result.tag, result.execution_times.len();
            "with an average time of {:?}.\n" <- result.average_time();
        ));

        if let Some(inspect) = result.inspect {
            final_result.push_str("```rs\n");
            let mut x = 0;
            for row in inspect {
                x += 1;
                if x == 5 {
                    final_result.push_str("- and more...");
                    break;
                }
                writeln!(final_result, "- {row:?}").unwrap();
            }
            final_result.push_str("```\n");
        }
        if let Some(err) = result.err {
            writeln!(final_result, "```sql\n{err}```").unwrap();
        }
    }

    if let Some(to_edit) = to_edit {
        let ret = bot
            .http
            .update_message(channel_id, to_edit)
            .content(Some(&final_result))?
            .await;

        if ret.is_ok() {
            return Ok(());
        }
    }

    let msg = bot
        .http
        .create_message(channel_id)
        .content(&final_result)?
        .reply(message_id)
        .await?
        .model()
        .await?;

    bot.cache.responses.insert(message_id, msg.id).await;

    Ok(())
}

fn row_to_json(row: PgRow) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for col in row.columns() {
        let value = row.try_get_raw(col.ordinal()).unwrap();
        let value = if value.is_null() {
            "NULL".to_string()
        } else {
            value.as_str().unwrap().to_string()
        };
        result.insert(col.name().to_string(), value);
    }

    result
}

#[derive(Debug)]
struct SqlResult {
    pub execution_times: Vec<Duration>,
    pub inspect: Option<Vec<HashMap<String, String>>>,
    pub err: Option<String>,
    pub tag: String,
}

impl SqlResult {
    pub fn average_time(&self) -> Duration {
        let mut total = Duration::new(0, 0);
        for time in &self.execution_times {
            total += *time;
        }
        total.div_f64(self.execution_times.len() as f64)
    }
}
