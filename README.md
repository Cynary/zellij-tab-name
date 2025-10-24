# zellij-tabula

A lightweight [Zellij](https://zellij.dev) plugin for explicit tab renaming with format string support.

## Installation

**Requirements:** Zellij `0.40.0` or newer

Add to your [Zellij config](https://zellij.dev/documentation/configuration.html):

```kdl
load_plugins {
    "https://github.com/bezbac/zellij-tabula/releases/download/v0.4.0/zellij-tabula.wasm"
}
```

## Usage

The plugin exposes a named pipe `change-tab-name` that accepts JSON payloads:

```bash
zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "My Tab"}'
```

### Payload Format

```json
{
  "pane_id": "123",
  "name": "Tab Name"
}
```

- `pane_id`: String containing the numeric ID of a pane (the tab containing that pane will be renamed)
- `name`: Format string for the tab name (supports `{tab_position}` placeholder)

### Format Strings

The `name` field supports dynamic placeholders:

- `{tab_position}` - Replaced with the 1-indexed tab position (first tab = 1, second tab = 2, etc.)
- `{{` and `}}` - Escaped to literal `{` and `}`

**Examples:**

```bash
# Tab position prefix
'{"pane_id": "1", "name": "{tab_position}: MyTab"}'
# Result: "1: MyTab"

# Literal braces (escaped)
'{"pane_id": "1", "name": "My {{Project}}"}'
# Result: "My {Project}"
```

## Shell Integration

### Manual Integration (Simple)

Basic helper function for manual tab renaming:

```zsh
# Add to .zshrc
function rename-tab() {
    local title=$1

    # Escape special characters
    title="${title//\\/\\\\}"
    title="${title//\"/\\\"}"
    title="${title//\{/\{\{}"
    title="${title//\}/\}\}}"

    zellij pipe \
        --name change-tab-name \
        -- "{\"pane_id\": \"$ZELLIJ_PANE_ID\", \"name\": \"$title\"}" \
        &>/dev/null &!
}

# Usage
rename-tab "My Tab Name"
rename-tab "{tab_position}: Development"
```

### Automatic Integration (Advanced)

Automatically update tab names based on current directory or running command:

```zsh
# Add to .zshrc
autoload -Uz add-zsh-hook

function current_dir() {
    local dir=$PWD
    if [[ $dir == $HOME ]]; then
        dir="~"
    else
        dir=${dir##*/}
    fi
    echo $dir
}

function change_tab_title() {
    local title=$1

    # Optional: Truncate long names
    if [[ ${#title} -gt 15 ]]; then
        title="${title:0:12}..."
    fi

    # Escape special characters
    title="${title//\\/\\\\}"
    title="${title//\"/\\\"}"

    zellij pipe \
        --name change-tab-name \
        -- "{\"pane_id\": \"$ZELLIJ_PANE_ID\", \"name\": \"{tab_position}: $title\"}" \
        &>/dev/null &!
}

function set_tab_to_working_dir() {
    local title=$(current_dir)
    change_tab_title $title
}

function set_tab_to_command_line() {
    local cmdline=$1
    change_tab_title $cmdline
}

# Enable automatic tab naming in Zellij
if [[ -n $ZELLIJ ]]; then
    add-zsh-hook precmd set_tab_to_working_dir
    add-zsh-hook preexec set_tab_to_command_line
fi
```

This automatically:
- Shows the current directory name before each prompt
- Shows the running command during execution
- Prefixes tab names with their position (e.g., "1: project-name")

## Building from Source

```bash
cd zellij
cargo build --release
```

The WASM plugin will be at `zellij/target/wasm32-wasip1/release/zellij-tabula.wasm`.

## License

BSD-3-Clause - see [LICENSE](LICENSE)

## Acknowledgments

Based on Zellij's [rust-plugin-example](https://github.com/zellij-org/rust-plugin-example).
