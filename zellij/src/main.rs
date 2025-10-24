use zellij_tile::prelude::*;
use serde::Deserialize;

use std::convert::TryFrom;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
struct RenamePayload {
    pane_id: String,
    name: String,
}

#[derive(Default)]
struct State {
    /// The tabs currently open in the terminal
    tabs: Vec<TabInfo>,

    /// The panes currently open in the terminal
    panes: PaneManifest,

    /// Maps pane id to tab position (0-indexed)
    pane_to_tab: BTreeMap<u32, usize>,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[
            EventType::TabUpdate,
            EventType::PaneUpdate,
        ]);
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        #[cfg(debug_assertions)]
        eprintln!("PLUGIN: Received pipe message: name='{}', has_payload={}",
                  pipe_message.name, pipe_message.payload.is_some());

        // Only handle messages for our named pipe
        if pipe_message.name != "change-tab-name" {
            #[cfg(debug_assertions)]
            eprintln!("PLUGIN: Ignoring pipe '{}' (not 'change-tab-name')", pipe_message.name);
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

        // Look up tab position
        let Some(&tab_position) = self.pane_to_tab.get(&pane_id) else {
            self.show_error(&format!("change-tab-name: pane {} not found", pane_id));
            return false;
        };

        // Format the tab name with tab_position placeholder
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

        // Check if rename is needed
        if self.tabs.get(tab_position).map(|t| &t.name) == Some(&final_name) {
            return false;
        }

        // Rename the tab (tab positions are 0-indexed internally, but rename_tab takes 1-indexed)
        if let Ok(tab_index) = u32::try_from(tab_position) {
            rename_tab(tab_index + 1, final_name);
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

    /// Format tab name with tab_position placeholder
    fn format_tab_name(&self, format_str: &str, tab_position: usize) -> Result<String, String> {
        use strfmt::strfmt;
        use std::collections::HashMap;

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
