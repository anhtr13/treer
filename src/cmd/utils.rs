use std::time::{SystemTime, UNIX_EPOCH};

pub fn format_permissions(mode: u32, is_dir: bool) -> String {
    let mut perms = String::with_capacity(10); // [drwxrwxrwx]

    perms.push(if is_dir { 'd' } else { '-' });

    // Owner permissions
    perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o100 != 0 { 'x' } else { '-' }); // Consider setuid/setgid later?

    // Group permissions
    perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o010 != 0 { 'x' } else { '-' }); // Consider setgid later?

    // Other permissions
    perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o001 != 0 { 'x' } else { '-' }); // Consider sticky bit later?

    format!("[{perms}]")
}

pub fn format_file_size(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let base: f64 = 1024.0;
    let i = (bytes as f64).log(base).floor() as usize;

    let i = if i < UNITS.len() { i } else { 0 };

    let size = bytes as f64 / base.powi(i as i32);

    if i == 0 {
        format!("{} {}", size as u64, UNITS[i])
    } else {
        format!("{:.1} {}", size, UNITS[i])
    }
}

pub fn format_date(time: SystemTime) -> String {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let time_parts = (
                (secs / 86400) % 36525, // days since epoch
                ((secs / 3600) % 24),   // hours
                ((secs / 60) % 60),     // minutes
                (secs % 60),            // seconds
            );

            // Start with Unix epoch (1970-01-01) and add days
            let mut year = 1970;
            let mut month = 1;
            let mut day = 1;
            let mut days_left = time_parts.0;

            // Simple date calculation - consider leap years, etc.
            while days_left > 0 {
                let days_in_year = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                    366
                } else {
                    365
                };
                if days_left >= days_in_year {
                    days_left -= days_in_year;
                    year += 1;
                } else {
                    let days_in_month = match month {
                        2 => {
                            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                                29
                            } else {
                                28
                            }
                        }
                        4 | 6 | 9 | 11 => 30,
                        _ => 31,
                    };

                    if days_left >= days_in_month {
                        days_left -= days_in_month;
                        month += 1;
                        if month > 12 {
                            month = 1;
                            year += 1;
                        }
                    } else {
                        day += days_left as u32;
                        days_left = 0;
                    }
                }
            }

            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                year, month, day, time_parts.1, time_parts.2, time_parts.3
            )
        }
        Err(_) => String::from("Unknown date"),
    }
}
