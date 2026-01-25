use std::{
    collections::HashSet,
    fs::{DirEntry, read_dir},
    io::{BufWriter, Result, Write, stdout},
    path::Path,
    time::SystemTime,
};

use crate::cmd::{display::format_entry_line, root::Opts};

struct EntryInfo {
    entry: DirEntry,
    last_modify: Result<SystemTime>,
}

fn find_display_entries(
    root: &DirEntry,
    opts: &Opts,
    depth: usize,
    ancestors_matched_pattern: bool,
    display_entries: &mut HashSet<String>,
    highlight_entries: &mut HashSet<String>,
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
            // if current entry matched pattern => highlight current entry
            matched_pattern = true;
            highlight_entries.insert(path.display().to_string());
        }
    }

    if path.is_dir()
        && let read_dir = read_dir(&path)
        && let Ok(reader) = read_dir
    {
        reader.filter_map(Result::ok).for_each(|dir| {
            // if descendants are matched pattern => still display
            should_display |= find_display_entries(
                &dir,
                opts,
                depth + 1,
                matched_pattern,
                display_entries,
                highlight_entries,
            );
        });
    }

    if should_display {
        display_entries.insert(path.display().to_string());
    }

    should_display
}

#[allow(clippy::too_many_arguments)]
fn traverse_directory(
    writer: &mut dyn Write,
    path: &Path,
    opts: &Opts,
    display_entries: &HashSet<String>,
    matched_entries: &HashSet<String>,
    depth: usize,
    furthest_highlighted_ancestor: usize,
    stats: &mut (u64, u64),
    indent_state: &[bool],
) -> Result<bool> {
    let mut entries_info: Vec<EntryInfo> = read_dir(path)?
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
        let should_highlight = matched_entries.contains(&path.display().to_string());
        let furthest_highlighted_ancestor = if should_highlight {
            furthest_highlighted_ancestor.min(depth)
        } else {
            furthest_highlighted_ancestor
        };

        let line = format_entry_line(
            &entry,
            opts,
            indent_state,
            is_last_entry,
            should_highlight,
            furthest_highlighted_ancestor,
        )?;

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
                display_entries,
                matched_entries,
                depth + 1,
                furthest_highlighted_ancestor,
                stats,
                &next_indent_state,
            )?;
        }
    }
    Ok(found_content)
}

pub fn print_tree(path: &Path, opts: &Opts) -> Result<()> {
    let mut writer = Box::new(BufWriter::new(stdout()));
    print_tree_with_writer(path, opts, &mut writer)
}

pub fn print_tree_with_writer(path: &Path, opts: &Opts, writer: &mut dyn Write) -> Result<()> {
    let mut display_entries = HashSet::new();
    let mut highlight_entries = HashSet::new();
    let reader = match read_dir(path) {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("Error reading directory {path:?}: {e}");
            return Err(e);
        }
    };
    reader.filter_map(Result::ok).for_each(|entry| {
        find_display_entries(
            &entry,
            opts,
            1,
            false,
            &mut display_entries,
            &mut highlight_entries,
        );
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
    traverse_directory(
        writer,
        path,
        opts,
        &display_entries,
        &highlight_entries,
        0,
        usize::MAX,
        &mut stats,
        &[],
    )?;

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
