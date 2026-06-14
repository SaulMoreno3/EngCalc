use crate::core::value::Value;

pub fn format_value(value: &Value) -> String {
    if let Some(ref s) = value.display_str {
        return s.clone();
    }

    if value.is_nan() {
        return "NaN".to_string();
    }

    if value.number().is_infinite() {
        if value.number() > 0.0 {
            return if let Some(unit_str) = value.get_unit_string() {
                format!("∞ {}", unit_str)
            } else {
                "∞".to_string()
            };
        } else {
            return if let Some(unit_str) = value.get_unit_string() {
                format!("-∞ {}", unit_str)
            } else {
                "-∞".to_string()
            };
        }
    }

    let num_str = format_number(value.number());

    if let Some(unit_str) = value.get_unit_string() {
        format!("{} {}", num_str, unit_str)
    } else {
        num_str
    }
}

fn format_number(n: f64) -> String {
    if n.is_nan() {
        return "NaN".to_string();
    }

    if n.is_infinite() {
        return if n > 0.0 {
            "∞".to_string()
        } else {
            "-∞".to_string()
        };
    }

    if n.fract() == 0.0 && n.abs() < 1e15 {
        if n.abs() >= 1e12 || (n.abs() < 1e-10 && n != 0.0) {
            format_dual_notation(n)
        } else {
            format!("{}", n as i64)
        }
    } else {
        let natural = format_natural(n);

        let abs = n.abs();
        if abs >= 1e12 || (abs < 1e-10 && abs != 0.0) || decimal_places(&natural) > 3 {
            format!("{} ({:.12e})", natural, n)
        } else {
            natural
        }
    }
}

fn format_dual_notation(n: f64) -> String {
    format!("{} ({:.12e})", format_natural(n), n)
}

fn format_natural(n: f64) -> String {
    trimmed_decimal(&format!("{:.12}", n))
}

fn decimal_places(s: &str) -> usize {
    s.split_once('.').map(|(_, decimals)| decimals.len()).unwrap_or(0)
}

fn trimmed_decimal(s: &str) -> String {
    if let Some(pos) = s.find('.') {
        let int_part = &s[..pos];
        let dec_part = &s[pos + 1..];

        // Find the last non-zero digit
        let trimmed_end = dec_part
            .char_indices()
            .rev()
            .find(|(_, c)| *c != '0')
            .map(|(i, _)| i + 1)
            .unwrap_or(0);

        if trimmed_end == 0 {
            int_part.to_string()
        } else {
            format!("{}.{}", int_part, &dec_part[..trimmed_end])
        }
    } else {
        s.to_string()
    }
}

pub fn format_error(msg: &str) -> String {
    format!("Error: {}", msg)
}

pub fn format_assignment(name: &str, value: &Value) -> String {
    format!("{} = {}", name, format_value(value))
}
