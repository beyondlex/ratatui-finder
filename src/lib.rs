pub mod fs;
pub mod matcher;
pub mod widgets;

pub use widgets::render_finder_popup;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fs::RawItem;

/// Filter mode: what types of entries to show.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FinderMode {
    Dir,
    File,
    Both,
}

impl Default for FinderMode {
    fn default() -> Self {
        Self::Both
    }
}

/// A single key binding: key code + modifiers.
#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
}

/// Configurable key bindings for the Finder.
#[derive(Debug, Clone)]
pub struct FinderKeys {
    pub confirm: Vec<KeyBinding>,
    pub cancel: Vec<KeyBinding>,
    pub tab_complete: Vec<KeyBinding>,
    pub cursor_up: Vec<KeyBinding>,
    pub cursor_down: Vec<KeyBinding>,
    pub home: Vec<KeyBinding>,
    pub end: Vec<KeyBinding>,
    pub cursor_left: Vec<KeyBinding>,
    pub cursor_right: Vec<KeyBinding>,
    pub backspace: Vec<KeyBinding>,
    pub delete: Vec<KeyBinding>,
    pub parent_dir: Vec<KeyBinding>,
    pub clear_input: Vec<KeyBinding>,
}

impl Default for FinderKeys {
    fn default() -> Self {
        Self {
            confirm: vec![KeyBinding::new(KeyCode::Enter, KeyModifiers::NONE)],
            cancel: vec![
                KeyBinding::new(KeyCode::Esc, KeyModifiers::NONE),
                KeyBinding::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            ],
            tab_complete: vec![KeyBinding::new(KeyCode::Tab, KeyModifiers::NONE)],
            cursor_up: vec![
                KeyBinding::new(KeyCode::Up, KeyModifiers::NONE),
                KeyBinding::new(KeyCode::Char('p'), KeyModifiers::CONTROL),
            ],
            cursor_down: vec![
                KeyBinding::new(KeyCode::Down, KeyModifiers::NONE),
                KeyBinding::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
            ],
            home: vec![
                KeyBinding::new(KeyCode::Home, KeyModifiers::NONE),
                KeyBinding::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
            ],
            end: vec![
                KeyBinding::new(KeyCode::End, KeyModifiers::NONE),
                KeyBinding::new(KeyCode::Char('e'), KeyModifiers::CONTROL),
            ],
            cursor_left: vec![KeyBinding::new(KeyCode::Left, KeyModifiers::NONE)],
            cursor_right: vec![KeyBinding::new(KeyCode::Right, KeyModifiers::NONE)],
            backspace: vec![KeyBinding::new(KeyCode::Backspace, KeyModifiers::NONE)],
            delete: vec![KeyBinding::new(KeyCode::Delete, KeyModifiers::NONE)],
            parent_dir: vec![KeyBinding::new(KeyCode::Char('w'), KeyModifiers::CONTROL)],
            clear_input: vec![KeyBinding::new(KeyCode::Char('u'), KeyModifiers::CONTROL)],
        }
    }
}

/// Configurable colors for the Finder.
#[derive(Debug, Clone, Copy)]
pub struct FinderColors {
    pub input_fg: Color,
    pub input_bg: Color,
    pub hint_fg: Color,
    pub hint_bg: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub normal_bg: Color,
    pub normal_fg: Color,
    pub match_fg: Color,
    pub border_fg: Color,
    pub border_bg: Color,
    pub separator_fg: Color,
}

use ratatui::style::Color;

impl Default for FinderColors {
    fn default() -> Self {
        Self {
            input_fg: Color::White,
            input_bg: Color::Black,
            hint_fg: Color::DarkGray,
            hint_bg: Color::Black,
            selected_bg: Color::DarkGray,
            selected_fg: Color::White,
            normal_bg: Color::Black,
            normal_fg: Color::White,
            match_fg: Color::Yellow,
            border_fg: Color::White,
            border_bg: Color::Black,
            separator_fg: Color::DarkGray,
        }
    }
}

/// Configuration for the Finder component.
#[derive(Debug, Clone)]
pub struct FinderConfig {
    pub mode: FinderMode,
    pub initial_path: String,
    pub extensions: Option<Vec<String>>,
    pub colors: FinderColors,
    pub keys: FinderKeys,
}

