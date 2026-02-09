use std::{
    fs::DirEntry,
    io::Result,
    os::unix::fs::PermissionsExt,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::cmd::root::Opts;

use ansi_term::Colour::Red;

pub fn format_permissions(mode: u32, is_dir: bool) -> String {
    let mut perms = String::with_capacity(10); // [drwxrwxrwx]

    perms.push(if is_dir { 'd' } else { '-' });

    // Owner permissions
    perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    // Group permissions
    perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o010 != 0 { 'x' } else { '-' });

    // Other permissions
    perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o001 != 0 { 'x' } else { '-' });

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

pub fn format_entry_line(
    entry: &DirEntry,
    opts: &Opts,
    indent_state: &[bool],
    is_last: bool,
    highlight: bool,
    first_ancestor_matched: usize,
) -> Result<String> {
    let path = entry.path();
    let mut line = String::new();
    let metadata = entry.metadata()?;
    let file_type = metadata.file_type();

    if opts.print_permissions {
        let mode = metadata.permissions().mode();
        let perms_str = format_permissions(mode, file_type.is_dir());
        line.push_str(&perms_str);
        line.push(' ');
    }

    if opts.last_modify {
        match metadata.modified() {
            Ok(mod_time) => {
                let date_str = format!("[{}] ", format_date(mod_time));
                line.push_str(&date_str);
            }
            Err(e) => {
                eprintln!(
                    "Warning: Could not get modification date for {:?}: {}",
                    entry.path(),
                    e
                );
            }
        }
    }

    if !opts.no_indent && !indent_state.is_empty() {
        for (indent_level, &is_parent_last) in indent_state.iter().enumerate() {
            if is_parent_last {
                line.push_str("    ");
            } else {
                let vertical_line = if opts.ascii { "|   " } else { "│   " };
                if first_ancestor_matched < indent_level {
                    line.push_str(&Red.paint(vertical_line).to_string());
                } else {
                    line.push_str(vertical_line);
                }
            }
        }
    }

    let line_prefix = match (opts.no_indent, is_last, opts.ascii) {
        (true, _, _) => "",
        (false, true, true) => "+---",
        (false, true, false) => "└── ",
        (false, false, true) => "|---",
        (false, false, false) => "├── ",
    };

    if first_ancestor_matched < indent_state.len() {
        line.push_str(&Red.paint(line_prefix).to_string());
    } else {
        line.push_str(line_prefix);
    }

    let mut display_path = String::new();
    if file_type.is_dir() {
        display_path.push_str(" ");
    } else if let Some(ext) = path.extension().and_then(|p| p.to_str()) {
        match ext {
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico" | "tiff" | "webp" | "bmp" => {
                display_path.push_str("󰈟 ")
            }
            "mp3" | "wav" | "flac" | "aac" | "ogg" => display_path.push_str("󰈣 "),
            "mp4" | "avi" | "mov" | "wmv" | "flv" | "webm" | "mkv" => display_path.push_str("󰈫 "),
            "zip" | "rar" | "tar" | "7z" | "gz" | "xz" => display_path.push_str(" "),
            "md" | "txt" | "xml" | "yml" | "yaml" => display_path.push_str("󰈙 "),
            "lock" | "key" | "pem" | "crt" | "p12" | "pfx" => display_path.push_str("󱆄 "),
            "toml" | "ini" | "cfg" | "conf" => display_path.push_str("󱁻 "),
            "json" | "csv" | "log" | "sql" => display_path.push_str("󰱾 "),
            &_ => display_path.push_str("󰈔 "),
        }
    } else {
        display_path.push_str("󰈔 ");
    }

    if opts.full_path {
        display_path.push_str(&path.display().to_string());
    } else {
        display_path.push_str(entry.file_name().to_string_lossy().as_ref());
    };

    if highlight {
        line.push_str(&Red.bold().paint(display_path).to_string());
    } else {
        line.push_str(&display_path);
    }

    if !file_type.is_dir() && opts.print_size {
        let size = metadata.len();
        let size_str = format!(" ({})", format_file_size(size));
        line.push_str(&size_str);
    }

    Ok(line)
}
