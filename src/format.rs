use crate::i18n::Lang;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Unit {
    Auto,
    B,
    KB,
    MB,
    GB,
    TB,
}

impl Unit {
    pub fn label(self) -> &'static str {
        match self {
            Unit::Auto => "Auto",
            Unit::B => "B",
            Unit::KB => "KB",
            Unit::MB => "MB",
            Unit::GB => "GB",
            Unit::TB => "TB",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Unit::Auto => "auto",
            Unit::B => "b",
            Unit::KB => "kb",
            Unit::MB => "mb",
            Unit::GB => "gb",
            Unit::TB => "tb",
        }
    }

    pub fn from_key(key: &str) -> Self {
        match key {
            "b" => Unit::B,
            "kb" => Unit::KB,
            "mb" => Unit::MB,
            "gb" => Unit::GB,
            "tb" => Unit::TB,
            _ => Unit::Auto,
        }
    }

    pub fn divisor(self) -> f64 {
        match self {
            Unit::Auto | Unit::B => 1.0,
            Unit::KB => 1024.0,
            Unit::MB => 1024.0 * 1024.0,
            Unit::GB => 1024.0 * 1024.0 * 1024.0,
            Unit::TB => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        }
    }
}

fn group_digits(digits: &str, lang: Lang) -> String {
    let separator = if lang == Lang::Id { '.' } else { ',' };
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    let len = digits.len();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(separator);
        }
        out.push(ch);
    }
    out
}

pub fn format_count(value: u64, lang: Lang) -> String {
    group_digits(&value.to_string(), lang)
}

fn format_float(value: f64, decimals: u8, lang: Lang) -> String {
    let raw = format!("{:.*}", decimals as usize, value);
    let (int_part, frac_part) = match raw.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (raw.as_str(), None),
    };
    let grouped = group_digits(int_part, lang);
    match frac_part {
        Some(frac) if decimals > 0 => {
            let sep = if lang == Lang::Id { ',' } else { '.' };
            format!("{grouped}{sep}{frac}")
        }
        _ => grouped,
    }
}

fn auto_unit(bytes: u64) -> Unit {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;
    if bytes >= TB {
        Unit::TB
    } else if bytes >= GB {
        Unit::GB
    } else if bytes >= MB {
        Unit::MB
    } else if bytes >= KB {
        Unit::KB
    } else {
        Unit::B
    }
}

pub fn bytes_label(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Bytes",
        Lang::Id => "Byte",
    }
}

pub fn format_bytes(bytes: u64, unit: Unit, decimals: u8, lang: Lang) -> String {
    let unit = if unit == Unit::Auto {
        auto_unit(bytes)
    } else {
        unit
    };

    if unit == Unit::B {
        return format!("{} {}", format_count(bytes, lang), bytes_label(lang));
    }

    let value = bytes as f64 / unit.divisor();
    format!("{} {}", format_float(value, decimals, lang), unit.label())
}

pub fn format_percent(percent: f64, lang: Lang) -> String {
    format!("{} %", format_float(percent, 1, lang))
}

pub fn format_duration(seconds: f64) -> String {
    if seconds < 1.0 {
        format!("{:.0} ms", (seconds * 1000.0).max(1.0))
    } else if seconds < 60.0 {
        format!("{:.1} s", seconds)
    } else {
        let mins = (seconds / 60.0).floor() as u64;
        let secs = seconds - mins as f64 * 60.0;
        format!("{}m {:.0}s", mins, secs)
    }
}
