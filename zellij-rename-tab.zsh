# Rename the current Zellij tab
#
# Usage:
#   zellij-rename-tab "My New Tab Name"
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

    # Escape double quotes and backslashes in the tab name
    new_name="${new_name//\\/\\\\}"  # Escape backslashes first
    new_name="${new_name//\"/\\\"}"  # Then escape double quotes

    # Send the JSON payload to the plugin
    zellij pipe --name change-tab-name -- "{\"pane_id\": \"$ZELLIJ_PANE_ID\", \"name\": \"$new_name\"}"
}
