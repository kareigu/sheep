use crate::subscription::{Subscription, Subscriptions};
use serenity::prelude::Context;
use std::{future::Future, sync::Arc, time::Duration};
use tracing::{debug, error, info, warn};

pub fn create_message_task(ctx: Arc<Context>) -> impl Future<Output = ()> + Send + 'static {
  async move {
    loop {
      let db = {
        let data = ctx.data.read().await;
        data
          .get::<Subscriptions>()
          .expect("No Subscriptions in shared data")
          .clone()
      };

      let subscriptions = match db.find_all::<Subscription>().await {
        Err(e) => {
          error!("Couldn't find subscriptions: {}", e);
          tokio::time::sleep(Duration::from_secs(60)).await;
          continue;
        }
        Ok(d) => d,
      };

      for subscription in subscriptions {
        let channel = subscription.data.channel;

        if let Err(e) = channel
          .send_message(&ctx.http, |message| {
            message.embed(|embed| embed.title("BÄÄ 🐑").colour(0xFFFFFF))
          })
          .await
        {
          error!(
            "Error sending automatic message for ChannelId({}): {}",
            channel, e
          );
        }
      }

      tokio::time::sleep(Duration::from_secs(60)).await;
    }
  }
}
