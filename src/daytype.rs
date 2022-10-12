use std::fmt::Display;

use chrono::{DateTime, Datelike, NaiveTime, TimeZone, Timelike, Weekday};
use serde::{Deserialize, Serialize};

pub enum DateConversionError {
  NotSpecialTime,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Time {
  Morning,
  MidDay,
  AfterWork,
  Evening,
}

impl TryFrom<NaiveTime> for Time {
  type Error = DateConversionError;

  fn try_from(time: NaiveTime) -> Result<Self, Self::Error> {
    match (time.hour(), time.minute()) {
      (6, 27..=33) => Ok(Time::Morning),
      (11..=14, 0..=59) => Ok(Time::MidDay),
      (17, 00..=15) => Ok(Time::AfterWork),
      (22..=23, 0..=59) => Ok(Time::Evening),
      _ => Err(DateConversionError::NotSpecialTime),
    }
  }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum DayType {
  WorkDay(Time),
  Friday(Time),
  Saturday(Time),
  Sunday(Time),
}

impl<T> TryFrom<DateTime<T>> for DayType
where
  T: TimeZone,
{
  type Error = DateConversionError;

  fn try_from(date: DateTime<T>) -> Result<Self, Self::Error> {
    let time = Time::try_from(date.time())?;

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
