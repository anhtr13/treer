# Treer

A simple tree command written in Rust with some interesting features:

- [x] Display directories that match a specific pattern and all their contents.
- [x] Highlight all matched directory names and all their subdirectories.

![screenshot_1](./screenshot_1.png)

![screenshot_2](./screenshot_2.png)

## Installation

- **Prequisite:** Rust and Cargo installed

- **Install:**

  ```sh
  git clone https://github.com/anhtr13/treer
  cd treer
  cargo install --path .
  ```

## Usage

```sh
  treer [OPTIONS] [PATH]
```

**Arguments:**

[PATH]: Path to the directory. [default: .]

**Options:**

| Short | Long                | Description                                                                        |
| ----- | ------------------- | ---------------------------------------------------------------------------------- |
| -a    | --all               | Include hidden files.                                                              |
| -A    | --ascii             | Use ascii characters to indent.                                                    |
| -d    | --directories       | List directories only.                                                             |
| -D    | --date              | Print last modification date.                                                      |
| -f    | --full              | Print full path prefix                                                             |
| -L    | --level <LEVEL>     | Descend only level directories deep.                                               |
| -i    | --no-indent         | Disable indentation.                                                               |
| -I    | --exclude <EXCLUDE> | Ignore directories that match the wild-card pattern. May have multiple -I options. |
| -s    | --size              | Print file size.                                                                   |
| -p    | --permissions       | Print permissions.                                                                 |
| -P    | --pattern <PATTERN> | List only directories that match the wild-card pattern and their subdirectories.   |
| -t    | --sort-by-time      | Sort by last modification time.                                                    |
| -h    | --help              | Print help                                                                         |
