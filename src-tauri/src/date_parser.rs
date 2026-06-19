use chrono::{Local, DateTime, Duration, NaiveTime, Weekday, Datelike};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParsedReminder {
    pub title: String,
    pub original_text: String,
    pub due_at: String,
    pub confidence: f64,
    pub parsed_time_expression: Option<String>,
}

pub fn parse_reminder_text(text: &str) -> ParsedReminder {
    let trimmed = text.trim();
    let now = Local::now();

    let cleaned = trimmed
        .replace("RECUÉRDAME", "")
        .replace("recuérdame", "")
        .replace("RECORDAR", "")
        .replace("recordar", "")
        .replace("QUE", "")
        .replace("que", "")
        .trim()
        .to_string();

    let due_at = parse_datetime(&cleaned, now);
    let title = extract_title(&cleaned, &due_at.expression);

    ParsedReminder {
        title: title.trim().to_string(),
        original_text: trimmed.to_string(),
        due_at: due_at.datetime.format("%Y-%m-%dT%H:%M:%S").to_string(),
        confidence: due_at.confidence,
        parsed_time_expression: due_at.expression.clone(),
    }
}

struct ParsedDateTime {
    datetime: DateTime<Local>,
    confidence: f64,
    expression: Option<String>,
}

fn parse_datetime(text: &str, now: DateTime<Local>) -> ParsedDateTime {
    let lowercase = text.to_lowercase();

    if let Some(result) = try_parse_relative_minutes(&lowercase, now) {
        return result;
    }
    if let Some(result) = try_parse_relative_hours(&lowercase, now) {
        return result;
    }
    if let Some(result) = try_parse_tomorrow(&lowercase, now) {
        return result;
    }
    if let Some(result) = try_parse_today(&lowercase, now) {
        return result;
    }
    if let Some(result) = try_parse_weekday(&lowercase, now) {
        return result;
    }
    if let Some(result) = try_parse_relative_week(&lowercase, now) {
        return result;
    }
    if let Some(result) = try_parse_time_only(&lowercase, now) {
        return result;
    }

    ParsedDateTime {
        datetime: now + Duration::hours(1),
        confidence: 0.2,
        expression: None,
    }
}

fn try_parse_relative_minutes(text: &str, now: DateTime<Local>) -> Option<ParsedDateTime> {
    let re = Regex::new(r"(?:en|dentro de)\s+(\d+)\s*(?:minutos?|min)").ok()?;
    if let Some(caps) = re.captures(text) {
        let minutes: i64 = caps[1].parse().ok()?;
        let expr = caps[0].to_string();
        return Some(ParsedDateTime {
            datetime: now + Duration::minutes(minutes),
            confidence: 0.95,
            expression: Some(expr),
        });
    }
    let re_una = Regex::new(r"(?:en|dentro de)\s+una?\s*(?:hora|minuto)").ok()?;
    if let Some(caps) = re_una.captures(text) {
        let expr = caps[0].to_string();
        let is_hour = expr.contains("hora");
        return Some(ParsedDateTime {
            datetime: now + if is_hour { Duration::hours(1) } else { Duration::minutes(1) },
            confidence: 0.9,
            expression: Some(expr),
        });
    }
    None
}

fn try_parse_relative_hours(text: &str, now: DateTime<Local>) -> Option<ParsedDateTime> {
    let re = Regex::new(r"(?:en|dentro de)\s+(\d+)\s*(?:horas?|hrs?)").ok()?;
    if let Some(caps) = re.captures(text) {
        let hours: i64 = caps[1].parse().ok()?;
        let expr = caps[0].to_string();
        return Some(ParsedDateTime {
            datetime: now + Duration::hours(hours),
            confidence: 0.95,
            expression: Some(expr),
        });
    }
    None
}

fn try_parse_tomorrow(text: &str, now: DateTime<Local>) -> Option<ParsedDateTime> {
    if text.contains("mañana") {
        let tomorrow = now.date_naive() + Duration::days(1);
        let time = extract_time_from_text(text).unwrap_or(NaiveTime::from_hms_opt(9, 0, 0)?);
        let expr = Some("mañana".to_string());
        return Some(ParsedDateTime {
            datetime: tomorrow.and_time(time).and_local_timezone(Local).unwrap(),
            confidence: 0.85,
            expression: expr,
        });
    }
    None
}