impl Default for FinderConfig {
    fn default() -> Self {
        Self {
            mode: FinderMode::Both,
            initial_path: "~".to_string(),
            extensions: None,
            colors: FinderColors::default(),
            keys: FinderKeys::default(),
        }
    }
}

/// An item in the finder results list, ready for rendering.
#[derive(Debug, Clone)]
pub struct FinderItem {
    pub name: String,
    pub display: String,
    pub is_dir: bool,
    pub is_self: bool,
    pub display_offset: usize,
    pub match_positions: Vec<usize>,
}

/// Actions the Finder can signal to the host application.
#[derive(Debug, Clone, PartialEq)]
pub enum FinderAction {
    None,
    Confirm(String),
    Cancel,
    Redraw,
}

/// The main state machine for the Finder component.
#[derive(Debug, Clone)]
pub struct FinderState {
    pub input: String,
    pub cursor: usize,
    pub items: Vec<FinderItem>,
    pub selected: usize,
    pub config: FinderConfig,

    parent_display: String,
    raw_items: Vec<RawItem>,
}

fn matches_any(key: &KeyEvent, bindings: &[KeyBinding]) -> bool {
    bindings
        .iter()
        .any(|b| b.code == key.code && b.modifiers == key.modifiers)
}

impl FinderState {
    /// Create a new FinderState with the given configuration.
    pub fn new(config: FinderConfig) -> Self {
        let mut state = Self {
            input: config.initial_path.clone(),
            cursor: config.initial_path.len(),
            items: Vec::new(),
            selected: 0,
            parent_display: String::new(),
            raw_items: Vec::new(),
            config,
        };
        state.refresh();
        state
    }

    /// Refresh the results list based on current input.
    pub fn refresh(&mut self) {
        self.update_items();
        if self.selected >= self.items.len() && !self.items.is_empty() {
            self.selected = self.items.len() - 1;
        } else if self.items.is_empty() {
            self.selected = 0;
        }
    }

    /// Handle a key event and return an action for the host.
    pub fn handle_key(&mut self, key: KeyEvent) -> FinderAction {
        let keys = &self.config.keys;

        if matches_any(&key, &keys.confirm) {
            return self.action_confirm();
        }
        if matches_any(&key, &keys.cancel) {
            return FinderAction::Cancel;
        }
        if matches_any(&key, &keys.tab_complete) {
            self.tab_complete();
            return FinderAction::Redraw;
        }
        if matches_any(&key, &keys.cursor_up) {
            self.move_selection(-1);
            return FinderAction::Redraw;
        }
        if matches_any(&key, &keys.cursor_down) {
            self.move_selection(1);
            return FinderAction::Redraw;
        }
        if matches_any(&key, &keys.home) {
            self.cursor = 0;
            return FinderAction::Redraw;
        }
        if matches_any(&key, &keys.end) {
            self.cursor = self.input.len();
            return FinderAction::Redraw;
        }
        if matches_any(&key, &keys.cursor_left) {
            if self.cursor > 0 {
                self.cursor = self.cursor.saturating_sub(1);
            }
            return FinderAction::Redraw;
        }
        if matches_any(&key, &keys.cursor_right) {
            if self.cursor < self.input.len() {
                self.cursor = self.cursor.saturating_add(1).min(self.input.len());
            }
            return FinderAction::Redraw;
        }
        if matches_any(&key, &keys.backspace) {
            if self.cursor > 0 && !self.input.is_empty() {
                let before = &self.input[..self.cursor];
                let new_cursor = before
                    .char_indices()
                    .rev()
                    .next()
                    .map(|(i, _c)| i)
                    .unwrap_or(0);
                self.input = format!(
                    "{}{}",
                    &self.input[..new_cursor],
                    &self.input[self.cursor..]
                );
                self.cursor = new_cursor;
                self.refresh();
            }
            return FinderAction::Redraw;
        }
        if matches_any(&key, &keys.delete) {
            if self.cursor < self.input.len() {
                let next = self.input[self.cursor..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _c)| self.cursor + i)
                    .unwrap_or(self.input.len());
                self.input = format!("{}{}", &self.input[..self.cursor], &self.input[next..]);
                self.refresh();
            }
            return FinderAction::Redraw;
        }
        if matches_any(&key, &keys.parent_dir) {
            self.go_up_dir();
            return FinderAction::Redraw;
        }
        if matches_any(&key, &keys.clear_input) {
            self.input.clear();
            self.cursor = 0;
            self.refresh();
            return FinderAction::Redraw;
        }

