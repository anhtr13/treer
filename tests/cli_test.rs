use std::path::Path;

use treer::cmd::{root::Opts, traversal::print_tree_with_writer};

#[test]
fn test_print_default() {
    let opts: Opts = Default::default();
    let path = Path::new("tests/sample-directory");

    let mut buffer = Vec::new();
    let _ = print_tree_with_writer(path, &opts, &mut buffer);

    let result = String::from_utf8(buffer).expect("Not valid UTF-8");
    let expected = r#"sample-directory
├──  sub-dir-lv1
│   ├──  sub-dir-lv2
│   │   ├──  sub-dir-lv3
│   │   │   └── 󰈔 file5.abc
│   │   └── 󰈔 file4
│   └── 󱁻 file3.toml
├── 󰈙 file1.md
└── 󰈙 file2.txt

3 directories, 8 files
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_print_hidden() {
    let path = Path::new("tests/sample-directory");
    let mut opts: Opts = Default::default();
    opts.show_hidden = true;

    let mut buffer = Vec::new();
    let _ = print_tree_with_writer(path, &opts, &mut buffer);

    let result = String::from_utf8(buffer).expect("Not valid UTF-8");
    let expected = r#"sample-directory
├──  .hidden
├──  sub-dir-lv1
│   ├──  sub-dir-lv2
│   │   ├──  sub-dir-lv3
│   │   │   └── 󰈔 file5.abc
│   │   ├── 󰈙 .hidden2.txt
│   │   └── 󰈔 file4
│   └── 󱁻 file3.toml
├── 󱆄 .hidden.lock
├── 󰈙 file1.md
└── 󰈙 file2.txt

4 directories, 11 files
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_match_pattern() {
    let path = Path::new("tests/sample-directory");
    let mut opts: Opts = Default::default();
    opts.pattern = Some(glob::Pattern::new("*1*").unwrap());

    let mut buffer = Vec::new();
    let _ = print_tree_with_writer(path, &opts, &mut buffer);

    let result = String::from_utf8(buffer).expect("Not valid UTF-8");
    let expected = "sample-directory\n├── \u{1b}[1;31m\u{e5fe} sub-dir-lv1\u{1b}[0m\n│   \u{1b}[31m├── \u{1b}[0m\u{e5fe} sub-dir-lv2\n│   \u{1b}[31m│   \u{1b}[0m\u{1b}[31m├── \u{1b}[0m\u{e5fe} sub-dir-lv3\n│   \u{1b}[31m│   \u{1b}[0m\u{1b}[31m│   \u{1b}[0m\u{1b}[31m└── \u{1b}[0m\u{f0214} file5.abc\n│   \u{1b}[31m│   \u{1b}[0m\u{1b}[31m└── \u{1b}[0m\u{f0214} file4\n│   \u{1b}[31m└── \u{1b}[0m\u{f107b} file3.toml\n└── \u{1b}[1;31m\u{f0219} file1.md\u{1b}[0m\n\n3 directories, 7 files\n";
    assert_eq!(result, expected);

    opts.pattern = Some(glob::Pattern::new("*lv*").unwrap());
    buffer = Vec::new();
    let _ = print_tree_with_writer(path, &opts, &mut buffer);

    let result = String::from_utf8(buffer).expect("Not valid UTF-8");
    let expected = "sample-directory\n└── \u{1b}[1;31m\u{e5fe} sub-dir-lv1\u{1b}[0m\n    \u{1b}[31m├── \u{1b}[0m\u{1b}[1;31m\u{e5fe} sub-dir-lv2\u{1b}[0m\n    \u{1b}[31m│   \u{1b}[0m\u{1b}[31m├── \u{1b}[0m\u{1b}[1;31m\u{e5fe} sub-dir-lv3\u{1b}[0m\n    \u{1b}[31m│   \u{1b}[0m\u{1b}[31m│   \u{1b}[0m\u{1b}[31m└── \u{1b}[0m\u{f0214} file5.abc\n    \u{1b}[31m│   \u{1b}[0m\u{1b}[31m└── \u{1b}[0m\u{f0214} file4\n    \u{1b}[31m└── \u{1b}[0m\u{f107b} file3.toml\n\n3 directories, 6 files\n";
    assert_eq!(result, expected);
}

#[test]
fn test_exclude_patterns() {
    let path = Path::new("tests/sample-directory");
    let mut opts: Opts = Default::default();
    opts.exclude_patterns = vec![glob::Pattern::new("*2*").unwrap()];

    let mut buffer = Vec::new();
    let _ = print_tree_with_writer(path, &opts, &mut buffer);

    let result = String::from_utf8(buffer).expect("Not valid UTF-8");
    let expected = r#"sample-directory
├──  sub-dir-lv1
│   └── 󱁻 file3.toml
└── 󰈙 file1.md

1 directory, 3 files
"#;
    assert_eq!(result, expected);

    opts.exclude_patterns = vec![
        glob::Pattern::new("*2.txt").unwrap(),
        glob::Pattern::new("*3*").unwrap(),
    ];
    buffer = Vec::new();
    let _ = print_tree_with_writer(path, &opts, &mut buffer);

    let result = String::from_utf8(buffer).expect("Not valid UTF-8");
    let expected = r#"sample-directory
├──  sub-dir-lv1
│   └──  sub-dir-lv2
│       └── 󰈔 file4
└── 󰈙 file1.md

2 directories, 4 files
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_max_level() {
    let path = Path::new("tests/sample-directory");
    let mut opts: Opts = Default::default();
    opts.level = Some(3);

    let mut buffer = Vec::new();
    let _ = print_tree_with_writer(path, &opts, &mut buffer);

    let result = String::from_utf8(buffer).expect("Not valid UTF-8");
    let expected = r#"sample-directory
├──  sub-dir-lv1
│   ├──  sub-dir-lv2
│   │   ├──  sub-dir-lv3
│   │   └── 󰈔 file4
│   └── 󱁻 file3.toml
├── 󰈙 file1.md
└── 󰈙 file2.txt

3 directories, 7 files
"#;
    assert_eq!(result, expected);

    opts.level = Some(2);
    buffer = Vec::new();
    let _ = print_tree_with_writer(path, &opts, &mut buffer);

    let result = String::from_utf8(buffer).expect("Not valid UTF-8");
    let expected = r#"sample-directory
├──  sub-dir-lv1
│   ├──  sub-dir-lv2
│   └── 󱁻 file3.toml
├── 󰈙 file1.md
└── 󰈙 file2.txt

2 directories, 5 files
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_combination() {
    let path = Path::new("tests/sample-directory");
    let mut opts: Opts = Default::default();
    opts.show_hidden = true;
    opts.ascii = true;
    opts.level = Some(3);
    opts.exclude_patterns = vec![
        glob::Pattern::new("*2.txt").unwrap(),
        glob::Pattern::new("*3*").unwrap(),
    ];

    let mut buffer = Vec::new();
    let _ = print_tree_with_writer(path, &opts, &mut buffer);

    let result = String::from_utf8(buffer).expect("Not valid UTF-8");
    let expected = r#"sample-directory
|--- .hidden
|--- sub-dir-lv1
|   +--- sub-dir-lv2
|       +---󰈔 file4
|---󱆄 .hidden.lock
+---󰈙 file1.md

3 directories, 6 files
"#;
    assert_eq!(result, expected);

    opts.pattern = Some(glob::Pattern::new("*lv*").unwrap());
    buffer = Vec::new();
    let _ = print_tree_with_writer(path, &opts, &mut buffer);

    let result = String::from_utf8(buffer).expect("Not valid UTF-8");
    let expected = "sample-directory\n+---\u{1b}[1;31m\u{e5fe} sub-dir-lv1\u{1b}[0m\n    \u{1b}[31m+---\u{1b}[0m\u{1b}[1;31m\u{e5fe} sub-dir-lv2\u{1b}[0m\n        \u{1b}[31m+---\u{1b}[0m\u{f0214} file4\n\n2 directories, 3 files\n";
    assert_eq!(result, expected);
}
