use crate::daytype::DayType;
use crate::subscription::{Subscription, Subscriptions};
use chrono::offset::FixedOffset;
use chrono::prelude::*;
use rand::Rng;
use serenity::prelude::Context;
use std::sync::Arc;
use tokio::time::sleep;
use tracing::error;

pub async fn message_task(ctx: Arc<Context>) {
  loop {
    let db = {
      let data = ctx.data.read().await;
      data
        .get::<Subscriptions>()
        .expect("No Subscriptions in shared data")
        .clone()
    };

    let offset = FixedOffset::east(2 * 3600);
    let now = Utc::now().with_timezone(&offset);

    let sleep_for = {
      let next = now
        .with_nanosecond(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .checked_add_signed(chrono::Duration::minutes(1))
        .unwrap();

      (next - now).to_std().unwrap()
    };

    let subscriptions_fetch =
      tokio::spawn(async move { db.find_all::<Subscription>().await });

    let day_type = match DayType::try_parse_from_date(now) {
      Err(_e) => {
        sleep(sleep_for).await;
        continue;
      }
      Ok(t) => t,
    };

    let subscriptions = match subscriptions_fetch.await {
      Err(e) => {
        error!("Couldn't join task: {}", e);
        sleep(sleep_for).await;
        continue;
      }
      Ok(data) => match data {
        Err(e) => {
          error!("Couldn't find subscriptions: {}", e);
          sleep(sleep_for).await;
          continue;
        }
        Ok(documents) => documents
          .into_iter()
          .map(|d| d.data)
          .collect::<Vec<Subscription>>(),
      },
    };

    for subscription in subscriptions {
      if let Some(t) = subscription.last_message {
        if t == day_type.data {
          continue;
        }
      }

      let channel = subscription.channel;
      let skip = rand::thread_rng().gen_bool(day_type.odds_to_skip);

      if skip && !day_type.last_possible {
        continue;
      }

      if let Err(e) = channel
        .send_message(&ctx.http, |message| {
          message.embed(|embed| embed.title(&day_type.data).colour(0xFFFFFF))
        })
        .await
      {
        error!(
          "Error sending automatic message for ChannelId({}): {}",
          channel, e
        );
        continue;
      }

      let new_subscription = Subscription {
        guild: subscription.guild,
        channel: subscription.channel,
        last_message: Some(day_type.data),
      };

      subscription.update(&ctx, new_subscription).await;
    }

    sleep(sleep_for).await;
  }
}
