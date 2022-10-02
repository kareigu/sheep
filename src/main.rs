use std::sync::Arc;

use config::Config;
use serenity::model::prelude::command::Command;
use serenity::model::prelude::{GuildId, ChannelId};
use serenity::model::application::interaction::Interaction;
use serenity::utils::Colour;
use serenity::{prelude::*, async_trait};
use serenity::model::{
  gateway::{Ready, Activity},
  channel::Message,
  application::interaction::InteractionResponseType,
  application::interaction::application_command::ApplicationCommandInteraction,
};
use serenity::Error;
use tracing::{info, debug, error, warn};
use reddb::RonDb;
use serde::{Deserialize, Serialize};

struct Handler;

pub async fn text_response<D>(ctx: &Context, command: ApplicationCommandInteraction, text: D) -> Result<(), Error>
where D: ToString, {
  let _a = command
    .edit_original_interaction_response(&ctx.http, |response| {
      response
        .embed(|embed| {
          embed
            .title(text)
            .colour(0xFFFFFF)
        })
    }).await?;
  Ok(())
}

#[async_trait]
impl EventHandler for Handler {
  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    let command = match interaction.clone().application_command() {
      Some(c) => c,
      None => {
        warn!("Unsupported interaction: {:?}", interaction);
        return
      },
    };

    if let Err(e) = command.defer(&ctx.http).await {
      error!("Error deferring command: {}", e);
      return
    }

    let handle_command = match command.data.name.as_str() {
      "subscribe" => {
        match Subscription::create_and_handle(&ctx, command.guild_id, command.channel_id).await {
          SubscriptionHandleResult::Error(e) => {
            error!(e);
            text_response(&ctx, command, "Unable to subscribe").await
          },
          SubscriptionHandleResult::Removed(s) => {
            info!("Removed {:?}", s);
            text_response(&ctx, command, s.format_to_string(&ctx, false).await).await
          },
          SubscriptionHandleResult::Added(s) => {
            info!("Added {:?}", s);
            text_response(&ctx, command, s.format_to_string(&ctx, true).await).await
          }
        }
      },
      _ => text_response(&ctx, command, "Unsupported").await,
    };
    
    if let Err(e) = handle_command {
      error!("Error handling command: {}", e);
      return
    }
  }

  async fn ready(&self, ctx: Context, ready: Ready) {
    let activity = Activity::playing("with pomon persekarvat");
    ctx.set_activity(activity).await;

    match Command::create_global_application_command(&ctx.http, |command| {
      command
        .name("subscribe")
        .name_localized("ja", "サブスクライブ")
        .name_localized("fi", "tilaa")
        .description("Toggle lamb's subscription to this channel")
        .description_localized("ja", "このチャッネルのサブスクリプチオンをトッグル")
        .description_localized("fi", "Tilaa/peru tilaus lampaasta tälle kanavalle")
    })
    .await {
      Ok(c) => info!("Created command {:?}", c),
      Err(e) => error!("Error creating command: {}", e),
    }

    

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

  async fn format_to_string(&self, ctx: &Context, subscribed: bool) -> String {
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
