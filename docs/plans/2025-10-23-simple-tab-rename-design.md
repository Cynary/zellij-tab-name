# Simple Tab Rename Plugin Design

**Date:** 2025-10-23
**Status:** Approved

## Overview

A simplified Zellij plugin that renames tabs based on pane ID. This replaces the existing zellij-tabula auto-naming logic with a simple pipe-based rename mechanism controlled by external scripts.

## Goals

- Accept JSON payloads via named pipe to rename specific tabs
- Find correct tab based on pane ID
- Maintain efficient pane-to-tab mapping
- Provide clear error feedback via toast notifications
- Keep codebase minimal (~100-120 lines)

## Non-Goals

- Automatic tab naming based on working directories
- Git repository detection
- Path formatting or home directory substitution
- Any user configuration

## Architecture

### State Structure

```rust
struct State {
    tabs: Vec<TabInfo>,                    // Complete tab list from TabUpdate
    panes: PaneManifest,                   // Complete pane manifest from PaneUpdate
    pane_to_tab: BTreeMap<u32, usize>,     // Mapping: pane_id → tab_position
    permissions: Option<PermissionStatus>,  // Permission tracking
}
```

### Event Handling

**TabUpdate / PaneUpdate:**
- Receive complete state snapshot
- Rebuild entire `pane_to_tab` mapping from scratch
- Exclude plugin panes and suppressed panes

**Pipe Messages:**
- Listen on named pipe: `change-tab-name`
- Parse JSON payload: `{"pane_id": "123", "name": "new name"}`
- Look up pane_id in pane_to_tab cache
- Rename tab if found and name differs
- Show toast on errors only

### Rebuild Strategy

On every TabUpdate or PaneUpdate event:
```rust
fn rebuild_pane_to_tab(&mut self) {
    self.pane_to_tab.clear();
    for (tab_position, pane_list) in &self.panes.panes {
        for pane_info in pane_list {
            if !pane_info.is_plugin && !pane_info.is_suppressed {
                self.pane_to_tab.insert(pane_info.id, *tab_position);
            }
        }
    }
}
```

**Rationale:** TabUpdate and PaneUpdate provide complete snapshots, not deltas. Rebuilding is simple, correct, and efficient enough for typical session sizes.

## Data Flow

1. User triggers shell command → sends JSON to Zellij pipe
2. Zellij routes pipe message to plugin
3. Plugin receives via `pipe()` method
4. Parse JSON and extract pane_id (string) and name (string)
5. Convert pane_id string to u32
6. Look up in `pane_to_tab` mapping
7. If found: get tab_position, check current name
8. If name differs: call `rename_tab(tab_position + 1, name)`
9. If not found or parse error: show toast with error details

## Error Handling

### Toast Messages (user-visible)

- Missing payload: `"change-tab-name: missing payload"`
- Invalid JSON: `"change-tab-name: invalid JSON: {error}"`
- Missing pane_id: `"change-tab-name: missing 'pane_id' field"`
- Missing name: `"change-tab-name: missing 'name' field"`
- Invalid pane_id: `"change-tab-name: pane_id must be a string containing a number"`
- Pane not found: `"change-tab-name: pane {pane_id} not found"`

### Silent Handling

- Permission denied: log to stderr only (startup issue, not runtime)
- No-op rename: skip silently when current name equals new name
- Success: no toast (the rename itself is the feedback)

## Edge Cases

- Plugin panes and suppressed panes excluded from mapping
- Tab positions are 0-indexed internally, 1-indexed for rename_tab()
- Empty/whitespace names allowed (Zellij handles validation)
- Concurrent pipe messages processed sequentially by Zellij

## Implementation Changes

### Removed from Existing Plugin

- PathMetadata and path_metadata tracking
- pane_working_dirs mapping
- get_git_worktree_root() and git command execution
- format_path() and home directory formatting
- organize() auto-naming logic
- RunCommandResult event handling
- RunCommands permission
- All userspace_configuration

### Kept from Existing Plugin

- Basic ZellijPlugin structure
- load(), update(), pipe() methods (simplified)
- TabUpdate and PaneUpdate event handling
- Permission request/tracking pattern

### New Dependencies

Add to Cargo.toml:
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Permissions Required

- `ReadApplicationState` - receive TabUpdate/PaneUpdate events
- `ChangeApplicationState` - call rename_tab()

## File Changes

- `zellij/src/main.rs` - rewrite to ~100-120 lines (down from 283)
- `zellij/Cargo.toml` - add serde dependencies
- `README.md` - update to reflect simplified functionality
- Keep: LICENSE, CONTRIBUTING.md, plugin-dev-workspace.kdl

## Testing Strategy

Manual testing scenarios:
1. Send valid JSON with existing pane_id → verify tab renamed
2. Send JSON with non-existent pane_id → verify toast error
3. Send invalid JSON → verify toast error with parse details
4. Send same name twice → verify second rename skipped (no-op)
5. Close pane and try to rename → verify pane not found error
6. Multiple tabs with multiple panes → verify correct tab renamed

## Future Considerations

- Shell integration (zsh plugin) to be updated separately
- Potential to add other pipe commands in future (e.g., query tab info)
- Could add optional debug/verbose configuration if needed
