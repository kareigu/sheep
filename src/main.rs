use config::Config;
use serenity::{prelude::*, async_trait};
use serenity::model::{
  gateway::Ready,
  channel::Message,
};
use tracing::{info, debug, error};

struct Handler;

#[async_trait]
impl EventHandler for Handler {

  async fn message(&self, ctx: Context, msg: Message) {
    debug!("{:?}", msg);

    if msg.content == "kohta ne herää" {
      if let Err(e) = msg.channel_id.say(&ctx.http, "BÄÄ").await {
        error!("Error sending message: {}", e);
      }
    }
  }

  async fn ready(&self, _: Context, ready: Ready) {
    info!("{} running", ready.user.tag());
  }
}

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  let config = Config::builder()
    .add_source(config::File::with_name("sheep"))
    .build()
    .expect("Couldn't load Config");

  let token = config.get_string("token").expect("No token in config");
  let intents = 
    GatewayIntents::non_privileged() |
    GatewayIntents::GUILD_MESSAGES |
    GatewayIntents::MESSAGE_CONTENT;

  let mut client = Client::builder(token, intents)
    .event_handler(Handler)
    .await
    .expect("Unable to create client");


  if let Err(e) = client.start().await {
    error!("Error while running client: {}", e);
  }
}
