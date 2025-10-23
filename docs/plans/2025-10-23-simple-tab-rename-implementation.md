# Simple Tab Rename Plugin Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the auto-naming logic in zellij-tabula with a simple pipe-based tab rename mechanism that accepts JSON payloads.

**Architecture:** The plugin maintains an always-current pane-to-tab mapping that rebuilds on TabUpdate/PaneUpdate events. When a pipe message arrives with JSON `{"pane_id": "123", "name": "new name"}`, it looks up the pane in the mapping and renames the corresponding tab. Errors show toast notifications.

**Tech Stack:** Rust, Zellij Plugin API (zellij-tile 0.40.1), serde/serde_json for JSON parsing

---

## Task 1: Add JSON Dependencies

**Files:**
- Modify: `.worktrees/simple-tab-rename/zellij/Cargo.toml`

**Step 1: Add serde dependencies**

Add these lines to the `[dependencies]` section in Cargo.toml:

```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

Complete dependencies section should look like:
```toml
[dependencies]
zellij-tile = "0.40.1"
chrono = "0.4.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Step 2: Verify dependencies resolve**

```bash
cd .worktrees/simple-tab-rename/zellij
cargo check
```

Expected: `Finished 'dev' profile` with no errors

**Step 3: Commit**

```bash
cd .worktrees/simple-tab-rename
git add zellij/Cargo.toml
git commit -m "Add serde dependencies for JSON parsing"
```

---

## Task 2: Simplify State Structure

**Files:**
- Modify: `.worktrees/simple-tab-rename/zellij/src/main.rs:1-31`

**Step 1: Remove PathMetadata struct**

Delete lines 7-10 (the entire `PathMetadata` struct).

**Step 2: Simplify State struct**

Replace the State struct (lines 12-31) with:

```rust
#[derive(Default)]
struct State {
    /// The tabs currently open in the terminal
    tabs: Vec<TabInfo>,

    /// The panes currently open in the terminal
    panes: PaneManifest,

    /// Maps pane id to tab position (0-indexed)
    pane_to_tab: BTreeMap<u32, usize>,

    /// Whether the plugin has the necessary permissions
    permissions: Option<PermissionStatus>,
}
```

**Step 3: Verify builds**

```bash
cd .worktrees/simple-tab-rename/zellij
cargo check
```

Expected: Compilation errors about missing imports and methods - that's OK for now

**Step 4: Commit**

```bash
cd .worktrees/simple-tab-rename
git add zellij/src/main.rs
git commit -m "Simplify State structure"
```

---

## Task 3: Add Serde Import and JSON Payload Struct

**Files:**
- Modify: `.worktrees/simple-tab-rename/zellij/src/main.rs:1-6`

**Step 1: Add serde to imports**

After line 1 (`use zellij_tile::prelude::*;`), add:

```rust
use serde::{Deserialize, Serialize};
```

**Step 2: Add payload struct**

After the imports (around line 6), before the State struct, add:

```rust
#[derive(Debug, Deserialize)]
struct RenamePayload {
    pane_id: String,
    name: String,
}
```

**Step 3: Remove rem_first_and_last function**

Delete the `rem_first_and_last` function (lines 35-40 in original).

**Step 4: Verify builds**

```bash
cd .worktrees/simple-tab-rename/zellij
cargo check
```

Expected: Still compilation errors, but serde imports should be OK

**Step 5: Commit**

```bash
cd .worktrees/simple-tab-rename
git add zellij/src/main.rs
git commit -m "Add serde imports and RenamePayload struct"
```

---

## Task 4: Simplify load() Method

**Files:**
- Modify: `.worktrees/simple-tab-rename/zellij/src/main.rs` (the `load` method in `impl ZellijPlugin for State`)

**Step 1: Rewrite load() method**

Replace the entire `load` method with:

```rust
fn load(&mut self, _configuration: BTreeMap<String, String>) {
    request_permission(&[
        PermissionType::ReadApplicationState,
        PermissionType::ChangeApplicationState,
    ]);
    subscribe(&[
        EventType::TabUpdate,
        EventType::PaneUpdate,
        EventType::PermissionRequestResult,
    ]);
}
```

**Step 2: Verify builds**

```bash
cd .worktrees/simple-tab-rename/zellij
cargo check
```

Expected: Fewer errors now

**Step 3: Commit**

```bash
cd .worktrees/simple-tab-rename
git add zellij/src/main.rs
git commit -m "Simplify load() method - remove config and RunCommands"
```

---

## Task 5: Rewrite pipe() Method

**Files:**
- Modify: `.worktrees/simple-tab-rename/zellij/src/main.rs` (the `pipe` method)

**Step 1: Replace pipe() method**

Replace the entire `pipe` method (lines 58-90 in original) with:

