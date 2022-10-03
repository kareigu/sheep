use crate::subscription::{Subscription, Subscriptions};
use chrono::offset::FixedOffset;
use chrono::prelude::*;
use serenity::prelude::Context;
use std::{future::Future, sync::Arc, time::Duration};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

enum Time {
  Morning,
  MidDay,
  AfterWork,
  Evening,
}

enum DayType {
  WorkDay(Time),
  Friday(Time),
  Saturday(Time),
  Sunday(Time),
}

impl DayType {
  pub fn from_date(date: DateTime<FixedOffset>) -> Option<Self> {
    let time = {
      if date.minute() != 0 {
        return None;
      }

      match date.hour() {
        8 => Some(Time::Morning),
        11 => Some(Time::MidDay),
        16 => Some(Time::AfterWork),
        22 => Some(Time::Evening),
        _ => None,
      }
    }?;

    match date.weekday() {
      Weekday::Fri => Some(DayType::Friday(time)),
      Weekday::Sat => Some(DayType::Saturday(time)),
      Weekday::Sun => Some(DayType::Sunday(time)),
      _ => Some(DayType::WorkDay(time)),
    }
  }

  pub fn to_string(self) -> String {
    use DayType::*;
    use Time::*;

    let str = match self {
      WorkDay(Morning) => "🏢🐑 Voi voi taas täytyy herätä imemään pomon perse karvoja,, BÄÄ BÄÄ",
      WorkDay(MidDay) => "💩🐑 Aika käyttää naapurin Ari-Jukan vinkkiä ja käydä paskalla niin minulle maksetaan paskomisesta RÄH HÄH",
      WorkDay(AfterWork) => "🏠🐑 Vihdoin pääsee kotiin niin ei tarvitse kusipää pomon olla nalkuttamassa",
      WorkDay(Evening) => "🛏️🐑 Kohtahan se pitää mennä nukkumaan,, taidanpa laittaa herätys kellon valmiiksi",
      Friday(Morning) => "🛏️🐑⏰ PIPIPI PIPIPI,,, saatanan herätyskello,, onneksi tänään on perjantai niin voi töiden jälkeen vetää pään tyhjäksi",
      Friday(MidDay) => "💩🐑 Taidanpa perjantain kunniaksi käydä erikois pitkällä paskalla",
      Friday(AfterWork) => "🏪🐑 Päästihän se pomo vihdoin lähtemään,, nyt äkkiä alkoon",
      Friday(Evening) => "🍺🐑 Vittu että on hyvä meno kun ei tarvitse huomenna herätä ja voin juoda koko yön",
      Saturday(Morning) => "🛏️🐑 Nythän se voisi olla aika mennä nukkumaan kun viinaksetkin on jo loppu",
      Saturday(MidDay) | Sunday(Morning) => "🛏️ Zzz",
      Saturday(AfterWork) => "🏪🐑 Voi vittu,, kello on jo noin paljon,, nyt äkkiä kauppaan hakemaan kaljat tälle päivälle",
      Saturday(Evening) => "🍺🐑 Aika lähteä baariin laulamaan karaokea ja juomaan paikka tyhjäksi",
      Sunday(MidDay) => "🐑 Vittu että on ihan hirveä krapula,, en kyllä juo enää ennen ensi kertaa RÄH HÄH",
      Sunday(AfterWork) => "🍕🐑 Olipa hyvä Grandiosan pakaste sipuli pizza tasaamaan oloa",
      Sunday(Evening) => "🛏️🐑 Oi voi,, taas pitää valmistautua nukkumaan että jaksaa huomenna leikkiä pomon perse karvoilla koko päivän"
    };
    str.to_string()
  }
}

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
        tokio::spawn(async move { return db.find_all::<Subscription>().await });

      let day_type = match DayType::from_date(now) {
        None => {
          sleep(sleep_for).await;
          continue;
        }
        Some(t) => t,
      };

      let message_content = day_type.to_string();

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
          Ok(d) => d,
        },
      };

      for subscription in subscriptions {
        let channel = subscription.data.channel;

        if let Err(e) = channel
          .send_message(&ctx.http, |message| {
            message.embed(|embed| embed.title(&message_content).colour(0xFFFFFF))
          })
          .await
        {
          error!(
            "Error sending automatic message for ChannelId({}): {}",
            channel, e
          );
        }
      }

      sleep(sleep_for).await;
    }
  }
}
