use std::sync::Arc;

use crate::daytype::DayType;
use reddb::{serializer::Ron, FileStorage, RedDb};
use serde::{Deserialize, Serialize};
use serenity::model::prelude::{ChannelId, GuildId};
use serenity::prelude::{Context, TypeMapKey};
use tracing::{debug, error, info};

#[derive(Clone, Serialize, Eq, PartialEq, Deserialize, Debug)]
pub struct Subscription {
  pub guild: GuildId,
  pub channel: ChannelId,
  pub last_message: Option<DayType>,
}

pub enum SubscriptionHandleResult {
  Added(Subscription),
  Removed(Subscription),
  Error(String),
}

impl Subscription {
  pub fn new(guild: GuildId, channel: ChannelId) -> Self {
    Self {
      guild,
      channel,
      last_message: None,
    }
  }

  pub async fn format_to_string(
    &self,
    ctx: &Context,
    subscribed: bool,
  ) -> String {
    let channel_name = self
      .channel
      .name(&ctx.cache)
      .await
      .unwrap_or_else(|| "this channel".to_string());

    let guild_name = self
      .guild
      .name(&ctx.cache)
      .unwrap_or_else(|| "this guild".to_string());

    let verb = match subscribed {
      true => "Subscribed to",
      false => "Unsubscribed from",
    };

    format!("{} {} in {}", verb, channel_name, guild_name)
  }

  pub async fn create_and_handle(
    ctx: &Context,
    guild_id: Option<GuildId>,
    channel_id: ChannelId,
  ) -> SubscriptionHandleResult {
    if let Some(guild) = guild_id {
      let subscription = Subscription::new(guild, channel_id);

      subscription.handle(ctx).await
    } else {
      SubscriptionHandleResult::Error("No GuildId provided".to_string())
    }
  }

  async fn handle(self, ctx: &Context) -> SubscriptionHandleResult {
    let db = {
      let data = ctx.data.read().await;
      if let Some(handle) = data.get::<Subscriptions>() {
        handle.clone()
      } else {
        return SubscriptionHandleResult::Error(
          "Database not in shared data".to_string(),
        );
      }
    };

    let deleted = match db.delete(&self).await {
      Ok(count) => {
        debug!("Removed {} subscriptions", count);
        count != 0
      }
      Err(e) => {
        return SubscriptionHandleResult::Error(format!(
          "Error removing: {}",
          e
        ))
      }
    };

    if deleted {
      return SubscriptionHandleResult::Removed(self.clone());
    }

    debug!(
      "Adding subscription for GuildID({}) in ChannelId({})",
      self.guild, self.channel
    );
    if let Err(e) = db.insert_one(self.clone()).await {
      return SubscriptionHandleResult::Error(format!(
        "Error inserting: {}",
        e
      ));
    };

    SubscriptionHandleResult::Added(self.clone())
  }

  pub async fn update(&self, ctx: &Context, new_subscription: Subscription) {
    let db = {
      let data = ctx.data.read().await;
      if let Some(handle) = data.get::<Subscriptions>() {
        handle.clone()
      } else {
        error!("Database not in shared data");
        return;
      }
    };

    if let Err(e) = db.delete(self).await {
      error!("Error deleting subscription for update{}", e);
      return;
    }
    match db.insert_one(new_subscription).await {
      Err(e) => error!("Error deleting subscription for update{}", e),
      Ok(d) => info!("Updated subscription for Guild({})", d.data.guild),
    }
  }
}

pub struct Subscriptions;

impl TypeMapKey for Subscriptions {
  type Value = Arc<RedDb<Ron, FileStorage<Ron>>>;
}