```rust
fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
    // Only handle messages for our named pipe
    if pipe_message.name != "change-tab-name" {
        return false;
    }

    // Check for payload
    let Some(payload) = pipe_message.payload else {
        self.show_error("change-tab-name: missing payload");
        return false;
    };

    // Parse JSON
    let rename_payload: RenamePayload = match serde_json::from_str(&payload) {
        Ok(p) => p,
        Err(e) => {
            self.show_error(&format!("change-tab-name: invalid JSON: {}", e));
            return false;
        }
    };

    // Parse pane_id
    let pane_id: u32 = match rename_payload.pane_id.parse() {
        Ok(id) => id,
        Err(_) => {
            self.show_error("change-tab-name: pane_id must be a string containing a number");
            return false;
        }
    };

    // Look up tab position
    let Some(&tab_position) = self.pane_to_tab.get(&pane_id) else {
        self.show_error(&format!("change-tab-name: pane {} not found", pane_id));
        return false;
    };

    // Check if rename is needed
    if self.tabs.get(tab_position).map(|t| &t.name) == Some(&rename_payload.name) {
        // No-op: name already matches
        return false;
    }

    // Rename the tab (tab positions are 0-indexed internally, but rename_tab takes 1-indexed)
    if let Ok(tab_index) = u32::try_from(tab_position) {
        rename_tab(tab_index + 1, rename_payload.name);
    }

    false
}
```

**Step 2: Verify builds**

```bash
cd .worktrees/simple-tab-rename/zellij
cargo check
```

Expected: Error about `show_error` method not existing - we'll add it next

**Step 3: Commit**

```bash
cd .worktrees/simple-tab-rename
git add zellij/src/main.rs
git commit -m "Rewrite pipe() method for JSON-based tab renaming"
```

---

## Task 6: Simplify update() Method

**Files:**
- Modify: `.worktrees/simple-tab-rename/zellij/src/main.rs` (the `update` method)

**Step 1: Replace update() method**

Replace the entire `update` method (lines 92-155 in original) with:

```rust
fn update(&mut self, event: Event) -> bool {
    match event {
        Event::TabUpdate(tab_info) => {
            self.tabs = tab_info;
            self.rebuild_pane_to_tab();
        }
        Event::PaneUpdate(data) => {
            self.panes = data;
            self.rebuild_pane_to_tab();
        }
        Event::PermissionRequestResult(status) => {
            self.permissions = Some(status);
        }
        _ => (),
    };

    false
}
```

**Step 2: Verify builds**

```bash
cd .worktrees/simple-tab-rename/zellij
cargo check
```

Expected: Error about `rebuild_pane_to_tab` method not existing

**Step 3: Commit**

```bash
cd .worktrees/simple-tab-rename
git add zellij/src/main.rs
git commit -m "Simplify update() method to rebuild pane mapping"
```

---

## Task 7: Replace State impl with New Methods

**Files:**
- Modify: `.worktrees/simple-tab-rename/zellij/src/main.rs` (the `impl State` block)

**Step 1: Remove old methods**

Delete the entire `impl State` block (lines 160-282 in original, containing `get_git_worktree_root`, `organize`, and `format_path`).

**Step 2: Add new impl block**

Add this new `impl State` block:

```rust
impl State {
    /// Rebuild the pane_id -> tab_position mapping from current state
    fn rebuild_pane_to_tab(&mut self) {
        self.pane_to_tab.clear();

        for (tab_position, pane_list) in &self.panes.panes {
            for pane_info in pane_list {
                // Only track regular panes (not plugins or suppressed panes)
                if !pane_info.is_plugin && !pane_info.is_suppressed {
                    self.pane_to_tab.insert(pane_info.id, *tab_position);
                }
            }
        }
    }

    /// Show an error toast to the user
    fn show_error(&self, message: &str) {
        eprintln!("{}", message);
        // Only show toast if we have permission
        if let Some(PermissionStatus::Granted) = self.permissions {
            // Toast API: show_self(message, timeout_seconds)
            // We'll use a 5-second timeout for errors
            show_self(false, message);
        }
    }
}
```

**Step 3: Verify builds**

```bash
cd .worktrees/simple-tab-rename/zellij
cargo build
```

Expected: Successful build with no errors

**Step 4: Commit**

```bash
cd .worktrees/simple-tab-rename
git add zellij/src/main.rs
git commit -m "Add rebuild_pane_to_tab and show_error methods"
```

---

## Task 8: Verify Complete Build

**Files:**
- Verify: `.worktrees/simple-tab-rename/zellij/src/main.rs`

**Step 1: Full clean build**

```bash
cd .worktrees/simple-tab-rename/zellij
cargo clean
cargo build
```

Expected: Successful build producing `.wasm` file

**Step 2: Check file size**

```bash
ls -lh target/wasm32-wasip1/debug/zellij_tabula.wasm
```

Expected: File exists, roughly 2-3 MB

**Step 3: Count lines of code**

