//! # numeral-format — format numbers with the numeral.js format-string DSL
//!
//! Format numbers using the familiar [numeral.js](http://numeraljs.com/) mini-language —
//! thousands separators, fixed/optional decimals, abbreviations, currency, percentages,
//! bytes, ordinals, forced signs, and parenthesized negatives:
//!
//! ```
//! use numeral_format::format;
//!
//! assert_eq!(format(1000.0, "0,0"), "1,000");
//! assert_eq!(format(1000.234, "0,0.00"), "1,000.23");
//! assert_eq!(format(1_234_567.0, "0.0a"), "1.2m");
//! assert_eq!(format(0.974_878, "0.000%"), "97.488%");
//! assert_eq!(format(-1000.0, "($0,0)"), "($1,000)");
//! assert_eq!(format(1000.0, "0b"), "1KB");
//! assert_eq!(format(2.0, "0o"), "2nd");
//! ```
//!
//! A faithful Rust port of the formatting half of the widely-used `numeral` npm package
//! (the default `en` locale), which has no Rust equivalent. **Zero dependencies.**
//!
//! ## Scope
//!
//! This crate ports number → string *formatting* with the default `en` locale. It does not
//! implement parsing formatted strings back to numbers (`unformat`), arithmetic helpers, or
//! alternate locales. `NaN` is treated as numeral's `null` (formatted as zero).

#![forbid(unsafe_code)]
#![doc(html_root_url = "https://docs.rs/numeral-format/0.1.0")]
// Index/length arithmetic here is over short format strings; conversions cannot overflow.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    // Exact float comparisons here intentionally mirror numeral.js's `===` checks.
    clippy::float_cmp
)]

// Compile-test the README's examples as part of `cargo test`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

// `en` locale.
const THOUSANDS: char = ',';
const DECIMAL: char = '.';
const ABBR_THOUSAND: &str = "k";
const ABBR_MILLION: &str = "m";
const ABBR_BILLION: &str = "b";
const ABBR_TRILLION: &str = "t";
const CURRENCY: &str = "$";

/// Format `value` using a numeral.js format string (default `"0,0"` when empty).
///
/// `NaN` is treated as numeral's `null` and formats as zero.
///
/// ```
/// # use numeral_format::format;
/// assert_eq!(format(1234.5678, "0,0.00"), "1,234.57");
/// assert_eq!(format(f64::NAN, "0,0"), "0");
/// ```
#[must_use]
pub fn format(value: f64, format: &str) -> String {
    let fmt = if format.is_empty() { "0,0" } else { format };
    // numeral treats null (here: NaN) as a value; `value || 0` later coerces it to 0.
    let value = if value.is_nan() { 0.0 } else { value };

    // Dispatch in numeral's registration order: bps, bytes, currency, exponential,
    // ordinal, percentage, time; default is the plain number format.
    if contains(fmt, "BPS") {
        format_bps(value, fmt)
    } else if has_bytes_token(fmt) {
        format_bytes(value, fmt)
    } else if contains(fmt, "$") {
        format_currency(value, fmt)
    } else if contains(fmt, "e+") || contains(fmt, "e-") {
        format_exponential(value, fmt)
    } else if contains(fmt, "o") {
        format_ordinal(value, fmt)
    } else if contains(fmt, "%") {
        format_percentage(value, fmt)
    } else if contains(fmt, ":") {
        format_time(value)
    } else {
        number_to_format(value, fmt)
    }
}

fn contains(haystack: &str, needle: &str) -> bool {
    haystack.contains(needle)
}

