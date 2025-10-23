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
echo '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "Test Tab"}' | \
  zellij pipe --name change-tab-name
```
Expected: Tab renames to "Test Tab"

### Test 2: Invalid JSON
```bash
echo 'not json' | zellij pipe --name change-tab-name
```
Expected: Toast error: "change-tab-name: invalid JSON: ..."

### Test 3: Missing pane_id
```bash
echo '{"name": "Test"}' | zellij pipe --name change-tab-name
```
Expected: Toast error: "change-tab-name: missing 'pane_id' field"

### Test 4: Pane not found
```bash
echo '{"pane_id": "99999", "name": "Test"}' | \
  zellij pipe --name change-tab-name
```
Expected: Toast error: "change-tab-name: pane 99999 not found"

### Test 5: No-op rename
```bash
# Rename to "Foo"
echo '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "Foo"}' | \
  zellij pipe --name change-tab-name

# Rename to "Foo" again
echo '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "Foo"}' | \
  zellij pipe --name change-tab-name
```
Expected: First rename works, second is silent (no-op)

### Test 6: Multiple tabs
```bash
# Create new tab
# In tab 1: echo '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "Tab 1"}' | zellij pipe --name change-tab-name
# Switch to tab 2: echo '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "Tab 2"}' | zellij pipe --name change-tab-name
```
Expected: Each tab renamed correctly
