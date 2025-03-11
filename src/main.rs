use zellij_tile::prelude::*;

use std::collections::BTreeMap;

struct State {
    confirm_key: KeyWithModifier,
    cancel_key: KeyWithModifier,
    test_key: KeyWithModifier,
}

impl Default for State {
    fn default() -> Self {
        Self {
            confirm_key: KeyWithModifier::new(BareKey::Enter),
            cancel_key: KeyWithModifier::new(BareKey::Esc),
            test_key: KeyWithModifier::new(BareKey::Char('t')),
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
        request_permission(&[PermissionType::ChangeApplicationState]);
        request_permission(&[PermissionType::WriteToStdin]);
        subscribe(&[EventType::Key]);

        if let Some(confirm_key) = configuration.get("confirm_key") {
            self.confirm_key = confirm_key.parse().unwrap_or(self.confirm_key.clone());
        }
        if let Some(abort_key) = configuration.get("cancel_key") {
            self.cancel_key = abort_key.parse().unwrap_or(self.cancel_key.clone());
        }
        if let Some(test_key) = configuration.get("test_key") {
            self.test_key = test_key.parse().unwrap_or(self.test_key.clone());
        }
    }
    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::Key(key) => {
                if self.confirm_key == key {
                    write("the plugin is working".as_bytes().to_vec());
                    eprintln!("the plugin is working");
                }
            }
            _ => {
                eprintln!("some key registered");
            },
        };
        should_render
    }

    fn pipe (&mut self, pipe_message: PipeMessage) -> bool {
        let mut should_render = false;
        // react to data piped to this plugin from the CLI, a keybinding or another plugin
        // read more about pipes: https://zellij.dev/documentation/plugin-pipes
        // return true if this plugin's `render` function should be called for the plugin to render
        // itself
        should_render
    }

    fn render(&mut self, rows: usize, cols: usize) {
        println!("Hi there! I have {rows} rows and {cols} columns");
    }
}