/// Matches numeral's bytes detector `/([0\s]i?b)/`.
fn has_bytes_token(fmt: &str) -> bool {
    let bytes = fmt.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'0' || bytes[i].is_ascii_whitespace() {
            let mut j = i + 1;
            if j < bytes.len() && bytes[j] == b'i' {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'b' {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// The core "separators, decimals, signs, abbreviations" formatter.
#[allow(clippy::too_many_lines)]
fn number_to_format(value: f64, format: &str) -> String {
    let mut value = value;
    let mut format = String::from(format);
    let abs = value.abs();
    let mut neg_paren = false;
    // `signed`: Some(0) => prefix sign, Some(n>0) => suffix sign, None => no forced sign.
    let mut signed: Option<usize> = None;

    if format.contains('(') {
        neg_paren = true;
        format = format.replace(['(', ')'], "");
    } else if format.contains('+') || format.contains('-') {
        signed = if format.contains('+') {
            format.find('+')
        } else if value < 0.0 {
            format.find('-')
        } else {
            None
        };
        format = format.replace(['+', '-'], "");
    }

    // Abbreviation.
    let mut abbr = String::new();
    let mut abbr_forced = false;
    if format.contains('a') {
        let abbr_force = match_abbr_force(&format);
        abbr_forced = abbr_force.is_some();
        if format.contains(" a") {
            abbr.push(' ');
        }
        format = remove_abbr_token(&format, abbr.starts_with(' '));

        let trillion = 1e12;
        let billion = 1e9;
        let million = 1e6;
        let thousand = 1e3;
        match abbr_force {
            Some('t') => {
                abbr.push_str(ABBR_TRILLION);
                value /= trillion;
            }
            Some('b') => {
                abbr.push_str(ABBR_BILLION);
                value /= billion;
            }
            Some('m') => {
                abbr.push_str(ABBR_MILLION);
                value /= million;
            }
            Some('k') => {
                abbr.push_str(ABBR_THOUSAND);
                value /= thousand;
            }
            _ => {
                if abs >= trillion {
                    abbr.push_str(ABBR_TRILLION);
                    value /= trillion;
                } else if abs >= billion {
                    abbr.push_str(ABBR_BILLION);
                    value /= billion;
                } else if abs >= million {
                    abbr.push_str(ABBR_MILLION);
                    value /= million;
                } else if abs >= thousand {
                    abbr.push_str(ABBR_THOUSAND);
                    value /= thousand;
                }
            }
        }
    }

    // Optional decimals `[.]`.
    let mut opt_dec = false;
    if format.contains("[.]") {
        opt_dec = true;
        format = format.replace("[.]", ".");
    }

    // `int_part` is assigned in both arms of the precision branch below (numeral computes an
    // initial value here too, but always overwrites it).
    let mut int_part: String;

    let format_int = format.split('.').next().unwrap_or("");
    let precision = format.split('.').nth(1);
    let thousands_pos = format.find(',');
    let leading_count = format_int
        .split(',')
        .next()
        .unwrap_or("")
        .bytes()
        .filter(|&b| b == b'0')
        .count();

    let mut decimal = String::new();
    if let Some(precision) = precision.filter(|p| !p.is_empty()) {
        if precision.contains('[') {
            let cleaned = precision.replace(']', "");
            let mut split = cleaned.split('[');
            let req = split.next().unwrap_or("");
            let opt = split.next().unwrap_or("");
            let fixed = to_fixed(value, req.len() + opt.len(), opt.len());
            int_part = fixed.split('.').next().unwrap_or("").to_string();
            decimal = fixed_decimal(&fixed);
        } else {
            let fixed = to_fixed(value, precision.len(), 0);
            int_part = fixed.split('.').next().unwrap_or("").to_string();
            decimal = fixed_decimal(&fixed);
        }
        if opt_dec && parse_num(decimal.get(1..).unwrap_or("")) == 0.0 {
            decimal.clear();
        }
    } else {
        int_part = to_fixed(value, 0, 0);
    }

    // Re-check abbreviation after rounding (e.g. 999_999 → "1,000k" becomes "1m").
    if !abbr.is_empty()
        && !abbr_forced
        && parse_num(&int_part) >= 1000.0
        && abbr.trim() != ABBR_TRILLION
    {
        int_part = js_num_to_string(parse_num(&int_part) / 1000.0);
        let trimmed = abbr.trim().to_string();
        let space = if abbr.starts_with(' ') { " " } else { "" };
        abbr = match trimmed.as_str() {
            ABBR_THOUSAND => format!("{space}{ABBR_MILLION}"),
            ABBR_MILLION => format!("{space}{ABBR_BILLION}"),
            ABBR_BILLION => format!("{space}{ABBR_TRILLION}"),
            _ => abbr,
        };
    }

    let mut neg = false;
    if int_part.contains('-') {
        int_part = int_part.trim_start_matches('-').to_string();
        neg = true;
    }

    if int_part.len() < leading_count {
        let pad = leading_count - int_part.len();
        let mut padded = String::with_capacity(leading_count);
        for _ in 0..pad {
            padded.push('0');
        }
        padded.push_str(&int_part);
        int_part = padded;
    }

    if thousands_pos.is_some() {
        int_part = insert_thousands(&int_part);
    }

    if format.starts_with('.') {
        int_part.clear();
    }

    let mut output = String::new();
    output.push_str(&int_part);
    output.push_str(&decimal);
    output.push_str(&abbr);

    if neg_paren {
        if neg {
            return format!("({output})");
        }
        output
    } else {
        match signed {
            Some(0) => {
                let sign = if neg { '-' } else { '+' };
                format!("{sign}{output}")
            }
            Some(_) => {
                let sign = if neg { '-' } else { '+' };
                format!("{output}{sign}")
            }
            None => {
                if neg {
                    format!("-{output}")
                } else {
                    output
                }
            }
        }
    }
}

/// Find the abbreviation force letter from `/a(k|m|b|t)?/`.
fn match_abbr_force(format: &str) -> Option<char> {
    let bytes = format.as_bytes();
    let pos = format.find('a')?;
    bytes.get(pos + 1).and_then(|&b| match b {
        b'k' | b'm' | b'b' | b't' => Some(b as char),
        _ => None,
    })
}

/// Remove the abbreviation token `(' ')?a[kmbt]?` (first occurrence).
fn remove_abbr_token(format: &str, leading_space: bool) -> String {
    let bytes = format.as_bytes();
    let prefix = if leading_space { " a" } else { "a" };
    if let Some(pos) = format.find(prefix) {
        let mut end = pos + prefix.len();
        if let Some(&b) = bytes.get(end) {
            if matches!(b, b'k' | b'm' | b'b' | b't') {
                end += 1;
            }
        }
        let mut out = String::with_capacity(format.len());
        out.push_str(&format[..pos]);
        out.push_str(&format[end..]);
        out
    } else {
        format.to_string()
    }
}

/// numeral's `toFixed`: decimal-aware rounding that avoids binary float surprises.
fn to_fixed(value: f64, max_decimals: usize, optionals: usize) -> String {
    let s = js_num_to_string(value);
    let min_decimals = max_decimals.saturating_sub(optionals);
    let bounded = if let Some(frac) = s.split('.').nth(1) {
        frac.len().max(min_decimals).min(max_decimals)
    } else {
        min_decimals
    };

    let power = 10f64.powi(bounded as i32);
    let scaled: f64 = format!("{s}e{bounded}").parse().unwrap_or(f64::NAN);
    let rounded = js_round(scaled);
    let result = rounded / power;
    let mut output = format_fixed(result, bounded);

    if optionals > max_decimals - bounded {
        let n = optionals - (max_decimals - bounded);
        output = strip_trailing_optional_zeros(&output, n);
    }
    output
}

/// `Math.round`: round half toward +∞.
fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// `Number.prototype.toFixed(d)` for an already-rounded value.
fn format_fixed(value: f64, decimals: usize) -> String {
    // Guard against `-0.00`-style output: JS toFixed renders -0 as "0.00".
    let value = if value == 0.0 { 0.0 } else { value };
    format!("{value:.decimals$}")
}

/// Remove `/\.?0{1,n}$/` from the end.
fn strip_trailing_optional_zeros(s: &str, n: usize) -> String {
    let bytes = s.as_bytes();
    let mut end = bytes.len();
    let mut removed = 0;
    while removed < n && end > 0 && bytes[end - 1] == b'0' {
        end -= 1;
        removed += 1;
    }
    if removed > 0 && end > 0 && bytes[end - 1] == b'.' {
        end -= 1;
    }
    s[..end].to_string()
}

/// The decimal portion of a `to_fixed` result, prefixed with the locale decimal mark.
fn fixed_decimal(fixed: &str) -> String {
    match fixed.split_once('.') {
        Some((_, frac)) => {
            let mut d = String::with_capacity(frac.len() + 1);
            d.push(DECIMAL);
            d.push_str(frac);
            d
        }
        None => String::new(),
    }
}

/// Insert thousands separators every three digits from the right.
fn insert_thousands(int_part: &str) -> String {
    let digits: Vec<u8> = int_part.bytes().collect();
    let len = digits.len();
    let mut out = String::with_capacity(len + len / 3);
    for (i, &d) in digits.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(THOUSANDS);
        }
        out.push(d as char);
    }
    out
}

/// `Number(str)` for already-numeric strings (returns 0.0 on failure, like our usage).
fn parse_num(s: &str) -> f64 {
    s.parse().unwrap_or(0.0)
}

/// `Number.prototype.toString()` for the magnitudes this crate targets.
fn js_num_to_string(value: f64) -> String {
    if value == 0.0 {
        // Covers -0.0 too (JS `String(-0)` is "0").
        return String::from("0");
    }
    value.to_string()
}

fn format_bytes(value: f64, format: &str) -> String {
    let binary = contains(format, "ib");
    let base: f64 = if binary { 1024.0 } else { 1000.0 };
    let suffixes: &[&str] = if binary {
        &["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"]
    } else {
        &["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"]
    };
    let mut suffix = if contains(format, " b") || contains(format, " ib") {
        String::from(" ")
    } else {
        String::new()
    };
    let format = remove_first_match_bytes(format);

    let mut value = value;
    for power in 0..=suffixes.len() {
        let min = base.powi(power as i32);
        let max = base.powi(power as i32 + 1);
        if value == 0.0 || (value >= min && value < max) {
            if let Some(s) = suffixes.get(power) {
                suffix.push_str(s);
            }
            if min > 0.0 {
                value /= min;
            }
            break;
        }
    }

    let mut out = number_to_format(value, &format);
    out.push_str(&suffix);
    out
}

/// Remove the first `\s?i?b` from a bytes format.
fn remove_first_match_bytes(format: &str) -> String {
    let bytes = format.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        let mut j = i;
        if bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j < bytes.len() && bytes[j] == b'i' {
            j += 1;
        }
        if j < bytes.len() && bytes[j] == b'b' {
            j += 1;
            let mut out = String::with_capacity(format.len());
            out.push_str(&format[..start]);
            out.push_str(&format[j..]);
            return out;
        }
        i += 1;
    }
    format.to_string()
}

fn format_currency(value: f64, format: &str) -> String {
    let before = leading_currency_symbols(format);
    let after = trailing_currency_symbols(format);
    let format = remove_first_currency(format);
    let mut output = number_to_format(value, &format);

    let mut before: String = before;
    let mut after: String = after;
    if value >= 0.0 {
        before = remove_first_of(&before, &['-', '(']);
        after = remove_first_of(&after, &['-', ')']);
    } else if !before.contains('-') && !before.contains('(') {
        before.insert(0, '-');
    }

    let before_chars: Vec<char> = before.chars().collect();
    for (i, &symbol) in before_chars.iter().enumerate() {
        match symbol {
            '$' => output = insert_at(&output, CURRENCY, i as isize),
            ' ' => output = insert_at(&output, " ", (i + CURRENCY.len() - 1) as isize),
            _ => {}
        }
    }

    let after_chars: Vec<char> = after.chars().collect();
    let alen = after_chars.len();
    for i in (0..alen).rev() {
        let symbol = after_chars[i];
        match symbol {
            '$' => {
                output = if i == alen - 1 {
                    let mut o = output;
                    o.push_str(CURRENCY);
                    o
                } else {
                    insert_at(&output, CURRENCY, -((alen - (1 + i)) as isize))
                };
            }
            ' ' => {
                output = if i == alen - 1 {
                    let mut o = output;
                    o.push(' ');
                    o
                } else {
                    insert_at(
                        &output,
                        " ",
                        -((alen - (1 + i) + CURRENCY.len() - 1) as isize),
                    )
                };
            }
            _ => {}
        }
    }
    output
}

fn leading_currency_symbols(format: &str) -> String {
    format
        .chars()
        .take_while(|c| matches!(c, '+' | '-' | '(' | ' ' | '$'))
        .collect()
}

fn trailing_currency_symbols(format: &str) -> String {
    let mut tail: Vec<char> = format
        .chars()
        .rev()
        .take_while(|c| matches!(c, '+' | '-' | ')' | ' ' | '$'))
        .collect();
    tail.reverse();
    tail.into_iter().collect()
}

fn remove_first_currency(format: &str) -> String {
    // Strip `\s?\$\s?` (first occurrence).
    if let Some(pos) = format.find('$') {
        let bytes = format.as_bytes();
        let mut start = pos;
        if start > 0 && bytes[start - 1].is_ascii_whitespace() {
            start -= 1;
        }
        let mut end = pos + 1;
        if end < bytes.len() && bytes[end].is_ascii_whitespace() {
            end += 1;
        }
        let mut out = String::with_capacity(format.len());
        out.push_str(&format[..start]);
        out.push_str(&format[end..]);
        out
    } else {
        format.to_string()
    }
}

fn remove_first_of(s: &str, chars: &[char]) -> String {
    if let Some(pos) = s.find(|c| chars.contains(&c)) {
        let mut out = String::with_capacity(s.len());
        out.push_str(&s[..pos]);
        out.push_str(&s[pos + 1..]);
        out
    } else {
        s.to_string()
    }
}

/// Insert `sub` into `s` at character index `at` (negative counts from the end), like
/// numeral's `insert` with JS `slice` semantics.
fn insert_at(s: &str, sub: &str, at: isize) -> String {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as isize;
    let idx = if at < 0 {
        (len + at).max(0)
    } else {
        at.min(len)
    } as usize;
    let mut out = String::with_capacity(s.len() + sub.len());
    out.extend(chars[..idx].iter());
    out.push_str(sub);
    out.extend(chars[idx..].iter());
    out
}

fn format_percentage(value: f64, format: &str) -> String {
    let space = if contains(format, " %") { " " } else { "" };
    let value = value * 100.0;
    let format = strip_optional_space_token(format, "%");
    let output = number_to_format(value, &format);
    insert_suffix_before_paren(&output, space, "%")
}

fn format_ordinal(value: f64, format: &str) -> String {
    let mut ordinal = if contains(format, " o") {
        String::from(" ")
    } else {
        String::new()
    };
    let format = remove_first_ordinal(format);
    ordinal.push_str(ordinal_suffix(value));
    let mut out = number_to_format(value, &format);
    out.push_str(&ordinal);
    out
}

fn remove_first_ordinal(format: &str) -> String {
    strip_optional_space_token(format, "o")
}

/// English ordinal suffix (the `en` locale `ordinal` function).
///
/// Mirrors numeral exactly: it operates on the raw (possibly fractional) value with strict
/// equality, so e.g. `1.052` yields `"th"` (since `1.052 != 1`).
fn ordinal_suffix(number: f64) -> &'static str {
    let b = number % 10.0;
    let tens = (number % 100.0 / 10.0).trunc() as i32;
    if tens == 1 {
        "th"
    } else if b == 1.0 {
        "st"
    } else if b == 2.0 {
        "nd"
    } else if b == 3.0 {
        "rd"
    } else {
        "th"
    }
}

