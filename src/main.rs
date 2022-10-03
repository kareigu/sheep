use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use config::Config;
use reddb::RonDb;
use serenity::model::application::interaction::Interaction;
use serenity::model::gateway::{Activity, Ready};
use serenity::model::prelude::command::Command;
use serenity::model::prelude::GuildId;
use serenity::{async_trait, prelude::*};
use tracing::{error, info, warn};

mod messages;
mod subscription;
mod utils;
use subscription::{Subscription, SubscriptionHandleResult, Subscriptions};

struct Handler {
  loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    let command = match interaction.clone().application_command() {
      Some(c) => c,
      None => {
        warn!("Unsupported interaction: {:?}", interaction);
        return;
      }
    };

    if let Err(e) = command.defer(&ctx.http).await {
      error!("Error deferring command: {}", e);
      return;
    }

    let handle_command = match command.data.name.as_str() {
      "subscribe" => {
        match Subscription::create_and_handle(&ctx, command.guild_id, command.channel_id).await {
          SubscriptionHandleResult::Error(e) => {
            error!(e);
            utils::text_response(&ctx, command, "Unable to subscribe").await
          }
          SubscriptionHandleResult::Removed(s) => {
            info!("Removed {:?}", s);
            utils::text_response(&ctx, command, s.format_to_string(&ctx, false).await).await
          }
          SubscriptionHandleResult::Added(s) => {
            info!("Added {:?}", s);
            utils::text_response(&ctx, command, s.format_to_string(&ctx, true).await).await
          }
        }
      }
      _ => utils::text_response(&ctx, command, "Unsupported").await,
    };

    if let Err(e) = handle_command {
      error!("Error handling command: {}", e);
      return;
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
    .await
    {
      Ok(c) => info!("Created command {:?}", c.name),
      Err(e) => error!("Error creating command: {}", e),
    }

    info!("{} running", ready.user.tag());
  }

  async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
    let ctx = Arc::new(ctx);

    if self.loop_running.load(Ordering::Relaxed) {
      return;
    }

    tokio::spawn(messages::message_task(ctx));
  }
}

#[tokio::main]
async fn main() {
  let format = tracing_subscriber::fmt::format()
    .with_source_location(false)
    .with_file(false)
    .with_thread_names(false)
    .with_target(false)
    .compact();

  tracing_subscriber::fmt().event_format(format).init();

  let config = Config::builder()
    .add_source(config::File::with_name("sheep"))
    .build()
    .expect("Couldn't load Config");

  let token = config.get_string("token").expect("No token in config");
  let intents = GatewayIntents::non_privileged()
    | GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::MESSAGE_CONTENT;

  let mut client = Client::builder(token, intents)
    .event_handler(Handler {
      loop_running: AtomicBool::new(false),
    })
    .await
    .expect("Unable to create client");

  {
    let mut data = client.data.write().await;

    let db = RonDb::new::<Subscription>("subscriptions.db").expect("Couldn't create DB");
    data.insert::<Subscriptions>(Arc::new(db));
  }

  if let Err(e) = client.start().await {
    error!("Error while running client: {}", e);
  }
}
