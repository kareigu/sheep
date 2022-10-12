use std::fmt::Display;

use chrono::{DateTime, Datelike, NaiveTime, TimeZone, Timelike, Weekday};
use serde::{Deserialize, Serialize};

pub enum DateConversionError {
  NotSpecialTime,
}

pub struct DateConversionReturn<T> {
  pub data: T,
  pub last_possible: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Time {
  Morning,
  MidDay,
  AfterWork,
  Evening,
}

impl Time {
  pub fn try_parse_from_time(
    time: NaiveTime,
  ) -> Result<DateConversionReturn<Self>, DateConversionError> {
    match (time.hour(), time.minute()) {
      (6, 27..=33) => Ok(DateConversionReturn {
        data: Time::Morning,
        last_possible: time.minute() == 33,
      }),
      (11..=14, 0..=59) => Ok(DateConversionReturn {
        data: Time::MidDay,
        last_possible: time.hour() == 14 && time.minute() == 59,
      }),
      (17, 00..=15) => Ok(DateConversionReturn {
        data: Time::AfterWork,
        last_possible: time.minute() == 15,
      }),
      (22..=23, 0..=59) => Ok(DateConversionReturn {
        data: Time::Evening,
        last_possible: time.hour() == 23 && time.minute() == 59,
      }),
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

impl DayType {
  pub fn try_parse_from_date<T>(
    date: DateTime<T>,
  ) -> Result<DateConversionReturn<Self>, DateConversionError>
  where
    T: TimeZone,
  {
    let time = Time::try_parse_from_time(date.time())?;

    match date.weekday() {
      Weekday::Fri => Ok(DateConversionReturn {
        data: DayType::Friday(time.data),
        last_possible: time.last_possible,
      }),
      Weekday::Sat => Ok(DateConversionReturn {
        data: DayType::Saturday(time.data),
        last_possible: time.last_possible,
      }),
      Weekday::Sun => Ok(DateConversionReturn {
        data: DayType::Sunday(time.data),
        last_possible: time.last_possible,
      }),
      _ => Ok(DateConversionReturn {
        data: DayType::WorkDay(time.data),
        last_possible: time.last_possible,
      }),
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
