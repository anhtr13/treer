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
│   │   └── 󰈔 file4
│   └── 󱁻 file3.toml
├── 󱆄 .hidden.lock
├── 󰈙 file1.md
└── 󰈙 file2.txt

4 directories, 10 files
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
    let expected = r#"sample-directory
├──  sub-dir-lv1
│   ├──  sub-dir-lv2
│   │   ├──  sub-dir-lv3
│   │   │   └── 󰈔 file5.abc
│   │   └── 󰈔 file4
│   └── 󱁻 file3.toml
└── 󰈙 file1.md

3 directories, 7 files
"#;
    assert_eq!(result, expected);

    opts.pattern = Some(glob::Pattern::new("*lv*").unwrap());
    buffer = Vec::new();
    let _ = print_tree_with_writer(path, &opts, &mut buffer);

    let result = String::from_utf8(buffer).expect("Not valid UTF-8");
    let expected = r#"sample-directory
└──  sub-dir-lv1
    ├──  sub-dir-lv2
    │   ├──  sub-dir-lv3
    │   │   └── 󰈔 file5.abc
    │   └── 󰈔 file4
    └── 󱁻 file3.toml

3 directories, 6 files
"#;
    assert_eq!(result, expected);
}

#[test]
fn test_exclude_patterns() {
    let path = Path::new("tests/sample-directory");
    let mut opts: Opts = Default::default();
    opts.exclude_patterns = vec![
        glob::Pattern::new("*2*").unwrap(),
    ];

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
    let expected = r#"sample-directory
+--- sub-dir-lv1
    +--- sub-dir-lv2
        +---󰈔 file4

2 directories, 3 files
"#;
    assert_eq!(result, expected);
}
