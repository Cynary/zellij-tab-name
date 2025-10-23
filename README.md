# zellij-tabula

A [Zellij](https://zellij.dev) plugin for renaming tabs via named pipes.

### 🚧 Disclaimer

This project is currently under development and may be subject to frequent changes. Features may be added, modified, or removed without notice. Please use at your own risk and feel free to submit any feedback or suggestions. Thank you for your understanding.

## Installation

zellij-tabula requires both a zellij-plugin _and_ a shell integration to function. As of right now, only zsh is supported.

**Requires Zellij `0.40.0` or newer**.

### Installing the Zellij plugin

Add the following to your [zellij config](https://zellij.dev/documentation/configuration.html):

```kdl
load_plugins {
    "https://github.com/bezbac/zellij-tabula/releases/download/v0.3.0/zellij-tabula.wasm"
}
```

## Usage

Send a JSON payload via Zellij's pipe mechanism to rename a tab:

```bash
echo '{"pane_id": "123", "name": "My Tab"}' | zellij pipe --name change-tab-name
```

The plugin will:
1. Parse the JSON payload
2. Find which tab contains the specified pane
3. Rename that tab to the provided name

### Payload Format

```json
{
  "pane_id": "123",
  "name": "desired tab name"
}
```

- `pane_id`: String containing the numeric ID of a pane in the tab you want to rename
- `name`: String with the new tab name

### Error Handling

If something goes wrong (invalid JSON, pane not found, etc.), the plugin will show a toast notification with details.

## Shell Integration

You can integrate this with your shell to automatically rename tabs. For example, in zsh:

```bash
# Get current pane ID
PANE_ID=$ZELLIJ_PANE_ID

# Rename tab based on some logic
echo "{\"pane_id\": \"$PANE_ID\", \"name\": \"$(generate_name)\"}" | \
  zellij pipe --name change-tab-name
```

## Contributing

Feel free to suggest ideas or report issues by [opening an issue](https://github.com/bezbac/zellij-tabula/issues/new).
If you want to contribute code changes you will find some useful information in [CONTRIBUTING.md](CONTRIBUTING.md).

## License

The content of this repository is licensed under the BSD-3-Clause license. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

This plugin is based on Zellij's [rust-example-plugin](https://github.com/zellij-org/rust-plugin-example).
