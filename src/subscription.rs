use std::sync::Arc;

use crate::daytype::DayType;
use reddb::{serializer::Ron, FileStorage, RedDb};
use reddb::{Document, Uuid};
use serde::{Deserialize, Serialize};
use serenity::model::prelude::{ChannelId, GuildId};
use serenity::prelude::{Context, TypeMapKey};
use tracing::{debug, error, info, warn};

#[derive(Clone, Serialize, Eq, PartialEq, Deserialize, Debug)]
pub struct Subscription {
  pub guild: GuildId,
  pub channel: ChannelId,
  pub last_message: Option<DayType>,
}

pub struct SubscriptionDoc(Document<Subscription>);

impl From<Document<Subscription>> for SubscriptionDoc {
  fn from(document: Document<Subscription>) -> Self {
    Self(document)
  }
}

impl SubscriptionDoc {
  pub async fn update(&self, ctx: &Context, last_message: Option<DayType>) {
    let db = {
      let data = ctx.data.read().await;
      if let Some(handle) = data.get::<Subscriptions>() {
        handle.clone()
      } else {
        error!("Database not in shared data");
        return;
      }
    };

    let new_subscription = Subscription {
      guild: self.guild(),
      channel: self.channel(),
      last_message,
    };

    let updated = match db.update_one(&self.id(), new_subscription).await {
      Ok(b) => b,
      Err(e) => {
        error!("Error deleting subscription for update{}", e);
        return;
      }
    };

    match updated {
      true => info!("Updated subscription for GuildId({})", self.guild()),
      false => warn!("Nothing to update for GuildId({})", self.guild()),
    }
  }

  pub fn id(&self) -> Uuid {
    self.0._id
  }

  pub fn last_message(&self) -> Option<DayType> {
    self.0.data.last_message
  }

  pub fn channel(&self) -> ChannelId {
    self.0.data.channel
  }

  pub fn guild(&self) -> GuildId {
    self.0.data.guild
  }
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
}

pub struct Subscriptions;

impl TypeMapKey for Subscriptions {
  type Value = Arc<RedDb<Ron, FileStorage<Ron>>>;
}