fn try_parse_today(text: &str, now: DateTime<Local>) -> Option<ParsedDateTime> {
    if text.contains("hoy") {
        if let Some(time) = extract_time_from_text(text) {
            let expr = Some("hoy".to_string());
            return Some(ParsedDateTime {
                datetime: now.date_naive().and_time(time).and_local_timezone(Local).unwrap(),
                confidence: 0.85,
                expression: expr,
            });
        }
    }
    None
}

fn try_parse_weekday(text: &str, now: DateTime<Local>) -> Option<ParsedDateTime> {
    let days = vec![
        ("lunes", Weekday::Mon),
        ("martes", Weekday::Tue),
        ("miercoles", Weekday::Wed),
        ("miércoles", Weekday::Wed),
        ("jueves", Weekday::Thu),
        ("viernes", Weekday::Fri),
        ("sabado", Weekday::Sat),
        ("sábado", Weekday::Sat),
        ("domingo", Weekday::Sun),
    ];

    for (name, weekday) in &days {
        if text.contains(name) {
            let today = now.date_naive();
            let mut target = today;
            while target.weekday() != *weekday {
                target = target.succ_opt()?;
            }
            if target <= today {
                target = target.succ_opt()?;
                while target.weekday() != *weekday {
                    target = target.succ_opt()?;
                }
            }
            let time = extract_time_from_text(text).unwrap_or(NaiveTime::from_hms_opt(9, 0, 0)?);
            let expr = Some(name.to_string());
            return Some(ParsedDateTime {
                datetime: target.and_time(time).and_local_timezone(Local).unwrap(),
                confidence: 0.8,
                expression: expr,
            });
        }
    }
    None
}

fn try_parse_relative_week(text: &str, now: DateTime<Local>) -> Option<ParsedDateTime> {
    if text.contains("una semana") || text.contains("1 semana") || text.contains("en una semana") {
        return Some(ParsedDateTime {
            datetime: now + Duration::weeks(1),
            confidence: 0.9,
            expression: Some("una semana".to_string()),
        });
    }
    let re = Regex::new(r"(\d+)\s*(?:días?|dias?)").ok()?;
    if let Some(caps) = re.captures(text) {
        let days: i64 = caps[1].parse().ok()?;
        let expr = caps[0].to_string();
        return Some(ParsedDateTime {
            datetime: now + Duration::days(days),
            confidence: 0.85,
            expression: Some(expr),
        });
    }
    None
}

fn try_parse_time_only(text: &str, now: DateTime<Local>) -> Option<ParsedDateTime> {
    if let Some(time) = extract_time_from_text(text) {
        let today = now.date_naive();
        let target = if time <= now.time() {
            today.succ_opt()?.and_time(time).and_local_timezone(Local).unwrap()
        } else {
            today.and_time(time).and_local_timezone(Local).unwrap()
        };
        return Some(ParsedDateTime {
            datetime: target,
            confidence: 0.6,
            expression: Some(format!("a las {}", time.format("%H:%M"))),
        });
    }
    None
}

fn extract_time_from_text(text: &str) -> Option<NaiveTime> {
    let re = Regex::new(r"(?:a\s*las|alas)\s*(\d{1,2})(?::(\d{2}))?\s*(?:de\s*la\s*(mañana|tarde|noche))?").ok()?;
    if let Some(caps) = re.captures(text) {
        let hour: u32 = caps[1].parse().ok()?;
        let minute: u32 = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
        let is_pm = if let Some(period) = caps.get(3) {
            let p = period.as_str();
            p == "tarde" || p == "noche"
        } else {
            false
        };
        let hour_24 = if is_pm && hour < 12 {
            hour + 12
        } else if !is_pm && hour == 12 {
            0
        } else {
            hour
        };
        return NaiveTime::from_hms_opt(hour_24.min(23), minute.min(59), 0);
    }

    let re2 = Regex::new(r"(\d{1,2}):(\d{2})").ok()?;
    if let Some(caps) = re2.captures(text) {
        let hour: u32 = caps[1].parse().ok()?;
        let minute: u32 = caps[2].parse().ok()?;
        return NaiveTime::from_hms_opt(hour.min(23), minute.min(59), 0);
    }
    None
}

fn extract_title(text: &str, expression: &Option<String>) -> String {
    let title = match expression {
        Some(expr) => {
            text.replace(expr, "")
                .replace("a las", "")
                .trim()
                .to_string()
        }
        None => text.to_string(),
    };
    if title.is_empty() {
        "Recordatorio".to_string()
    } else {
        title
    }
}
