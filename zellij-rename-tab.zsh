# Rename the current Zellij tab
#
# Usage:
#   zellij-rename-tab "My New Tab Name"
#   zellij-rename-tab "{tab_position}: My Tab"  # Include tab position (1-indexed)
#   zellij-rename-tab "My {Literal} Name"       # Braces are automatically escaped
#
# Format strings:
#   - Use {tab_position} to include the 1-indexed tab position
#   - Literal { and } in names are automatically escaped
#
# Requires:
#   - Zellij with zellij-tabula plugin loaded
#   - ZELLIJ_PANE_ID environment variable (set by Zellij)

zellij-rename-tab() {
    if [[ -z "$ZELLIJ" ]]; then
        echo "Error: Not running in Zellij" >&2
        return 1
    fi

    if [[ -z "$1" ]]; then
        echo "Usage: zellij-rename-tab <new-name>" >&2
        return 1
    fi

    local new_name="$1"

    # Escape special characters in the tab name
    new_name="${new_name//\\/\\\\}"  # Escape backslashes first
    new_name="${new_name//\"/\\\"}"  # Then escape double quotes
    new_name="${new_name//\{/\{\{}"  # Escape { for format string (literal braces)
    new_name="${new_name//\}/\}\}}"  # Escape } for format string (literal braces)

    # Send the JSON payload to the plugin
    zellij pipe --name change-tab-name -- "{\"pane_id\": \"$ZELLIJ_PANE_ID\", \"name\": \"$new_name\"}"
}