        // Fallback: character input (any Char not caught by configured bindings)
        if let KeyCode::Char(c) = key.code {
            if self.cursor <= self.input.len() {
                self.input.insert(self.cursor, c);
                self.cursor += 1;
                self.refresh();
            }
            return FinderAction::Redraw;
        }

        FinderAction::None
    }

    fn action_confirm(&mut self) -> FinderAction {
        if self.items.is_empty() {
            return FinderAction::Confirm(self.input.clone());
        }
        let selected = self.selected.min(self.items.len().saturating_sub(1));
        let item = &self.items[selected];
        if item.is_self {
            FinderAction::Confirm(self.input.clone())
        } else {
            let path = if self.input.ends_with('/') {
                format!("{}{}", self.input, item.name)
            } else {
                let parent = self.parent_display.clone();
                format!("{}{}", parent, item.name)
            };
            FinderAction::Confirm(path)
        }
    }

    fn move_selection(&mut self, delta: isize) {
        if self.items.is_empty() {
            return;
        }
        let new = self.selected as isize + delta;
        if new < 0 {
            self.selected = 0;
        } else if new >= self.items.len() as isize {
            self.selected = self.items.len() - 1;
        } else {
            self.selected = new as usize;
        }
    }

    fn tab_complete(&mut self) {
        if self.items.is_empty() || self.selected >= self.items.len() {
            return;
        }
        let item = &self.items[self.selected];
        if item.is_self {
            return;
        }
        let name = &item.name;
        if self.input.ends_with('/') {
            self.input.push_str(name);
        } else {
            if let Some(slash_pos) = self.input.rfind('/') {
                self.input.truncate(slash_pos + 1);
                self.input.push_str(name);
            } else {
                self.input = name.clone();
            }
        }
        self.cursor = self.input.len();
        self.refresh();
    }

    fn go_up_dir(&mut self) {
        if self.input.is_empty() || self.input == "/" {
            return;
        }
        let trimmed = if self.input.ends_with('/') && self.input.len() > 1 {
            &self.input[..self.input.len() - 1]
        } else {
            &self.input
        };
        if let Some(slash_pos) = trimmed.rfind('/') {
            self.input = if slash_pos == 0 {
                "/".to_string()
            } else {
                format!("{}/", &trimmed[..slash_pos])
            };
        } else {
            let parent = fs::parent(&self.input);
            if parent != self.input {
                self.input = parent;
            }
        }
        self.cursor = self.input.len();
        self.refresh();
    }

    fn update_items(&mut self) {
        let input = self.input.clone();

        if input.is_empty() {
            self.items.clear();
            self.raw_items.clear();
            return;
        }

        let expanded = fs::expand(&input);

        if input.ends_with('/') {
            self.raw_items = fs::list(&expanded, self.config.mode);
            self.parent_display = input.clone();
            self.items = self.build_listing_items(&expanded, &input);
        } else if !input.contains('/') {
            if fs::is_dir(&expanded) {
                let dir_path = if input == "~" {
                    "~/".to_string()
                } else {
                    format!("{}/", input)
                };
                self.raw_items = fs::list(&expanded, self.config.mode);
                self.parent_display = dir_path.clone();
                self.items = self.build_listing_items(&expanded, &dir_path);
            } else {
                let cwd = ".";
                self.raw_items = fs::list(cwd, self.config.mode);
                self.parent_display = String::new();
                let matched = matcher::match_items(&self.raw_items, &input);
                self.items = matched
                    .into_iter()
                    .map(|m| FinderItem {
                        display: m.name.clone(),
                        name: m.name,
                        is_dir: m.is_dir,
                        is_self: false,
                        display_offset: 0,
                        match_positions: m.match_positions,
                    })
                    .collect();
            }
        } else {
            if fs::is_dir(&expanded) {
                let dir_path = format!("{}/", input);
                self.raw_items = fs::list(&expanded, self.config.mode);
                self.parent_display = dir_path.clone();
                self.items = self.build_listing_items(&expanded, &dir_path);
            } else {
                let slash_pos = input.rfind('/').unwrap_or(0);
                let parent_dir = &input[..=slash_pos];
                let partial = &input[slash_pos + 1..];

                let expanded_parent = fs::expand(parent_dir);
                self.raw_items = fs::list(&expanded_parent, self.config.mode);
                self.parent_display = parent_dir.to_string();

                let matched = matcher::match_items(&self.raw_items, partial);
                self.items = matched
                    .into_iter()
                    .map(|m| {
                        FinderItem {
                            display: format!("{}{}", parent_dir, m.name),
                            name: m.name,
                            is_dir: m.is_dir,
                            is_self: false,
                            display_offset: parent_dir.len(),
                            match_positions: m.match_positions,
                        }
                    })
                    .collect();
            }
        }
    }

    fn build_listing_items(&mut self, expanded_dir: &str, display_dir: &str) -> Vec<FinderItem> {
        let mut items = Vec::new();

        let dir_name = fs::basename(expanded_dir);
        let dir_display = if dir_name.is_empty() {
            display_dir.to_string()
        } else {
            let parent = fs::parent(expanded_dir);
            let contracted_parent = fs::contract(&parent);
            if contracted_parent.ends_with('/') {
                format!("{}{}", contracted_parent, dir_name)
            } else {
                format!("{}/{}", contracted_parent, dir_name)
            }
        };

        items.push(FinderItem {
            name: dir_name.clone(),
            display: dir_display,
            is_dir: true,
            is_self: true,
            display_offset: 0,
            match_positions: Vec::new(),
        });

        for raw in &self.raw_items {
            items.push(FinderItem {
                display: format!("{}{}", display_dir, raw.name),
                name: raw.name.clone(),
                is_dir: raw.is_dir,
                is_self: false,
                display_offset: display_dir.len(),
                match_positions: Vec::new(),
            });
        }

        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn test_new_finder_state() {
        let state = FinderState::new(FinderConfig {
            mode: FinderMode::Both,
            initial_path: "~".to_string(),
            ..Default::default()
        });
        assert_eq!(state.input, "~");
        assert_eq!(state.cursor, 1);
    }

    #[test]
    fn test_handle_key_char_input() {
        let mut state = FinderState::new(FinderConfig {
            mode: FinderMode::Both,
            initial_path: "".to_string(),
            ..Default::default()
        });
        let action = state.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        assert_eq!(action, FinderAction::Redraw);
        assert_eq!(state.input, "a");
        assert_eq!(state.cursor, 1);
    }

    #[test]
    fn test_handle_key_esc() {
        let mut state = FinderState::new(FinderConfig::default());
        let action = state.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
        assert_eq!(action, FinderAction::Cancel);
    }

    #[test]
    fn test_handle_key_ctrl_c() {
        let mut state = FinderState::new(FinderConfig::default());
        let action = state.handle_key(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        ));
        assert_eq!(action, FinderAction::Cancel);
    }

    #[test]
    fn test_handle_key_backspace() {
        let mut state = FinderState::new(FinderConfig {
            mode: FinderMode::Both,
            initial_path: "ab".to_string(),
            ..Default::default()
        });
        state.cursor = 2;
        let action = state.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty()));
        assert_eq!(action, FinderAction::Redraw);
        assert_eq!(state.input, "a");
    }

    #[test]
    fn test_handle_key_delete() {
        let mut state = FinderState::new(FinderConfig {
            mode: FinderMode::Both,
            initial_path: "ab".to_string(),
            ..Default::default()
        });
        state.cursor = 0;
        let action = state.handle_key(KeyEvent::new(KeyCode::Delete, KeyModifiers::empty()));
        assert_eq!(action, FinderAction::Redraw);
        assert_eq!(state.input, "b");
    }

    #[test]
    fn test_move_selection() {
        let mut state = FinderState::new(FinderConfig::default());
        state.items = vec![
            FinderItem {
                name: "a".into(),
                display: "a".into(),
                is_dir: false,
                is_self: false,
                display_offset: 0,
                match_positions: vec![],
            },
            FinderItem {
                name: "b".into(),
                display: "b".into(),
                is_dir: false,
                is_self: false,
                display_offset: 0,
                match_positions: vec![],
            },
        ];
        state.selected = 0;
        state.move_selection(1);
        assert_eq!(state.selected, 1);
        state.move_selection(1);
        assert_eq!(state.selected, 1);
        state.move_selection(-1);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_go_up_dir() {
        let mut state = FinderState::new(FinderConfig {
            mode: FinderMode::Both,
            initial_path: "~/a/b/".to_string(),
            ..Default::default()
        });
        state.go_up_dir();
        assert_eq!(state.input, "~/a/");
    }

    #[test]
    fn test_tab_complete() {
        let mut state = FinderState::new(FinderConfig {
            mode: FinderMode::Both,
            initial_path: "~/".to_string(),
            ..Default::default()
        });
        state.refresh();
        let _ = state.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()));
    }

    #[test]
    fn test_ctrl_w() {
        let mut state = FinderState::new(FinderConfig {
            mode: FinderMode::Both,
            initial_path: "~/a/b/c".to_string(),
            ..Default::default()
        });
        let action = state.handle_key(KeyEvent::new(
            KeyCode::Char('w'),
            KeyModifiers::CONTROL,
        ));
        assert_eq!(action, FinderAction::Redraw);
        assert_eq!(state.input, "~/a/b/");
    }

    #[test]
    fn test_ctrl_u() {
        let mut state = FinderState::new(FinderConfig {
            mode: FinderMode::Both,
            initial_path: "some text".to_string(),
            ..Default::default()
        });
        let action = state.handle_key(KeyEvent::new(
            KeyCode::Char('u'),
            KeyModifiers::CONTROL,
        ));
        assert_eq!(action, FinderAction::Redraw);
        assert!(state.input.is_empty());
    }

    #[test]
    fn test_enter_on_empty_items() {
        let mut state = FinderState::new(FinderConfig {
            mode: FinderMode::Both,
            initial_path: "/tmp".to_string(),
            ..Default::default()
        });
        state.items.clear();
        let action = state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));
        assert_eq!(action, FinderAction::Confirm("/tmp".to_string()));
    }

    #[test]
    fn test_cursor_movement() {
        let mut state = FinderState::new(FinderConfig {
            mode: FinderMode::Both,
            initial_path: "hello".to_string(),
            ..Default::default()
        });
        state.cursor = 5;

        let _ = state.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::empty()));
        assert_eq!(state.cursor, 4);

        let _ = state.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::empty()));
        assert_eq!(state.cursor, 5);

        let _ = state.handle_key(KeyEvent::new(KeyCode::Home, KeyModifiers::empty()));
        assert_eq!(state.cursor, 0);

        let _ = state.handle_key(KeyEvent::new(KeyCode::End, KeyModifiers::empty()));
        assert_eq!(state.cursor, 5);
    }

    #[test]
    fn test_ctrl_a_and_ctrl_e() {
        let mut state = FinderState::new(FinderConfig {
            mode: FinderMode::Both,
            initial_path: "hello".to_string(),
            ..Default::default()
        });
        state.cursor = 3;

        let _ = state.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL));
        assert_eq!(state.cursor, 0);

        let _ = state.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL));
        assert_eq!(state.cursor, 5);
    }

    #[test]
    fn test_listing_mode_on_slash_ending() {
        let state = FinderState::new(FinderConfig {
            mode: FinderMode::Both,
            initial_path: "/tmp/".to_string(),
            ..Default::default()
        });
        assert!(!state.items.is_empty());
        assert!(state.items[0].is_self);
    }

    #[test]
    fn test_auto_listing_on_directory() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir_path = tmp.path().to_string_lossy().to_string();
        std::fs::write(tmp.path().join("test.txt"), "hello").unwrap();
        let state = FinderState::new(FinderConfig {
            mode: FinderMode::Both,
            initial_path: dir_path.clone(),
            ..Default::default()
        });
        assert!(!state.items.is_empty(), "items should not be empty for directory");
        assert!(state.items[0].is_self, "first item should be self-item");
    }

    #[test]
    fn test_custom_keys_override_defaults() {
        let custom_keys = FinderKeys {
            confirm: vec![KeyBinding::new(KeyCode::Char('o'), KeyModifiers::CONTROL)],
            cancel: vec![KeyBinding::new(KeyCode::Char('q'), KeyModifiers::NONE)],
            ..Default::default()
        };
        let mut state = FinderState::new(FinderConfig {
            keys: custom_keys,
            ..Default::default()
        });
        state.items.clear();

        // Default Enter should NOT confirm (overridden), returns None
        let action = state.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, FinderAction::None);

        // Custom Ctrl-o should confirm
        let action = state.handle_key(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL));
        assert_eq!(action, FinderAction::Confirm("~".to_string()));

        // Custom 'q' should cancel
        let action = state.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        assert_eq!(action, FinderAction::Cancel);
    }
}