```bash
wc -l src/main.rs
```

Expected: Approximately 100-130 lines (down from 283)

**Step 4: Review final structure**

Read through `src/main.rs` to verify:
- ✓ Imports include serde
- ✓ RenamePayload struct exists
- ✓ State has 4 fields (tabs, panes, pane_to_tab, permissions)
- ✓ pipe() handles "change-tab-name" with JSON parsing
- ✓ update() rebuilds mapping on events
- ✓ rebuild_pane_to_tab() method exists
- ✓ show_error() method exists
- ✓ No git/path formatting code remains

**Step 5: Commit verification**

```bash
cd .worktrees/simple-tab-rename
git log --oneline
```

Expected: 7-8 commits visible

---

## Task 9: Update README

**Files:**
- Modify: `.worktrees/simple-tab-rename/README.md`

**Step 1: Replace README content**

Replace the entire README.md with:

```markdown
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
```

**Step 2: Verify markdown formatting**

```bash
cd .worktrees/simple-tab-rename
cat README.md | head -20
```

Expected: Well-formatted markdown visible

**Step 3: Commit**

```bash
cd .worktrees/simple-tab-rename
git add README.md
git commit -m "Update README for simplified tab rename functionality"
```

---

## Task 10: Build Release Binary

**Files:**
- Build: `.worktrees/simple-tab-rename/zellij/target/wasm32-wasip1/release/zellij_tabula.wasm`

**Step 1: Build release version**

```bash
cd .worktrees/simple-tab-rename/zellij
cargo build --release
```

Expected: Successful build with optimizations

**Step 2: Check release binary size**

```bash
ls -lh target/wasm32-wasip1/release/zellij_tabula.wasm
```

Expected: Smaller than debug build, roughly 500KB-1MB

**Step 3: Copy to root for easy access**

```bash
cd .worktrees/simple-tab-rename
cp zellij/target/wasm32-wasip1/release/zellij_tabula.wasm .
```

**Step 4: Verify it exists**

```bash
cd .worktrees/simple-tab-rename
ls -lh zellij_tabula.wasm
```

Expected: File exists in worktree root

**Step 5: Commit**

```bash
cd .worktrees/simple-tab-rename
git add zellij_tabula.wasm
git commit -m "Add release build of simplified plugin"
```

---

## Task 11: Manual Testing Setup

**Files:**
- Test: Manual testing with Zellij

**Step 1: Prepare test config**

Create a temporary Zellij config for testing at `.worktrees/simple-tab-rename/test-config.kdl`:

```kdl
load_plugins {
    "file:/Users/rodrigo.gomes/zellij-tabula/.worktrees/simple-tab-rename/zellij_tabula.wasm"
}
```

**Step 2: Document test commands**

Create `.worktrees/simple-tab-rename/TESTING.md`:

```markdown
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
```

**Step 3: Commit**

```bash
cd .worktrees/simple-tab-rename
git add test-config.kdl TESTING.md
git commit -m "Add manual testing documentation and config"
```

---

## Task 12: Final Review and Merge Preparation

**Files:**
- Review all changes in worktree

**Step 1: Review all commits**

```bash
cd .worktrees/simple-tab-rename
git log --oneline main..HEAD
```

Expected: 10-11 commits visible

**Step 2: Check diff from main**

```bash
cd .worktrees/simple-tab-rename
git diff main..HEAD --stat
```

Expected: Shows files modified, lines added/removed

**Step 3: Verify key changes**

```bash
cd .worktrees/simple-tab-rename
git diff main..HEAD zellij/src/main.rs | wc -l
```

Expected: Significant diff in main.rs

**Step 4: Create summary**

Document what was accomplished:
- ✓ Removed PathMetadata, git integration, path formatting
- ✓ Added JSON parsing with serde
- ✓ Implemented pane-to-tab mapping rebuild
- ✓ Added error toasts for invalid payloads
- ✓ Changed pipe name from "tabula" to "change-tab-name"
- ✓ Updated README with new usage
- ✓ Reduced code from 283 to ~130 lines
- ✓ No configuration needed

**Step 5: Ready for merge**

The worktree is ready to merge back to main. Use the **finishing-a-development-branch** skill to decide how to integrate.

---

## Success Criteria

- [ ] Plugin builds without errors
- [ ] Code reduced to ~100-130 lines
- [ ] JSON parsing works for valid payloads
- [ ] Error toasts show for invalid payloads
- [ ] No-op renames are skipped silently
- [ ] pane_to_tab mapping rebuilds on updates
- [ ] README reflects new functionality
- [ ] All changes committed with clear messages

## Next Steps

After implementation:
1. Use **superpowers:finishing-a-development-branch** to merge/PR
2. Manual testing with Zellij (see TESTING.md)
3. Update zsh plugin separately (not in this plan)
4. Create GitHub release with new .wasm file
