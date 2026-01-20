use std::{io, path::Path};

use clap::Parser;
use glob::Pattern;

use crate::cmd::traversal::print_tree;

pub fn parse_glob_pattern(s: &str) -> Result<Pattern, String> {
    Pattern::new(s).map_err(|err| err.to_string())
}

#[derive(Default, Debug)]
pub struct Opts {
    pub show_hidden: bool,
    pub ascii: bool,
    pub full_path: bool,
    pub dir_only: bool,
    pub last_modify: bool,
    pub level: Option<u32>,
    pub no_indent: bool,
    pub print_size: bool,
    pub print_permissions: bool,
    pub pattern: Option<Pattern>,
    pub sort_by_time: bool,
}

#[derive(Parser, Debug)]
pub struct Cmd {
    #[arg(default_value = ".", help = "Path to the directory.")]
    pub path: String,

    #[arg(short = 'a', long = "all", help = "Include hidden files.")]
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

    #[arg(short = 's', long = "size", help = "Print file size.")]
    pub print_size: bool,

    #[arg(short = 'p', long = "permissions", help = "Print permissions.")]
    pub print_permissions: bool,

    #[arg(
        short = 'P',
        long = "pattern",
        help = "List only files/folders that match the wild-card pattern."
    )]
    pub pattern: Option<String>,

    #[arg(
        short = 't',
        long = "sort-by-time",
        help = "Sort by last modification time."
    )]
    pub sort_by_time: bool,
}

impl Cmd {
    pub fn to_opts(&self) -> Result<Opts, String> {
        let glob_pattern: Option<Pattern> = self
            .pattern
            .as_ref()
            .map(|p| parse_glob_pattern(p))
            .transpose()?;
        Ok(Opts {
            show_hidden: self.show_hidden,
            ascii: self.ascii,
            full_path: self.full_path,
            dir_only: self.dir_only,
            print_permissions: self.print_permissions,
            last_modify: self.last_modify,
            level: self.level,
            no_indent: self.no_indent,
            print_size: self.print_size,
            pattern: glob_pattern,
            sort_by_time: self.sort_by_time,
        })
    }
}

pub fn run() -> io::Result<()> {
    let cli = Cmd::parse();
    let opts = cli.to_opts().map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid pattern: {e}"))
    })?;

    let root_path = Path::new(&cli.path);
    print_tree(&root_path, &opts)
}
