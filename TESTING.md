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

## Format String Tests

### Test 7: Basic tab position prefix
```bash
zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "{tab_position}: MyTab"}'
```
Expected: On first tab → "1: MyTab", on second tab → "2: MyTab"

### Test 8: Custom format with brackets
```bash
zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "[{tab_position}] MyTab"}'
```
Expected: On first tab → "[1] MyTab"

### Test 9: Tab position as suffix
```bash
zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "MyTab [{tab_position}]"}'
```
Expected: On first tab → "MyTab [1]"

### Test 10: Escaped braces with placeholder
```bash
zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "{{{tab_position}}} MyTab"}'
```
Expected: On first tab → "{1} MyTab"

### Test 11: Multiple placeholders
```bash
zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "{tab_position}-{tab_position}: MyTab"}'
```
Expected: On first tab → "1-1: MyTab"

### Test 12: Literal braces (no placeholder)
```bash
zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "My{{Project}}"}'
```
Expected: "My{Project}"

### Test 13: No placeholder (backward compatible)
```bash
zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "Static Name"}'
```
Expected: "Static Name"

### Test 14: Invalid format string (bare placeholder)
```bash
zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "{}"}'
```
Expected: Error logged to stderr, tab not renamed

### Test 15: Invalid format string (unknown variable)
```bash
zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "{unknown}"}'
```
Expected: Error logged to stderr, tab not renamed

### Test 16: Different tab positions (verify 1-indexed)
```bash
# Create 3 tabs
# In tab 1: zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "{tab_position}"}'
# In tab 2: zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "{tab_position}"}'
# In tab 3: zellij pipe --name change-tab-name -- '{"pane_id": "'"$ZELLIJ_PANE_ID"'", "name": "{tab_position}"}'
```
Expected: Tab names are "1", "2", "3" (1-indexed, not 0-indexed)
