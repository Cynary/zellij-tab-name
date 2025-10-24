use serde::Deserialize;
use zellij_tile::prelude::*;

use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
struct RenamePayload {
    pane_id: String,
    name: String,
    /// Use stable tab ID tracking (default: true)
    /// Set to false to use tab.position directly (will break after tab deletion until Zellij #3535 is fixed)
    #[serde(default = "default_use_stable_ids")]
    use_stable_ids: bool,
}

fn default_use_stable_ids() -> bool {
    true
}

#[derive(Default)]
struct State {
    /// The tabs currently open in the terminal
    tabs: Vec<TabInfo>,

    /// The panes currently open in the terminal
    panes: PaneManifest,

    /// Maps pane id to tab position (0-indexed)
    pane_to_tab: BTreeMap<u32, usize>,

    /// WORKAROUND for Zellij issue #3535:
    /// https://github.com/zellij-org/zellij/issues/3535
    ///
    /// Zellij's rename_tab() expects stable auto-incrementing tab IDs,
    /// but TabInfo doesn't expose them. We track stable IDs ourselves by
    /// assigning each pane a stable tab ID when first seen.
    ///
    /// This can be disabled via use_stable_ids=false in the payload
    /// (for when Zellij fixes the issue)
    pane_to_stable_tab_id: BTreeMap<u32, u32>,

    /// Tracks the next stable tab ID to assign (starts at 1)
    next_stable_tab_id: u32,

    /// AUTO-UPDATE: Stores the original format string (with {tab_position} placeholder)
    /// per stable tab ID. When a tab's position changes, we re-evaluate and rename.
    stable_tab_id_to_format_str: BTreeMap<u32, String>,

    /// Tracks the last known display position for each stable tab ID
    /// Used to detect when positions change and trigger re-evaluation
    stable_tab_id_to_last_position: BTreeMap<u32, usize>,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        // Initialize stable tab ID counter (Zellij tab IDs start at 1)
        self.next_stable_tab_id = 1;

        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[EventType::TabUpdate, EventType::PaneUpdate]);
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        #[cfg(debug_assertions)]
        eprintln!(
            "PLUGIN: Received pipe message: name='{}', has_payload={}",
            pipe_message.name,
            pipe_message.payload.is_some()
        );

        // Only handle messages for our named pipe
        if pipe_message.name != "change-tab-name" {
            #[cfg(debug_assertions)]
            eprintln!(
                "PLUGIN: Ignoring pipe '{}' (not 'change-tab-name')",
                pipe_message.name
            );
            return false;
        }

        #[cfg(debug_assertions)]
        eprintln!("PLUGIN: Processing change-tab-name pipe");

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

        // Look up tab position (current display index)
        let Some(&tab_position) = self.pane_to_tab.get(&pane_id) else {
            self.show_error(&format!("change-tab-name: pane {} not found in mapping (pane_to_tab has {} entries, tabs has {} entries)",
                pane_id, self.pane_to_tab.len(), self.tabs.len()));
            return false;
        };

        // Format the tab name with tab_position placeholder
        let final_name = match self.format_tab_name(&rename_payload.name, tab_position) {
            Ok(name) => name,
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "PLUGIN: Failed to format name '{}': {}",
                    rename_payload.name, e
                );

                self.show_error(&format!(
                    "change-tab-name: invalid name format '{}': {}",
                    rename_payload.name, e
                ));
                return false;
            }
        };

        #[cfg(debug_assertions)]
        eprintln!(
            "PIPE: pane_id={}, tab_position={}, final_name={:?}",
            pane_id, tab_position, final_name
        );

        // Check if rename is needed
        if self.tabs.get(tab_position).map(|t| &t.name) == Some(&final_name) {
            #[cfg(debug_assertions)]
            eprintln!("PIPE: No-op, name already matches");
            return false;
        }

        // Get the tab_id to use for rename_tab
        // See: https://github.com/zellij-org/zellij/issues/3535
        let tab_id = if rename_payload.use_stable_ids {
            // Mode 1 (default): Use our tracked stable tab IDs
            // This works correctly even after tabs are deleted/reordered
            let Some(&stable_tab_id) = self.pane_to_stable_tab_id.get(&pane_id) else {
                self.show_error(&format!(
                    "change-tab-name: no stable tab ID found for pane {}",
                    pane_id
                ));
                return false;
            };

            #[cfg(debug_assertions)]
            eprintln!(
                "PIPE: Using stable_tab_id={} (display_index={})",
                stable_tab_id, tab_position
            );

            stable_tab_id
        } else {
            // Mode 2: Use tab.position + 1 (1-indexed)
            // WARNING: This breaks after tab deletion due to Zellij bug #3535
            let Some(tab) = self.tabs.get(tab_position) else {
                self.show_error(&format!(
                    "change-tab-name: tab at display index {} not found",
                    tab_position
                ));
                return false;
            };

            let tab_id = (tab.position as u32) + 1;

            #[cfg(debug_assertions)]
            eprintln!(
                "PIPE: Using tab.position + 1 = {} (display_index={})",
                tab_id, tab_position
            );

            tab_id
        };

        #[cfg(debug_assertions)]
        eprintln!("PIPE: Calling rename_tab({}, {:?})", tab_id, final_name);

        rename_tab(tab_id, final_name);

        // Store the original format string for auto-update on position changes
        // (works in both modes - stable IDs are always tracked)
        if let Some(&stable_tab_id) = self.pane_to_stable_tab_id.get(&pane_id) {
            self.stable_tab_id_to_format_str
                .insert(stable_tab_id, rename_payload.name.clone());
            self.stable_tab_id_to_last_position
                .insert(stable_tab_id, tab_position);

            #[cfg(debug_assertions)]
            eprintln!(
                "PIPE: Stored format string {:?} for stable_tab_id {} at position {}",
                rename_payload.name, stable_tab_id, tab_position
            );
        }

        false
    }

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
            _ => (),
        };

        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {}
}

