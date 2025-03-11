use zellij_tile::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::collections::BTreeMap;
use std::fmt;
use owo_colors::OwoColorize;

#[derive(Clone, Serialize, Deserialize)]
pub struct Pane {
    pub pane_info: PaneInfo,
    pub tab_info: TabInfo,
}

impl fmt::Display for Pane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pane ID: {:?} | Tab: {} | Title: {}", self.pane_info.id, self.tab_info.name, self.pane_info.title)
    }
}

struct State {
    confirm_key: KeyWithModifier,
    cancel_key: KeyWithModifier,
    accept_key: KeyWithModifier,
    panes: Vec<Pane>,
    selected: usize,
    current_pane_id: Option<u32>,
    latest_pane_manifest: Option<PaneManifest>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            confirm_key: KeyWithModifier::new(BareKey::Enter),
            cancel_key: KeyWithModifier::new(BareKey::Esc),
            accept_key: KeyWithModifier::new(BareKey::Char('y')),
            panes: Vec::new(),
            selected: 0,
            current_pane_id: None,
            latest_pane_manifest: None,
        }
    }
}

register_plugin!(State);

// NOTE: you can start a development environment inside Zellij by running `zellij -l zellij.kdl` in
// this plugin's folder
//
// More info on plugins: https://zellij.dev/documentation/plugins

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ChangeApplicationState,
            PermissionType::WriteToStdin,
            PermissionType::ReadApplicationState
        ]);
        subscribe(&[EventType::Key, EventType::PaneUpdate, EventType::ModeUpdate, EventType::TabUpdate]);

        if let Some(confirm_key) = configuration.get("confirm_key") {
            self.confirm_key = confirm_key.parse().unwrap_or(self.confirm_key.clone());
        }
        if let Some(abort_key) = configuration.get("cancel_key") {
            self.cancel_key = abort_key.parse().unwrap_or(self.cancel_key.clone());
        }
        if let Some(accept_key) = configuration.get("accept_key") {
            self.accept_key = accept_key.parse().unwrap_or(self.accept_key.clone());
        }
    }
    
    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::Key(key) => {
                if self.accept_key == key {
                    // Just render the current pane ID
                    should_render = true;
                } else if self.cancel_key == key {
                    // Close the plugin
                    hide_self();
                }
            },
            Event::PaneUpdate(pane_manifest) => {
                self.update_pane_info(pane_manifest);
                should_render = true;
            },
            Event::TabUpdate(tabs) => {
                self.update_tab_info(tabs);
                should_render = true;
            },
            _ => {},
        };
        should_render
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        let mut should_render = false;
        // react to data piped to this plugin from the CLI, a keybinding or another plugin
        // read more about pipes: https://zellij.dev/documentation/plugin-pipes
        // return true if this plugin's `render` function should be called for the plugin to render
        // itself
        should_render
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        if let Some(pane_id) = self.current_pane_id {
            // Find the pane with the matching ID
            let pane_info = self.panes.iter().find(|p| p.pane_info.id == pane_id);
            
            if let Some(pane) = pane_info {
                println!("{}", pane.to_string().green().bold());
            } else {
                println!("{}", format!("Pane ID: {}", pane_id).green().bold());
            }
        } else {
            println!("{}", "No current pane selected".red().bold());
        }
    }
}

impl State {
    fn update_tab_info(&mut self, tabs: Vec<TabInfo>) {
        for tab in tabs {
            for pane in &mut self.panes {
                if pane.tab_info.position == tab.position {
                    pane.tab_info = tab.clone();
                }
            }
        }
    }

    fn update_pane_info(&mut self, pane_manifest: PaneManifest) {
        self.latest_pane_manifest = Some(pane_manifest.clone());
        
        // Update the list of panes
        self.panes.clear();
        
        // Iterate through each tab and its panes
        for (tab_index, panes) in pane_manifest.panes.iter() {
            for pane_info in panes {
                if pane_info.is_focused {
                    self.current_pane_id = Some(pane_info.id);
                }
                
                // Create a placeholder TabInfo - this will be updated when we receive tab updates
                let tab_info = TabInfo {
                    position: *tab_index as usize,
                    name: format!("Tab {}", tab_index),
                    active: false,
                    panes_to_hide: 0,
                    is_fullscreen_active: false,
                    is_sync_panes_active: false,
                    are_floating_panes_visible: false,
                    other_focused_clients: vec![],
                    active_swap_layout_name: None,
                    is_swap_layout_dirty: false,
                };
                
                // Add pane to our list
                self.panes.push(Pane {
                    pane_info: pane_info.clone(),
                    tab_info,
                });
            }
        }
    }
}