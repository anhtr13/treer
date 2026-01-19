use std::{
    fs,
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
    entry: fs::DirEntry,
    last_modify: io::Result<SystemTime>,
}

fn should_skip_entry(entry: &fs::DirEntry, opts: &Opts) -> std::io::Result<bool> {
    let path = entry.path();
    let file_name = path.file_name().and_then(|name| name.to_str());

    let is_hidden = file_name.map(|name| name.starts_with('.')).unwrap_or(false);
    if !opts.all_files && is_hidden {
        return Ok(true);
    }

    if opts.dir_only && !path.is_dir() {
        return Ok(true);
    }

    if let Some(pattern) = &opts.pattern {
        if !file_name.is_some_and(|name| pattern.matches(name)) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn format_entry_line(
    entry: &fs::DirEntry,
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
        line.push_str("󰈙 ");
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

fn climb_tree(
    writer: &mut dyn Write,
    path: &Path,
    opts: &Opts,
    depth: usize,
    stats: &mut (u64, u64),
    indent_state: &[bool],
) -> std::io::Result<bool> {
    let read_dir_result = fs::read_dir(path);
    let mut entries_info: Vec<EntryInfo> = match read_dir_result {
        Ok(reader) => reader
            .filter_map(Result::ok)
            .filter(|entry| match should_skip_entry(entry, opts) {
                Ok(skip) => !skip,
                Err(e) => {
                    eprintln!("Could not apply filter to entry {:?}: {}", entry.path(), e);
                    false
                }
            })
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
            .collect(),
        Err(e) => {
            eprintln!("Error reading directory {path:?}: {e}");
            return Err(e);
        }
    };

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
            if let Some(max_level) = opts.level
                && depth + 1 >= max_level as usize
            {
                continue;
            }

            let mut next_indent_state = indent_state.to_vec();
            next_indent_state.push(is_last_entry);
            climb_tree(writer, &path, opts, depth + 1, stats, &next_indent_state)?;
        }
    }
    Ok(found_content)
}

pub fn print_tree(path: &Path, opts: &Opts) -> std::io::Result<()> {
    let mut writer = Box::new(BufWriter::new(io::stdout()));
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
    climb_tree(&mut writer, path, opts, 0, &mut stats, &[])?;

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

    writer.flush()
}