fn format_exponential(value: f64, format: &str) -> String {
    let exponential = to_exponential(value);
    let mut parts = exponential.split('e');
    let mantissa: f64 = parts.next().unwrap_or("0").parse().unwrap_or(0.0);
    let exp = parts.next().unwrap_or("+0");
    let format = remove_first_exponential(format);
    let mut out = number_to_format(mantissa, &format);
    out.push('e');
    out.push_str(exp);
    out
}

/// `Number.prototype.toExponential()` (shortest), e.g. `12345` → `1.2345e+4`.
fn to_exponential(value: f64) -> String {
    if value == 0.0 {
        return String::from("0e+0");
    }
    // Rust's `{:e}` gives e.g. "1.2345e4"; reformat the exponent with an explicit sign.
    let s = format!("{value:e}");
    let (mantissa, exp) = s.split_once('e').unwrap_or((s.as_str(), "0"));
    let exp_num: i32 = exp.parse().unwrap_or(0);
    let sign = if exp_num < 0 { '-' } else { '+' };
    format!("{mantissa}e{sign}{}", exp_num.abs())
}

fn remove_first_exponential(format: &str) -> String {
    // Strip `e[+|-]0` (first occurrence).
    for marker in ["e+0", "e-0"] {
        if let Some(pos) = format.find(marker) {
            let mut out = String::with_capacity(format.len());
            out.push_str(&format[..pos]);
            out.push_str(&format[pos + marker.len()..]);
            return out;
        }
    }
    format.to_string()
}

