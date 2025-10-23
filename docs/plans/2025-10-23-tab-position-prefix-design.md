# Tab Position in Tab Names Feature Design

**Date:** 2025-10-23
**Status:** Approved

## Overview

Treat tab `name` as a format string supporting `{tab_position}` placeholder with proper escaping.

## Goals

- Allow users to include tab position anywhere in tab name (e.g., "1: MyTab", "MyTab [1]")
- Support custom format strings with `{tab_position}` placeholder
- Handle escaping (`{{` → `{`, `}}` → `}`) correctly
- Fully backward compatible (literal strings still work)

## Non-Goals

- Support bare `{}` placeholder (only `{tab_position}`)
- Limit placeholder usage (multiple `{tab_position}` allowed)

## Design Decisions

### Payload Structure

**RenamePayload (unchanged):**
```rust
#[derive(Debug, Deserialize)]
struct RenamePayload {
    pane_id: String,
    name: String,  // Now treated as format string
}
```

**Field behavior:**
- `name` is always treated as a format string
- If it contains `{tab_position}`: replaced with 1-indexed tab position
- If it contains no placeholders: used as literal name (backward compatible)
- Escaping with `{{` and `}}` supported

### Format String Syntax

Uses `strfmt` crate for runtime format string processing:

**Placeholder:**
- `{tab_position}` - replaced with 1-indexed tab position (matches Zellij UI)

**Escaping:**
- `{{` → literal `{`
- `}}` → literal `}`

**Multiple placeholders:**
- Allowed: `"{tab_position}-{tab_position}"` → `"1-1"` (all replaced with same value)

**Literal prefix:**
- No placeholder: `"Tab "` → `"Tab MyTab"` (used as-is)

### Examples

```json
{"pane_id": "1", "name": "{tab_position}: MyTab"}
→ "1: MyTab"

{"pane_id": "1", "name": "[{tab_position}] MyTab"}
→ "[1] MyTab"

{"pane_id": "1", "name": "{{{tab_position}}} MyTab"}
→ "{1} MyTab"

{"pane_id": "1", "name": "MyTab [{tab_position}]"}
→ "MyTab [1]"

{"pane_id": "1", "name": "{tab_position}-{tab_position}: MyTab"}
→ "1-1: MyTab"

// Backward compatible - no placeholder
{"pane_id": "1", "name": "MyTab"}
→ "MyTab"

// Literal braces in name
{"pane_id": "1", "name": "MyTab {{escaped}}"}
→ "MyTab {escaped}"
```

### Error Cases

**Invalid format strings (handled by strfmt):**
- `"{}"` - bare placeholder not supported
- `"{unknown}"` - unrecognized variable
- `"{tab_position"` - unclosed brace
- `"tab_position}"` - unmatched brace

**Error handling:**
- Log error to stderr (with debug logging if enabled)
- Use name without prefix
- Continue processing (don't fail entire rename)

## Implementation

### Dependencies

Add to `Cargo.toml`:
```toml
strfmt = "0.2"
```

### New Helper Method

```rust
impl State {
    fn format_tab_name(&self, format_str: &str, tab_position: usize) -> Result<String, String> {
        use strfmt::strfmt;
        use std::collections::HashMap;

        // Create variables map with 1-indexed position
        let mut vars = HashMap::new();
        vars.insert("tab_position".to_string(), (tab_position + 1).to_string());

        // Let strfmt handle all validation and escaping
        strfmt(format_str, &vars).map_err(|e| e.to_string())
    }
}
```

### Integration in pipe() Method

```rust
// After tab position lookup:
let final_name = match self.format_tab_name(&rename_payload.name, tab_position) {
    Ok(name) => name,
    Err(e) => {
        #[cfg(debug_assertions)]
        eprintln!("PLUGIN: Failed to format name '{}': {}", rename_payload.name, e);

        self.show_error(&format!(
            "change-tab-name: invalid name format '{}': {}",
            rename_payload.name, e
        ));
        return false;
    }
};

// Use final_name for no-op check and rename:
if self.tabs.get(tab_position).map(|t| &t.name) == Some(&final_name) {
    return false; // No-op
}

if let Ok(tab_index) = u32::try_from(tab_position) {
    rename_tab(tab_index + 1, final_name);
}
```

## Testing

**Test cases:**
1. Basic prefix: `"{tab_position}: MyTab"` → "1: MyTab"
2. Custom format: `"[{tab_position}] MyTab"` → "[1] MyTab"
3. Escaped braces: `"{{{tab_position}}} MyTab"` → "{1} MyTab"
4. Multiple placeholders: `"{tab_position}-{tab_position}: MyTab"` → "1-1: MyTab"
5. Suffix: `"MyTab [{tab_position}]"` → "MyTab [1]"
6. No placeholder: `"MyTab"` → "MyTab" (backward compatible)
7. Literal braces: `"My{{Tab}}"` → "My{Tab}"
8. Invalid format: `"{}"` → error, don't rename
9. Different tab positions: verify 1-indexed (first tab = 1, second tab = 2, etc.)

## Backward Compatibility

- Existing payloads with literal names work unchanged (no `{tab_position}` = literal string)
- No breaking changes to existing functionality
- Fully backward compatible: `"MyTab"` → `"MyTab"`

## Shell Integration Changes

**zellij-rename-tab.zsh:**
Must escape literal `{` and `}` in user-provided names:
- `{` → `{{`
- `}` → `}}`

This ensures user input like `"My {Project}"` becomes `"My {{Project}}"` and renders as `"My {Project}"` instead of being interpreted as a placeholder.

## Documentation Updates

**README.md:**
- Document that `name` is a format string
- Show `{tab_position}` placeholder examples
- Document escaping rules (`{{` and `}}`)
- Show examples: prefix, suffix, middle

**TESTING.md:**
- Add test cases for format string variations
