use comfy_table::{Cell, Color as TColor};
use console::style;

pub fn print_warning(msg: &str) {
    println!("  {} {}", style("!").yellow().bold(), style(msg).yellow());
}

pub fn print_error(msg: &str) {
    println!("  {} {}", style("✗").red().bold(), style(msg).red());
}

pub fn print_dim(msg: &str) {
    println!("  {}", style(msg).dim());
}

pub fn success_cell() -> Cell {
    Cell::new("●").fg(TColor::Green)
}

pub fn inactive_cell() -> Cell {
    Cell::new("○").fg(TColor::DarkGrey)
}

pub fn truncate_model(model_id: &str, max: usize) -> String {
    if model_id.len() > max {
        format!("{}...", &model_id[..max - 3])
    } else {
        model_id.to_string()
    }
}

pub fn yes_no_cell(value: bool) -> Cell {
    if value {
        Cell::new("yes").fg(TColor::Green)
    } else {
        Cell::new("no").fg(TColor::DarkGrey)
    }
}

pub fn dim_cell(value: &str) -> Cell {
    Cell::new(value).fg(TColor::DarkGrey)
}

pub fn cyan_cell(value: &str) -> Cell {
    Cell::new(value).fg(TColor::Cyan)
}

pub fn format_context(limit: Option<i64>) -> String {
    limit
        .map(|c| format!("{}k", c / 1000))
        .unwrap_or_else(|| "-".to_string())
}

pub fn format_human(n: i64) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let abs = n.unsigned_abs();
    if abs < 1_000 {
        n.to_string()
    } else if abs < 1_000_000 {
        let whole = abs / 1_000;
        let frac = (abs % 1_000) / 100;
        if frac == 0 {
            format!("{whole}.0k")
        } else {
            format!("{whole}.{frac}k")
        }
    } else {
        let whole = abs / 1_000_000;
        let frac = (abs % 1_000_000) / 100_000;
        if frac == 0 {
            format!("{whole}.0m")
        } else {
            format!("{whole}.{frac}m")
        }
    }
}

pub fn format_timestamp_millis(millis: i64, fmt: &str) -> String {
    chrono::DateTime::from_timestamp_millis(millis)
        .map(|dt| dt.with_timezone(&chrono::Local).format(fmt).to_string())
        .unwrap_or_else(|| "-".to_string())
}

pub fn format_timestamp_rfc3339(secs: i64) -> String {
    chrono::DateTime::from_timestamp(secs, 0)
        .map(|dt| dt.with_timezone(&chrono::Local).to_rfc3339())
        .unwrap_or_else(|| secs.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_timestamp_millis_valid() {
        let result = format_timestamp_millis(1_700_000_000_000, "%Y-%m-%d %H:%M");
        assert!(!result.is_empty());
        assert!(result.contains('-'));
        assert!(result.contains(':'));
    }

    #[test]
    fn test_format_timestamp_millis_epoch() {
        let result = format_timestamp_millis(0, "%Y-%m-%d");
        assert!(!result.is_empty());
        assert!(result.contains("1969") || result.contains("1970"));
    }

    #[test]
    fn test_format_timestamp_rfc3339_valid() {
        let result = format_timestamp_rfc3339(1_700_000_000);
        assert!(!result.is_empty());
        assert!(result.contains('T'));
    }

    #[test]
    fn test_format_timestamp_rfc3339_epoch() {
        let result = format_timestamp_rfc3339(0);
        assert!(!result.is_empty());
        assert!(result.contains("1969") || result.contains("1970"));
    }
}
