# zellij-tab-name

A lightweight [Zellij](https://zellij.dev) plugin for explicit tab renaming with format string support.

## Installation

**Requirements:** Zellij `0.40.0` or newer

Add to your [Zellij config](https://zellij.dev/documentation/configuration.html):

```kdl
load_plugins {
    "https://github.com/Cynary/zellij-tab-name/releases/download/v0.4.0/zellij-tab-name.wasm"
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
  "name": "Tab Name",
  "use_stable_ids": true
}
```

**Fields:**
- `pane_id`: String containing the numeric ID of a pane (the tab containing that pane will be renamed)
- `name`: Format string for the tab name (supports `{tab_position}` placeholder)
- `use_stable_ids`: Optional boolean (default: `true`)
  - `true`: Use stable tab ID tracking (works correctly after tab deletion)
  - `false`: Use simpler approach (breaks after tab deletion - see Known Issues below)

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

### Auto-Update on Position Changes

When you use `{tab_position}` in a tab name, the plugin automatically updates the tab name when its position changes (e.g., when you delete a tab that comes before it).

**Example:**
1. Create 3 tabs and name them: `"{tab_position}: Dev"`, `"{tab_position}: Test"`, `"{tab_position}: Prod"`
2. Tabs show: "1: Dev", "2: Test", "3: Prod"
3. Delete the first tab
4. **Tabs automatically update to:** "1: Test", "2: Prod"

This works because the plugin stores the original format string and re-evaluates it whenever a tab moves to a different position.

**Note:** Auto-update only works for tabs renamed using `{tab_position}`. Static tab names (without placeholders) are not affected.

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
    title="${title//\{/\{\{}"
    title="${title//\}/\}\}}"

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

## Known Issues

### Tab Deletion Workaround (Zellij #3535)

Due to [Zellij issue #3535](https://github.com/zellij-org/zellij/issues/3535), Zellij's `rename_tab()` API expects stable auto-incrementing tab IDs, but doesn't expose them through `TabInfo`. When tabs are deleted, the position values get renumbered, causing renames to target the wrong tab.

**Our Solution:** The plugin tracks stable tab IDs internally by assigning each pane a stable ID when first seen. This is enabled by default (`use_stable_ids: true`) and works correctly in most cases.

**Known Limitations of the Workaround:**
- Using `zellij action close-tab` may cause issues with stable ID tracking
- Using `zellij action move-tab` to reorder tabs may cause issues
- Recommended: Close tabs by closing all panes within them (Ctrl+d or similar)
- Recommended: Avoid moving tabs programmatically

**Alternative:** Set `use_stable_ids: false` in the payload to use the simpler approach (using `tab.position` directly). This will break after any tab deletion until Zellij fixes the underlying issue.

Once Zellij #3535 is resolved, the workaround can be removed and `use_stable_ids: false` will become the recommended default.

## Building from Source

```bash
cd zellij
cargo build --release
```

The WASM plugin will be at `zellij/target/wasm32-wasip1/release/zellij-tab-name.wasm`.

## License

BSD-3-Clause - see [LICENSE](LICENSE)

## Acknowledgments

This plugin is based on [zellij-tabula](https://github.com/bezbac/zellij-tabula) by Ben Bachem, which provided the foundation and structure for this project. The core functionality has been rewritten to focus on explicit, format-string-based tab renaming rather than automatic directory-based naming.

Additional inspiration from Zellij's [rust-plugin-example](https://github.com/zellij-org/rust-plugin-example).
