use chrono::{DateTime, Datelike, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, str::FromStr};

pub const EXECUTION_MODEL_VERSION: &str = "EXECUTION_MODEL_V1";
pub const DEFAULT_MARKET: &str = "US_EQUITIES";
pub const DEFAULT_TIMEZONE: &str = "America/New_York";
pub const DEFAULT_SESSION_TYPE: &str = "REGULAR";
pub const MAX_DATASET_CANDLES: usize = 500_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSessionConfig {
    pub market: String,
    pub timezone: String,
    pub session_type: String,
    pub regular_open_minute: u16,
    pub regular_close_minute: u16,
}

impl MarketSessionConfig {
    pub fn new(market: &str, timezone: &str, session_type: &str) -> Result<Self, String> {
        Tz::from_str(timezone.trim()).map_err(|_| format!("timezone IANA inválido: {timezone}"))?;
        let session_type = session_type.trim().to_ascii_uppercase();
        if !matches!(session_type.as_str(), "DAILY" | "REGULAR") {
            return Err("session type inválido: use DAILY ou REGULAR".into());
        }
        Ok(Self {
            market: market.trim().to_ascii_uppercase(),
            timezone: timezone.trim().into(),
            session_type,
            regular_open_minute: 570,
            regular_close_minute: 960,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketTimeframe {
    #[serde(rename = "1D")]
    Daily,
    #[serde(rename = "15M")]
    FifteenMinutes,
    #[serde(rename = "1H")]
    OneHour,
    #[serde(rename = "30M")]
    ThirtyMinutes,
    #[serde(rename = "5M")]
    FiveMinutes,
    #[serde(rename = "1M")]
    OneMinute,
}

impl MarketTimeframe {
    pub fn parse_supported(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_uppercase().as_str() {
            "1D" | "D" | "DAILY" => Ok(Self::Daily),
            "15M" | "15MIN" => Ok(Self::FifteenMinutes),
            "1H" | "30M" | "5M" | "1M" => Err(format!(
                "timeframe {} está reservado, mas ainda não é suportado na v0.5",
                value.trim()
            )),
            _ => Err("timeframe inválido: use 1D ou 15M".into()),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Daily => "1D",
            Self::FifteenMinutes => "15M",
            Self::OneHour => "1H",
            Self::ThirtyMinutes => "30M",
            Self::FiveMinutes => "5M",
            Self::OneMinute => "1M",
        }
    }

    pub fn minutes(self) -> usize {
        match self {
            Self::Daily => 390,
            Self::FifteenMinutes => 15,
            Self::OneHour => 60,
            Self::ThirtyMinutes => 30,
            Self::FiveMinutes => 5,
            Self::OneMinute => 1,
        }
    }

    pub fn annualization_factor(self) -> f64 {
        match self {
            Self::Daily => 252.0,
            Self::FifteenMinutes => 252.0 * 26.0,
            _ => 252.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketTimestamp {
    pub utc: String,
    pub epoch_seconds: i64,
    pub session_id: String,
    pub session_state: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetContinuity {
    pub session_count: usize,
    pub expected_gap_count: usize,
    pub unexpected_gap_count: usize,
}

fn parse_datetime(value: &str, timezone: Tz) -> Result<DateTime<Utc>, String> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Ok(parsed.with_timezone(&Utc));
    }
    for format in [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
    ] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(value, format) {
            return match timezone.from_local_datetime(&naive) {
                LocalResult::Single(value) => Ok(value.with_timezone(&Utc)),
                LocalResult::Ambiguous(_, _) => {
                    Err(format!("timestamp local ambíguo por DST: {value}"))
                }
                LocalResult::None => Err(format!("timestamp local inexistente por DST: {value}")),
            };
        }
    }
    Err(format!("data/timestamp inválido: {value}"))
}

pub fn normalize_timestamp(
    value: &str,
    timeframe: MarketTimeframe,
    timezone_name: &str,
) -> Result<MarketTimestamp, String> {
    let value = value.trim();
    let timezone = Tz::from_str(timezone_name.trim())
        .map_err(|_| format!("timezone IANA inválido: {timezone_name}"))?;
    let utc = if !value.is_empty() && value.chars().all(|character| character.is_ascii_digit()) {
        let raw = value
            .parse::<i64>()
            .map_err(|_| format!("timestamp numérico inválido: {value}"))?;
        let seconds = if raw > 10_000_000_000 {
            raw / 1_000
        } else {
            raw
        };
        DateTime::<Utc>::from_timestamp(seconds, 0)
            .ok_or_else(|| format!("timestamp numérico inválido: {value}"))?
    } else if timeframe == MarketTimeframe::Daily {
        if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
            DateTime::<Utc>::from_naive_utc_and_offset(
                date.and_hms_opt(0, 0, 0).ok_or("data inválida")?,
                Utc,
            )
        } else {
            parse_datetime(value, timezone)?
        }
    } else {
        parse_datetime(value, timezone)?
    };
    let local = utc.with_timezone(&timezone);
    let minute = local.hour() * 60 + local.minute();
    let state = if timeframe == MarketTimeframe::Daily {
        "DAILY"
    } else if (570..960).contains(&minute) {
        "REGULAR"
    } else if minute < 570 {
        "PRE_MARKET"
    } else {
        "AFTER_HOURS"
    };
    let session_id = if timeframe == MarketTimeframe::Daily {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map(|date| date.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|_| {
                format!(
                    "{:04}-{:02}-{:02}",
                    local.year(),
                    local.month(),
                    local.day()
                )
            })
    } else {
        format!(
            "{:04}-{:02}-{:02}",
            local.year(),
            local.month(),
            local.day()
        )
    };
    Ok(MarketTimestamp {
        utc: utc.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        epoch_seconds: utc.timestamp(),
        session_id,
        session_state: state.into(),
    })
}

pub fn validate_continuity(
    timestamps: &[MarketTimestamp],
    timeframe: MarketTimeframe,
    session_type: &str,
) -> Result<DatasetContinuity, String> {
    if timestamps.is_empty() {
        return Err("dataset sem candles".into());
    }
    if timestamps.len() > MAX_DATASET_CANDLES {
        return Err(format!(
            "dataset excede o limite de {MAX_DATASET_CANDLES} candles"
        ));
    }
    let mut sessions = HashSet::new();
    let mut expected = 0;
    let mut unexpected = 0;
    let interval = timeframe.minutes() as i64 * 60;
    let regular_only = timeframe == MarketTimeframe::FifteenMinutes
        && session_type.eq_ignore_ascii_case("REGULAR");
    for (index, current) in timestamps.iter().enumerate() {
        sessions.insert(current.session_id.clone());
        if regular_only && current.session_state != "REGULAR" {
            return Err(format!(
                "candle {} está fora da sessão regular 09:30–16:00",
                index + 1
            ));
        }
        if let Some(previous) = index.checked_sub(1).map(|value| &timestamps[value]) {
            if current.epoch_seconds <= previous.epoch_seconds {
                return Err(format!(
                    "candle {} possui timestamp duplicado ou fora de ordem",
                    index + 1
                ));
            }
            if timeframe == MarketTimeframe::FifteenMinutes {
                if current.session_id == previous.session_id {
                    if current.epoch_seconds - previous.epoch_seconds != interval {
                        unexpected += 1;
                    }
                } else {
                    expected += 1;
                }
            } else if current.epoch_seconds - previous.epoch_seconds > interval {
                expected += 1;
            }
        }
    }
    if unexpected > 0 {
        return Err(format!(
            "dataset possui {unexpected} gap(s) intraday inesperado(s)"
        ));
    }
    Ok(DatasetContinuity {
        session_count: sessions.len(),
        expected_gap_count: expected,
        unexpected_gap_count: unexpected,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_daily_and_intraday_to_utc() {
        let daily =
            normalize_timestamp("2026-01-02", MarketTimeframe::Daily, DEFAULT_TIMEZONE).unwrap();
        assert_eq!(daily.utc, "2026-01-02T00:00:00Z");
        let intraday = normalize_timestamp(
            "2026-01-02 09:30",
            MarketTimeframe::FifteenMinutes,
            DEFAULT_TIMEZONE,
        )
        .unwrap();
        assert_eq!(intraday.utc, "2026-01-02T14:30:00Z");
        assert_eq!(intraday.session_state, "REGULAR");
    }

    #[test]
    fn distinguishes_expected_session_gap_from_intraday_gap() {
        let values = ["2026-01-02 15:45", "2026-01-05 09:30"].map(|value| {
            normalize_timestamp(value, MarketTimeframe::FifteenMinutes, DEFAULT_TIMEZONE).unwrap()
        });
        let result =
            validate_continuity(&values, MarketTimeframe::FifteenMinutes, "REGULAR").unwrap();
        assert_eq!(result.expected_gap_count, 1);
        let broken = ["2026-01-02 09:30", "2026-01-02 10:00"].map(|value| {
            normalize_timestamp(value, MarketTimeframe::FifteenMinutes, DEFAULT_TIMEZONE).unwrap()
        });
        assert!(validate_continuity(&broken, MarketTimeframe::FifteenMinutes, "REGULAR").is_err());
    }

    #[test]
    fn annualization_is_timeframe_aware() {
        assert_eq!(MarketTimeframe::Daily.annualization_factor(), 252.0);
        assert_eq!(
            MarketTimeframe::FifteenMinutes.annualization_factor(),
            6552.0
        );
    }

    #[test]
    fn validates_five_hundred_twenty_candles_across_twenty_sessions() {
        let dates = [
            "2026-01-05", "2026-01-06", "2026-01-07", "2026-01-08", "2026-01-09",
            "2026-01-12", "2026-01-13", "2026-01-14", "2026-01-15", "2026-01-16",
            "2026-01-20", "2026-01-21", "2026-01-22", "2026-01-23", "2026-01-26",
            "2026-01-27", "2026-01-28", "2026-01-29", "2026-01-30", "2026-02-02",
        ];
        let timestamps = dates
            .iter()
            .flat_map(|date| {
                (0..26).map(move |candle| {
                    let minutes = 570 + candle * 15;
                    normalize_timestamp(
                        &format!("{date} {:02}:{:02}", minutes / 60, minutes % 60),
                        MarketTimeframe::FifteenMinutes,
                        DEFAULT_TIMEZONE,
                    )
                    .unwrap()
                })
            })
            .collect::<Vec<_>>();
        let continuity =
            validate_continuity(&timestamps, MarketTimeframe::FifteenMinutes, "REGULAR").unwrap();

        assert_eq!(timestamps.len(), 520);
        assert_eq!(continuity.session_count, 20);
        assert_eq!(continuity.expected_gap_count, 19);
        assert_eq!(continuity.unexpected_gap_count, 0);
    }
}
