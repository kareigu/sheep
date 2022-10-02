use std::sync::Arc;

use config::Config;
use serenity::model::prelude::{GuildId, ChannelId};
use serenity::{prelude::*, async_trait};
use serenity::model::{
  gateway::Ready,
  channel::Message,
};
use tracing::{info, debug, error};
use reddb::RonDb;
use serde::{Deserialize, Serialize};

struct Handler;

#[async_trait]
impl EventHandler for Handler {

  async fn message(&self, ctx: Context, msg: Message) {
    debug!("{:?}", msg);

    if msg.content == "kohta ne herää" {
      match Subscription::create_and_handle(&ctx, &msg).await {
        SubscriptionHandleResult::Error(e) => error!(e),
        SubscriptionHandleResult::Added(s) => info!("Added {:?}", s),
        SubscriptionHandleResult::Removed(s) => info!("Removed {:?}", s),
      }

      if let Err(e) = msg.channel_id.say(&ctx.http, "BÄÄ").await {
        error!("Error sending message: {}", e);
      }
    }
  }

  async fn ready(&self, _: Context, ready: Ready) {
    info!("{} running", ready.user.tag());
  }
}

#[derive(Clone, Serialize, PartialEq, Deserialize, Debug)]
struct Subscription {
  pub guild: GuildId,
  pub channel: ChannelId,
}

enum SubscriptionHandleResult {
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

  pub async fn create_and_handle(ctx: &Context, msg: &Message) -> SubscriptionHandleResult {
    if let Some(guild) = msg.guild_id {
      let subscription = Subscription::new(guild, msg.channel_id);
      
      subscription.handle(&ctx).await
    } else {
      SubscriptionHandleResult::Error("No GuildId provided".to_string())
    }
  }
  
  async fn handle(self, ctx: &Context) -> SubscriptionHandleResult {
    let db_handle = { 
      let data = ctx.data.read().await;
      if let Some(handle) = data.get::<Subscriptions>() {
        handle.clone()
      } else {
        return SubscriptionHandleResult::Error("Database not in shared data".to_string())
      }
    };
  
    let db = db_handle
      .read()
      .await;


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

struct Subscriptions;


impl TypeMapKey for Subscriptions {
  type Value = Arc<
    RwLock<
      reddb::RedDb<
        reddb::serializer::Ron, 
        reddb::FileStorage<reddb::serializer::Ron>
      >
    >
  >;
}

#[tokio::main]
async fn main() {
  let format = tracing_subscriber::fmt::format()
    .with_source_location(false)
    .with_file(false)
    .with_thread_names(false)
    .with_target(false)
    .compact();

  tracing_subscriber::fmt()
    .event_format(format)
    .init();

  let config = Config::builder()
    .add_source(config::File::with_name("sheep"))
    .build()
    .expect("Couldn't load Config");

  let token = config
    .get_string("token")
    .expect("No token in config");
  let intents = 
    GatewayIntents::non_privileged() |
    GatewayIntents::GUILD_MESSAGES |
    GatewayIntents::MESSAGE_CONTENT;



  let mut client = Client::builder(token, intents)
    .event_handler(Handler)
    .await
    .expect("Unable to create client");

  {
    let mut data = client.data.write().await;

    
    let db = RonDb::new::<Subscription>("subscriptions.db")
      .expect("Couldn't create DB");
    data.insert::<Subscriptions>(Arc::new(RwLock::new(db)));
  }


  if let Err(e) = client.start().await {
    error!("Error while running client: {}", e);
  }
}
