//! SI önekli değerlerin ayrıştırılması.
//!
//! Desteklenen yazımlar: `4.7k`, `4k7`, `4,7k`, `100n`, `2.2µ` / `2.2u`,
//! `1M` (mega), `1m` (mili), `1meg`, `1e-3`, `10kΩ`, `5V`, `50Hz`.
//! Önekler büyük/küçük harf duyarlıdır: `m` mili, `M` mega.

/// Ayrıştırılamayan değer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("geçersiz değer: {0:?}")]
pub struct ParseValueError(pub String);

/// Değerin sonunda yazılabilecek birimler (uzundan kısaya).
const UNITS: &[&str] = &["ohm", "Hz", "Ω", "°", "F", "H", "V", "A", "s", "W"];

/// Bir değer metnini sayıya çevirir.
pub fn parse_value(text: &str) -> Result<f64, ParseValueError> {
    let s = text.trim().replace(',', ".").replace('−', "-");
    let err = || ParseValueError(text.to_owned());
    if s.is_empty() {
        return Err(err());
    }
    if let Some(v) = parse_core(&s) {
        return Ok(v);
    }
    for unit in UNITS {
        if let Some(rest) = s.strip_suffix(unit)
            && let Some(v) = parse_core(rest.trim_end())
        {
            return Ok(v);
        }
    }
    Err(err())
}

fn prefix_multiplier(s: &str) -> Option<(f64, usize)> {
    if s.starts_with("meg") {
        return Some((1e6, 3));
    }
    let c = s.chars().next()?;
    let m = match c {
        'f' => 1e-15,
        'p' => 1e-12,
        'n' => 1e-9,
        'u' | 'µ' | 'μ' => 1e-6,
        'm' => 1e-3,
        'k' | 'K' => 1e3,
        'M' => 1e6,
        'G' => 1e9,
        'T' => 1e12,
        _ => return None,
    };
    Some((m, c.len_utf8()))
}

fn parse_core(s: &str) -> Option<f64> {
    let bytes = s.as_bytes();
    let mut i = 0;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    let int_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let mut has_point = false;
    if i < bytes.len() && bytes[i] == b'.' {
        has_point = true;
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    if i == int_start || (has_point && i == int_start + 1) {
        return None; // rakam yok
    }
    // Üs: 1e-3
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        let mut j = i + 1;
        if j < bytes.len() && (bytes[j] == b'+' || bytes[j] == b'-') {
            j += 1;
        }
        let exp_digits = j;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if j > exp_digits {
            i = j;
        }
    }
    let number: f64 = s[..i].parse().ok()?;
    let rest = &s[i..];
    if rest.is_empty() {
        return Some(number);
    }
    let (mult, len) = prefix_multiplier(rest)?;
    let tail = &rest[len..];
    if tail.is_empty() {
        return Some(number * mult);
    }
    // 4k7 yazımı: önekten sonra ondalık basamaklar
    if !has_point && tail.bytes().all(|b| b.is_ascii_digit()) {
        let combined: f64 = format!("{}.{}", &s[..i], tail).parse().ok()?;
        return Some(combined * mult);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-12 * b.abs().max(1e-30)
    }

    #[test]
    fn parses_common_forms() {
        let cases = [
            ("1k", 1e3),
            ("4k7", 4.7e3),
            ("4.7k", 4.7e3),
            ("4,7k", 4.7e3),
            ("100n", 100e-9),
            ("2.2µ", 2.2e-6),
            ("2.2u", 2.2e-6),
            ("1M", 1e6),
            ("1m", 1e-3),
            ("1meg", 1e6),
            ("1e-3", 1e-3),
            ("10kΩ", 10e3),
            ("5V", 5.0),
            ("50Hz", 50.0),
            ("100mA", 0.1),
            ("-12", -12.0),
            ("0", 0.0),
            ("90°", 90.0),
            ("1F", 1.0),
            ("1f", 1e-15),
            (" 3.3 V ", 3.3),
        ];
        for (text, expected) in cases {
            let v = parse_value(text).unwrap_or_else(|e| panic!("{text}: {e}"));
            assert!(close(v, expected), "{text}: {v} != {expected}");
        }
    }

    #[test]
    fn rejects_invalid() {
        for text in ["", "abc", "k", "1x", "1.2k3", "--1", "."] {
            assert!(parse_value(text).is_err(), "{text} kabul edilmemeliydi");
        }
    }
}
