use crate::subscription::{Subscription, Subscriptions};
use chrono::offset::FixedOffset;
use chrono::prelude::*;
use serenity::prelude::Context;
use std::{fmt::Display, sync::Arc};
use tokio::time::sleep;
use tracing::error;

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

enum DayTypeConversionError {
  NotSpecialTime,
}

impl<T> TryFrom<DateTime<T>> for DayType
where
  T: TimeZone,
{
  type Error = DayTypeConversionError;

  fn try_from(date: DateTime<T>) -> Result<Self, Self::Error> {
    use DayTypeConversionError::NotSpecialTime;

    if date.minute() != 0 {
      return Err(NotSpecialTime);
    }

    let time = match date.hour() {
      8 => Ok(Time::Morning),
      11 => Ok(Time::MidDay),
      16 => Ok(Time::AfterWork),
      22 => Ok(Time::Evening),
      _ => Err(NotSpecialTime),
    }?;

    match date.weekday() {
      Weekday::Fri => Ok(DayType::Friday(time)),
      Weekday::Sat => Ok(DayType::Saturday(time)),
      Weekday::Sun => Ok(DayType::Sunday(time)),
      _ => Ok(DayType::WorkDay(time)),
    }
  }
}

impl Display for DayType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    write!(f, "{}", str)
  }
}

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

    let subscriptions_fetch = tokio::spawn(async move { db.find_all::<Subscription>().await });

    let day_type = match DayType::try_from(now) {
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
        Ok(d) => d,
      },
    };

    for subscription in subscriptions {
      let channel = subscription.data.channel;

      if let Err(e) = channel
        .send_message(&ctx.http, |message| {
          message.embed(|embed| embed.title(&day_type).colour(0xFFFFFF))
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