impl State {
    /// Rebuild the pane_id -> tab_position mapping from current state
    ///
    /// WORKAROUND for Zellij issue #3535:
    /// Also assigns stable tab IDs to panes for use with rename_tab().
    /// All panes in the same tab share the same stable ID, which persists
    /// even when other tabs are deleted.
    fn rebuild_pane_to_tab(&mut self) {
        self.pane_to_tab.clear();
        // Don't clear pane_to_stable_tab_id - we want to remember stable IDs

        #[cfg(debug_assertions)]
        eprintln!("\n=== REBUILD PANE TO TAB ===");

        // Step 1: Build tab_position -> stable_tab_id map from existing panes
        let mut tab_position_to_stable_id: BTreeMap<usize, u32> = BTreeMap::new();

        for (current_display_index, tab) in self.tabs.iter().enumerate() {
            if let Some(pane_list) = self.panes.panes.get(&tab.position) {
                for pane_info in pane_list {
                    if !pane_info.is_plugin && !pane_info.is_suppressed {
                        // If this pane already has a stable ID, remember it for this tab position
                        if let Some(&stable_id) = self.pane_to_stable_tab_id.get(&pane_info.id) {
                            tab_position_to_stable_id.insert(current_display_index, stable_id);
                            #[cfg(debug_assertions)]
                            eprintln!(
                                "  EXISTING: pane {} has stable_id {} at display_index {}",
                                pane_info.id, stable_id, current_display_index
                            );
                        }
                    }
                }
            }
        }

        // Step 2: Assign stable IDs to new panes and build pane_to_tab mapping
        for (current_display_index, tab) in self.tabs.iter().enumerate() {
            if let Some(pane_list) = self.panes.panes.get(&tab.position) {
                for pane_info in pane_list {
                    if !pane_info.is_plugin && !pane_info.is_suppressed {
                        // Map pane to current display index
                        self.pane_to_tab.insert(pane_info.id, current_display_index);

                        // Assign stable tab ID if this is a new pane
                        #[allow(clippy::map_entry)]
                        if !self.pane_to_stable_tab_id.contains_key(&pane_info.id) {
                            let stable_id = if let Some(&existing_id) =
                                tab_position_to_stable_id.get(&current_display_index)
                            {
                                // Tab already has a stable ID (from other panes), use it
                                #[cfg(debug_assertions)]
                                eprintln!(
                                    "  NEW PANE in existing tab: pane {} gets stable_id {} from tab position {}",
                                    pane_info.id, existing_id, current_display_index
                                );
                                existing_id
                            } else {
                                // New tab, assign a new stable ID
                                let new_id = self.next_stable_tab_id;
                                self.next_stable_tab_id += 1;
                                tab_position_to_stable_id.insert(current_display_index, new_id);

                                #[cfg(debug_assertions)]
                                eprintln!(
                                    "  NEW TAB: pane {} assigned new stable_id {} at position {}",
                                    pane_info.id, new_id, current_display_index
                                );
                                new_id
                            };

                            self.pane_to_stable_tab_id.insert(pane_info.id, stable_id);
                        }

                        #[cfg(debug_assertions)]
                        {
                            let stable_id = self.pane_to_stable_tab_id.get(&pane_info.id).unwrap();
                            eprintln!(
                                "  pane {} -> display_index={}, stable_tab_id={}",
                                pane_info.id, current_display_index, stable_id
                            );
                        }
                    }
                }
            }
        }

        // Auto-update: Check if any tab positions have changed and re-evaluate format strings
        self.auto_update_tab_names();

        #[cfg(debug_assertions)]
        eprintln!("=== END REBUILD ===\n");
    }

