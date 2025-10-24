# zellij-tabula

A lightweight [Zellij](https://zellij.dev) plugin for explicit tab renaming with format string support.

## Overview

zellij-tabula is a simple background plugin that lets you rename Zellij tabs on-demand by sending JSON payloads through named pipes. It supports dynamic tab names with placeholders like `{tab_position}` for displaying the tab's position.

**Key Features:**
- **Explicit renaming:** You control when tabs are renamed (no automatic behavior)
- **Format strings:** Use `{tab_position}` to include the 1-indexed tab position in names
- **Lightweight:** Runs silently in the background with no UI
- **Shell integration:** Includes a zsh helper function for easy tab renaming

## Installation

**Requirements:** Zellij `0.40.0` or newer

### 1. Install the plugin

Add to your [zellij config](https://zellij.dev/documentation/configuration.html):

```kdl
load_plugins {
    "https://github.com/bezbac/zellij-tabula/releases/download/v0.3.0/zellij-tabula.wasm"
}
```

### 2. Add shell integration (optional but recommended)

For zsh, source the helper function in your `.zshrc`:

```bash
source /path/to/zellij-rename-tab.zsh
```

This provides the `zellij-rename-tab` command with automatic escaping.

## Usage

### Quick Start

Using the zsh helper (recommended):

```bash
zellij-rename-tab "My Tab Name"
zellij-rename-tab "{tab_position}: Development"
```

### Direct pipe usage

Send a JSON payload via Zellij's pipe mechanism:

```bash
zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "My Tab"}'
```

The plugin finds which tab contains the specified pane and renames it.

### Payload Format

```json
{
  "pane_id": "123",
  "name": "desired tab name"
}
```

- `pane_id`: String containing the numeric ID of a pane in the tab you want to rename
- `name`: Format string for the new tab name (supports `{tab_position}` placeholder)

### Format Strings

The `name` field is treated as a format string that supports dynamic placeholders:

**Placeholder:**
- `{tab_position}` - Replaced with the 1-indexed tab position (first tab = 1, second tab = 2, etc.)

**Escaping:**
- Use `{{` to include a literal `{` in the tab name
- Use `}}` to include a literal `}` in the tab name

**Examples:**

```bash
# Prefix with tab position
zellij pipe --name change-tab-name -- '{"pane_id": "123", "name": "{tab_position}: MyTab"}'
# Result: "1: MyTab" (on first tab)

# Custom format with brackets
zellij pipe --name change-tab-name -- '{"pane_id": "123", "name": "[{tab_position}] MyTab"}'
# Result: "[1] MyTab"

# Tab position as suffix
zellij pipe --name change-tab-name -- '{"pane_id": "123", "name": "MyTab [{tab_position}]"}'
# Result: "MyTab [1]"

# Literal braces in name (escaped)
zellij pipe --name change-tab-name -- '{"pane_id": "123", "name": "My {{Project}}"}'
# Result: "My {Project}"

# No placeholder (backward compatible)
zellij pipe --name change-tab-name -- '{"pane_id": "123", "name": "Static Name"}'
# Result: "Static Name"
```

### Error Handling

If something goes wrong (invalid JSON, pane not found, etc.), the plugin will log the error to Zellij's log file (typically `$TMPDIR/zellij-*/zellij-log/zellij.log`). You can view errors with:

```bash
tail -f $TMPDIR/zellij-*/zellij-log/zellij.log | grep "change-tab-name"
```

**Note:** The plugin must be loaded at Zellij startup (via the config above) to remain active. It runs in the background and doesn't show a UI.

## Advanced Usage

### Custom shell integrations

You can build custom integrations that generate tab names dynamically:

```bash
# Example: Rename based on current directory
zellij-rename-tab "{tab_position}: $(basename "$PWD")"

# Example: Call from scripts
custom_name=$(generate_name_from_context)
zellij-rename-tab "$custom_name"
```

### Direct JSON construction

When building JSON payloads manually (without the zsh helper), remember:
- Escape `{` as `{{` and `}` as `}}` for literal braces
- The `pane_id` identifies which tab to rename (the tab containing that pane)

```bash
zellij pipe --name change-tab-name -- "{\"pane_id\": \"$ZELLIJ_PANE_ID\", \"name\": \"My {{Project}}\"}"
# Results in tab name: "My {Project}"
```

## Contributing

Feel free to suggest ideas or report issues by [opening an issue](https://github.com/bezbac/zellij-tabula/issues/new).
If you want to contribute code changes you will find some useful information in [CONTRIBUTING.md](CONTRIBUTING.md).

## License

The content of this repository is licensed under the BSD-3-Clause license. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

This plugin is based on Zellij's [rust-example-plugin](https://github.com/zellij-org/rust-plugin-example).
