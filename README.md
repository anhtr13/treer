# treer

A simple tree command to learn the Rust programming language.

![screenshot](./screenshot.png)

## Installation

- **Prequisite:** Rust and Cargo installed

- **Installation:**

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
  [PATH]:  Path to the directory. [default: .]

**Options:**
| Short | Long                | Description                                               |
|-------|---------------------|-----------------------------------------------------------|
| -a    | --all               | Include hidden files.                                     |
| -A    | --ascii             | Use ascii characters to indent.                           |
| -d    | --directories       | List directories only.                                    |
| -D    | --date              | Print last modification date.                             |
| -f    | --full              | Print full path prefix                                    |
| -l    | --level <LEVEL>     | Descend only level directories deep.                      |
| -i    | --no-indent         | Disable indentation.                                      |
| -s    | --size              | Print file size.                                          |
| -p    | --permissions       | Print permissions.                                        |
| -P    | --pattern <PATTERN> | List only files/folders that match the wild-card pattern. |
| -t    | --sort-by-time      | Sort by last modification time.                           |
| -h    | --help              | Print help                                                |

