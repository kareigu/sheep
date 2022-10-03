use std::sync::Arc;

use serde::{Serialize, Deserialize};
use serenity::model::prelude::{GuildId, ChannelId};
use serenity::prelude::{Context, TypeMapKey};
use reddb::{RedDb, FileStorage, serializer::Ron};
use tracing::{info, debug, error, warn};


#[derive(Clone, Serialize, PartialEq, Deserialize, Debug)]
pub struct Subscription {
  pub guild: GuildId,
  pub channel: ChannelId,
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
    }
  }

  pub async fn format_to_string(&self, ctx: &Context, subscribed: bool) -> String {
    let channel_name = self
      .channel
      .name(&ctx.cache)
      .await
      .unwrap_or("this channel".to_string());

    let guild_name = self
      .guild
      .name(&ctx.cache)
      .unwrap_or("this guild".to_string());

    let verb = match subscribed {
      true => "Subscribed to",
      false => "Unsubscribed from",
    };

    format!("{} {} in {}",
      verb,
      channel_name,
      guild_name
    )
  }

  pub async fn create_and_handle(ctx: &Context, guild_id: Option<GuildId>, channel_id: ChannelId) -> SubscriptionHandleResult {
    if let Some(guild) = guild_id {
      let subscription = Subscription::new(guild, channel_id);
      
      subscription.handle(&ctx).await
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
        return SubscriptionHandleResult::Error("Database not in shared data".to_string())
      }
    };


    let deleted = match db.delete(&self).await {
      Ok(count) => {
        debug!("Removed {} subscriptions", count);
        count != 0
      },
      Err(e) => {
        return SubscriptionHandleResult::Error(format!("Error removing: {}", e))
      },
    };

    if deleted {
      return SubscriptionHandleResult::Removed(self.clone())
    }


    debug!("Adding subscription for GuildID({}) in ChannelId({})", 
      self.guild, 
      self.channel
    );
    if let Err(e) = db.insert_one(self.clone()).await {
      return SubscriptionHandleResult::Error(format!("Error inserting: {}", e));
    };

    SubscriptionHandleResult::Added(self.clone())
  }
}

pub struct Subscriptions;


impl TypeMapKey for Subscriptions {
  type Value = Arc<RedDb<Ron, FileStorage<Ron>>>;
}