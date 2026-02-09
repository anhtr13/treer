use std::{io, path::Path};

use clap::Parser;
use glob::Pattern;

use crate::cmd::traversal::print_tree;

#[derive(Default, Debug)]
pub struct Opts {
    pub show_hidden: bool,
    pub ascii: bool,
    pub exclude_patterns: Vec<Pattern>,
    pub full_path: bool,
    pub dir_only: bool,
    pub last_modify: bool,
    pub level: Option<u32>,
    pub no_indent: bool,
    pub print_size: bool,
    pub print_permissions: bool,
    pub patterns: Vec<Pattern>,
    pub sort_by_time: bool,
}

#[derive(Parser, Debug)]
pub struct Cmd {
    #[arg(default_value = ".", help = "Path to the directory.")]
    pub path: String,

    #[arg(short = 'a', long = "all", help = "All (include hidden) directories.")]
    pub show_hidden: bool,

    #[arg(short = 'A', long = "ascii", help = "Use ascii characters to indent.")]
    pub ascii: bool,

    #[arg(short = 'd', long = "directories", help = "List directories only.")]
    pub dir_only: bool,

    #[arg(short = 'D', long = "date", help = "Print last modification date.")]
    pub last_modify: bool,

    #[arg(short = 'f', long = "full", help = "Print full path prefix")]
    pub full_path: bool,

    #[arg(
        short = 'L',
        long = "level",
        help = "Descend only level directories deep."
    )]
    pub level: Option<u32>,

    #[arg(short = 'i', long = "no-indent", help = "Disable indentation.")]
    pub no_indent: bool,

    #[arg(
        short = 'I',
        long = "exclude",
        help = "Ignore files/folders that match the wild-card pattern. May have multiple -I."
    )]
    pub exclude: Vec<String>,

    #[arg(short = 's', long = "size", help = "Print file size.")]
    pub print_size: bool,

    #[arg(short = 'p', long = "permissions", help = "Print permissions.")]
    pub print_permissions: bool,

    #[arg(
        short = 'P',
        long = "pattern",
        help = "List only directories that match the wild-card pattern. May have multiple -P."
    )]
    pub pattern: Vec<String>,

    #[arg(short = 't', long = "time", help = "Sort by last modification time.")]
    pub sort_by_time: bool,
}

fn parse_glob_pattern(s: &str) -> Result<Pattern, String> {
    Pattern::new(s).map_err(|err| err.to_string())
}

fn cmd_to_opts(cmd: &Cmd) -> Result<Opts, String> {
    let glob_patterns: Vec<Pattern> = cmd
        .pattern
        .iter()
        .map(|p| parse_glob_pattern(p))
        .collect::<Result<Vec<_>, _>>()?;
    let exclude_patterns: Vec<Pattern> = cmd
        .exclude
        .iter()
        .map(|p| parse_glob_pattern(p))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Opts {
        show_hidden: cmd.show_hidden,
        ascii: cmd.ascii,
        exclude_patterns,
        full_path: cmd.full_path,
        dir_only: cmd.dir_only,
        print_permissions: cmd.print_permissions,
        last_modify: cmd.last_modify,
        level: cmd.level,
        no_indent: cmd.no_indent,
        print_size: cmd.print_size,
        patterns: glob_patterns,
        sort_by_time: cmd.sort_by_time,
    })
}

pub fn run() -> io::Result<()> {
    let cmd = Cmd::parse();
    let opts = cmd_to_opts(&cmd).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid pattern: {e}"))
    })?;

    let root_path = Path::new(&cmd.path);
    print_tree(root_path, &opts)
}
