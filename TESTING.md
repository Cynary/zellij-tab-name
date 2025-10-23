# Manual Testing Guide

## Setup

1. Build the plugin: `cd zellij && cargo build --release`
2. Start Zellij with test config: `zellij -c test-config.kdl`

## Test Cases

### Test 1: Valid rename
```bash
# In a Zellij pane, get pane ID
echo $ZELLIJ_PANE_ID

# Rename current tab
zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "Test Tab"}'
```
Expected: Tab renames to "Test Tab"

### Test 2: Invalid JSON
```bash
zellij pipe --name change-tab-name -- 'not json'
```
Expected: Toast error: "change-tab-name: invalid JSON: ..."

### Test 3: Missing pane_id
```bash
zellij pipe --name change-tab-name -- '{"name": "Test"}'
```
Expected: Toast error: "change-tab-name: missing 'pane_id' field"

### Test 4: Pane not found
```bash
zellij pipe --name change-tab-name -- '{"pane_id": "99999", "name": "Test"}'
```
Expected: Toast error: "change-tab-name: pane 99999 not found"

### Test 5: No-op rename
```bash
# Rename to "Foo"
zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "Foo"}'

# Rename to "Foo" again
zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "Foo"}'
```
Expected: First rename works, second is silent (no-op)

### Test 6: Multiple tabs
```bash
# Create new tab
# In tab 1: zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "Tab 1"}'
# Switch to tab 2: zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "Tab 2"}'
```
Expected: Each tab renamed correctly