    /// Auto-update tab names when positions change
    /// For tabs with stored format strings, check if their position changed
    /// and re-evaluate the format string with the new position
    fn auto_update_tab_names(&mut self) {
        // Build stable_tab_id -> current_display_index mapping
        let mut stable_tab_id_to_current_position: BTreeMap<u32, usize> = BTreeMap::new();
        for (&pane_id, &stable_tab_id) in &self.pane_to_stable_tab_id {
            if let Some(&current_position) = self.pane_to_tab.get(&pane_id) {
                stable_tab_id_to_current_position.insert(stable_tab_id, current_position);
            }
        }

        // Check each tab with a stored format string for position changes
        let tabs_to_update: Vec<(u32, usize, String)> = self
            .stable_tab_id_to_format_str
            .iter()
            .filter_map(|(&stable_tab_id, format_str)| {
                let current_position = stable_tab_id_to_current_position.get(&stable_tab_id)?;
                let last_position = self.stable_tab_id_to_last_position.get(&stable_tab_id)?;

                if current_position != last_position {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "AUTO-UPDATE: stable_tab_id {} moved from position {} to {}",
                        stable_tab_id, last_position, current_position
                    );

                    Some((stable_tab_id, *current_position, format_str.clone()))
                } else {
                    None
                }
            })
            .collect();

        // Re-evaluate and rename tabs that moved
        for (stable_tab_id, new_position, format_str) in tabs_to_update {
            if let Ok(new_name) = self.format_tab_name(&format_str, new_position) {
                #[cfg(debug_assertions)]
                eprintln!(
                    "AUTO-UPDATE: Renaming stable_tab_id {} to {:?} (position {})",
                    stable_tab_id, new_name, new_position
                );

                rename_tab(stable_tab_id, new_name);
                self.stable_tab_id_to_last_position
                    .insert(stable_tab_id, new_position);
            }
        }
    }

    /// Format tab name with tab_position placeholder
    fn format_tab_name(&self, format_str: &str, tab_position: usize) -> Result<String, String> {
        use std::collections::HashMap;
        use strfmt::strfmt;

        // Create variables map with 1-indexed position
        let mut vars = HashMap::new();
        vars.insert("tab_position".to_string(), (tab_position + 1).to_string());

        // Let strfmt handle all validation and escaping
        strfmt(format_str, &vars).map_err(|e| e.to_string())
    }

    /// Log an error message to stderr
    fn show_error(&self, message: &str) {
        eprintln!("{}", message);
    }
}
