use std::{
    collections::HashSet,
    fs::{self, DirEntry},
    io::{self, BufWriter, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
    time::SystemTime,
};

use crate::cmd::{
    root::Opts,
    utils::{format_date, format_file_size, format_permissions},
};

struct EntryInfo {
    entry: DirEntry,
    last_modify: io::Result<SystemTime>,
}

fn add_entry_to_display_set(
    root: &DirEntry,
    opts: &Opts,
    depth: usize,
    ancestors_matched_pattern: bool,
    display_entries: &mut HashSet<String>,
) -> bool {
    let path = root.path();
    let name = path.file_name().and_then(|name| name.to_str());

    let is_hidden = name.map(|name| name.starts_with('.')).unwrap_or(false);
    if !opts.show_hidden && is_hidden {
        return false;
    }

    if opts.dir_only && !path.is_dir() {
        return false;
    }

    if let Some(max_level) = opts.level
        && depth > max_level as usize
    {
        return false;
    }

    for exclude_pattern in opts.exclude_patterns.iter() {
        if name.is_some_and(|name| exclude_pattern.matches(name)) {
            return false;
        }
    }

    let mut should_display = true;
    let mut matched_pattern = ancestors_matched_pattern;

    if let Some(pattern) = &opts.pattern {
        if !name.is_some_and(|name| pattern.matches(name)) {
            // if name is not match pattern but ancestors are matched pattern => still display
            should_display = ancestors_matched_pattern;
        } else {
            matched_pattern = true;
        }
    }

    if path.is_dir()
        && let read_dir = fs::read_dir(&path)
        && let Ok(reader) = read_dir
    {
        reader.filter_map(Result::ok).for_each(|dir| {
            // if descendants are matched pattern => still display
            should_display |=
                add_entry_to_display_set(&dir, opts, depth + 1, matched_pattern, display_entries);
        });
    }

    if should_display {
        display_entries.insert(path.display().to_string());
    }

    should_display
}

fn format_entry_line(
    entry: &DirEntry,
    opts: &Opts,
    indent_state: &[bool],
    is_last: bool,
) -> std::io::Result<String> {
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

    if !opts.no_indent && !indent_state.is_empty() {
        for &is_parent_last in indent_state.iter() {
            if is_parent_last {
                line.push_str("    ");
            } else {
                let vertical_line = if opts.ascii { "|   " } else { "│   " };
                line.push_str(vertical_line);
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
    line.push_str(line_prefix);

    if file_type.is_dir() {
        line.push_str(" ");
    } else {
        if let Some(ext) = path.extension().and_then(|p| p.to_str()) {
            match ext {
                "png" | "jpg" | "jpeg" | "gif" | "svg" | "ico" | "tiff" | "webp" | "bmp" => {
                    line.push_str("󰈟 ")
                }
                "mp3" | "wav" | "flac" | "aac" | "ogg" => line.push_str("󰈣 "),
                "mp4" | "avi" | "mov" | "wmv" | "flv" | "webm" | "mkv" => line.push_str("󰈫 "),
                "zip" | "rar" | "tar" | "7z" | "gz" | "xz" => line.push_str(" "),
                "md" | "txt" | "xml" | "yml" | "yaml" => line.push_str("󰈙 "),
                "lock" | "key" | "pem" | "crt" | "p12" | "pfx" => line.push_str("󱆄 "),
                "toml" | "ini" | "cfg" | "conf" => line.push_str("󱁻 "),
                "json" | "csv" | "log" | "sql" => line.push_str("󰱾 "),
                &_ => line.push_str("󰈔 "),
            }
        } else {
            line.push_str("󰈔 ");
        }
    }

    let name = if opts.full_path {
        path.display().to_string()
    } else {
        entry.file_name().to_string_lossy().to_string()
    };
    line.push_str(&name);

    if !file_type.is_dir() && opts.print_size {
        let size = metadata.len();
        let size_str = format!(" ({})", format_file_size(size));
        line.push_str(&size_str);
    }

    if opts.last_modify {
        match metadata.modified() {
            Ok(mod_time) => {
                let date_str = format!(" [{}]", format_date(mod_time));
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

    Ok(line)
}

fn traverse_directory(
    writer: &mut dyn Write,
    path: &Path,
    opts: &Opts,
    display_entries: &HashSet<String>,
    stats: &mut (u64, u64),
    indent_state: &[bool],
) -> std::io::Result<bool> {
    let mut entries_info: Vec<EntryInfo> = fs::read_dir(path)?
        .filter_map(Result::ok)
        .filter(|entry| display_entries.contains(&entry.path().display().to_string()))
        .map(|entry| {
            let last_modify = entry.metadata().and_then(|m| m.modified());
            if let Err(e) = &last_modify {
                eprintln!(
                    "Warning: Could not get metadata/last_modify for {:?}: {}",
                    entry.path(),
                    e
                );
            }
            EntryInfo { entry, last_modify }
        })
        .collect();

    let (mut dirs, mut files): (Vec<EntryInfo>, Vec<EntryInfo>) = std::mem::take(&mut entries_info)
        .into_iter()
        .partition(|info| {
            info.entry
                .file_type()
                .map(|ft| ft.is_dir())
                .unwrap_or(false)
        });

    let sort_comparison = |a: &EntryInfo, b: &EntryInfo| {
        if opts.sort_by_time {
            let time_a = a.last_modify.as_ref().unwrap_or(&SystemTime::UNIX_EPOCH);
            let time_b = b.last_modify.as_ref().unwrap_or(&SystemTime::UNIX_EPOCH);
            time_a
                .cmp(time_b)
                .then_with(|| a.entry.file_name().cmp(&b.entry.file_name()))
        } else {
            a.entry.file_name().cmp(&b.entry.file_name())
        }
    };

    dirs.sort_unstable_by(sort_comparison);
    files.sort_unstable_by(sort_comparison);

    entries_info.append(&mut dirs);
    entries_info.append(&mut files);

    let mut found_content = false;
    let last_idx = entries_info.len().saturating_sub(1);
    for (idx, info) in entries_info.into_iter().enumerate() {
        let entry = info.entry;
        let path = entry.path();
        let is_last_entry = idx == last_idx;
        let line = format_entry_line(&entry, opts, indent_state, is_last_entry)?;

        writeln!(writer, "{line}")?;
        stats.1 += 1;
        found_content = true;

        if entry.file_type()?.is_dir() {
            stats.0 += 1;

            let mut next_indent_state = indent_state.to_vec();
            next_indent_state.push(is_last_entry);
            traverse_directory(
                writer,
                &path,
                opts,
                &display_entries,
                stats,
                &next_indent_state,
            )?;
        }
    }
    Ok(found_content)
}

pub fn print_tree(path: &Path, opts: &Opts) -> std::io::Result<()> {
    let mut writer = Box::new(BufWriter::new(io::stdout()));
    print_tree_with_writer(path, opts, &mut writer)
}

pub fn print_tree_with_writer(
    path: &Path,
    opts: &Opts,
    writer: &mut dyn Write,
) -> std::io::Result<()> {
    let mut display_entries = HashSet::new();
    let reader = match fs::read_dir(path) {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("Error reading directory {path:?}: {e}");
            return Err(e);
        }
    };
    reader.filter_map(Result::ok).for_each(|entry| {
        add_entry_to_display_set(&entry, opts, 1, false, &mut display_entries);
    });

    let display_path = if opts.full_path {
        path.canonicalize()?.display().to_string()
    } else {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(".")
            .to_string()
    };

    let mut stats = (0, 0); // count dirs, count files
    writeln!(writer, "{display_path}")?;
    traverse_directory(writer, path, opts, &display_entries, &mut stats, &[])?;

    let dir_str = if stats.0 == 1 {
        "directory"
    } else {
        "directories"
    };
    let file_str = if stats.1 == 1 { "file" } else { "files" };
    writeln!(
        writer,
        "\n{} {}, {} {}",
        stats.0, dir_str, stats.1, file_str
    )?;
    Ok(())
}