fn format_time(value: f64) -> String {
    let hours = (value / 3600.0).floor();
    let minutes = ((value - hours * 3600.0) / 60.0).floor();
    let seconds = js_round(value - hours * 3600.0 - minutes * 60.0);
    let h = hours as i64;
    let m = minutes as i64;
    let s = seconds as i64;
    format!("{h}:{m:02}:{s:02}")
}

fn format_bps(value: f64, format: &str) -> String {
    let space = if contains(format, " BPS") { " " } else { "" };
    let format = strip_optional_space_token(format, "BPS");
    let output = number_to_format(value * 10000.0, &format);
    insert_suffix_before_paren(&output, space, "BPS")
}

/// Strip `\s?<token>` (first occurrence) from a format string.
fn strip_optional_space_token(format: &str, token: &str) -> String {
    if let Some(pos) = format.find(token) {
        let bytes = format.as_bytes();
        let start = if pos > 0 && bytes[pos - 1].is_ascii_whitespace() {
            pos - 1
        } else {
            pos
        };
        let mut out = String::with_capacity(format.len());
        out.push_str(&format[..start]);
        out.push_str(&format[pos + token.len()..]);
        out
    } else {
        format.to_string()
    }
}

/// Append `space + suffix`, inserting it before a trailing `)` if the output is
/// parenthesized (shared by the percentage and bps formats).
fn insert_suffix_before_paren(output: &str, space: &str, suffix: &str) -> String {
    if output.contains(')') {
        let mut chars: Vec<char> = output.chars().collect();
        let pos = chars.len().saturating_sub(1);
        for (k, c) in format!("{space}{suffix}").chars().enumerate() {
            chars.insert(pos + k, c);
        }
        chars.into_iter().collect()
    } else {
        format!("{output}{space}{suffix}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separators_and_decimals() {
        assert_eq!(format(1000.0, "0,0"), "1,000");
        assert_eq!(format(1000.234, "0,0.00"), "1,000.23");
        assert_eq!(format(1234.5678, "0,0"), "1,235");
        assert_eq!(format(0.0, "0.0000"), "0.0000");
        assert_eq!(format(-0.5, "0,0.00"), "-0.50");
    }

    #[test]
    fn optional_decimals() {
        assert_eq!(format(3.5, "0[.]0"), "3.5");
        assert_eq!(format(3.0, "0[.]0"), "3");
    }

    #[test]
    fn abbreviations() {
        assert_eq!(format(1_234_567.0, "0.0a"), "1.2m");
        assert_eq!(format(1234.0, "0.00a"), "1.23k");
        assert_eq!(format(1_000_000.0, "0.0a"), "1.0m");
        assert_eq!(format(1_230_974.0, "0.000a"), "1.231m");
    }

    #[test]
    fn signs_and_parens() {
        assert_eq!(format(1.5, "+0,0"), "+2");
        assert_eq!(format(-1000.0, "(0,0)"), "(1,000)");
        assert_eq!(format(-1.5, "0.0"), "-1.5");
    }

    #[test]
    fn currency() {
        assert_eq!(format(1000.234, "$0,0.00"), "$1,000.23");
        assert_eq!(format(-1000.0, "($0,0)"), "($1,000)");
        assert_eq!(format(1000.0, "0,0 $"), "1,000 $");
    }

    #[test]
    fn percentage() {
        assert_eq!(format(0.974_878, "0.000%"), "97.488%");
        assert_eq!(format(1.0, "0%"), "100%");
    }

    #[test]
    fn bytes() {
        assert_eq!(format(1000.0, "0b"), "1KB");
        assert_eq!(format(1024.0, "0ib"), "1KiB");
        assert_eq!(format(1052.0, "0.0b"), "1.1KB");
    }

    #[test]
    fn ordinal() {
        assert_eq!(format(1.0, "0o"), "1st");
        assert_eq!(format(2.0, "0o"), "2nd");
        assert_eq!(format(3.0, "0o"), "3rd");
        assert_eq!(format(11.0, "0o"), "11th");
        assert_eq!(format(21.0, "0o"), "21st");
    }

    #[test]
    fn nan_is_zero() {
        assert_eq!(format(f64::NAN, "0,0"), "0");
    }
}
