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

    // Check if the number is effectively an integer
    if n.fract() == 0.0 && n.abs() < 1e15 {
        // Check for very large/small integers that should use scientific notation
        if n.abs() >= 1e12 || (n.abs() < 1e-10 && n != 0.0) {
            format!("{:.12e}", n)
        } else {
            format!("{}", n as i64)
        }
    } else {
        // For decimals, show up to 12 significant digits
        let formatted = format!("{:.12}", n);
        // Trim trailing zeros after decimal point
        let trimmed = trimmed_decimal(&formatted);

        // Check if scientific notation is better
        let abs = n.abs();
        if abs >= 1e12 || (abs < 1e-10 && abs != 0.0) {
            format!("{:.12e}", n)
        } else {
            trimmed
        }
    }
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
