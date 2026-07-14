# ratatui-finder

A macOS Finder-style "Go to Path" directory navigation component for [ratatui](https://ratatui.rs) TUI applications.

## Features

- **Interactive path browser** — type a path, get instant listings and fuzzy-matched results
- **Three modes**: listing (`/`-ending), auto-listing (existing directory), matching (partial name)
- **Fuzzy matching** with first-character filter and `*`-wildcard substring search
- **Tab completion** — autocomplete the selected item name
- **Parent navigation** — `Ctrl-w` jumps to parent directory
- **Customizable colors** — override any color in the UI
- **Customizable key bindings** — rebind all actions to your preferred keys
- **Fully self-contained** — no dependency on any framework, just pure data structures + `Widget` rendering

## Quick Start

```toml
[dependencies]
ratatui-finder = { path = "../ratatui-finder" }
```

```rust
use ratatui_finder::{FinderState, FinderConfig, FinderMode, render_finder_popup};

let mut state = FinderState::new(FinderConfig {
    mode: FinderMode::Dir,
    initial_path: "~".to_string(),
    ..Default::default()
});

// In your event loop:
let action = state.handle_key(key_event);

// In your render function:
render_finder_popup(f, area, &mut state);
```

## Default Key Bindings

| Key | Action |
|---|---|
| Character input | Append to input, refresh results |
| `Enter` | Confirm selected path |
| `Esc` / `Ctrl-c` | Cancel |
| `Tab` | Complete selected item name |
| `Up` / `Down` or `Ctrl-p` / `Ctrl-n` | Navigate results |
| `Ctrl-w` | Go to parent directory |
| `Ctrl-u` | Clear input |
| `Home` / `End` or `Ctrl-a` / `Ctrl-e` | Cursor to start/end |
| `Left` / `Right` | Move cursor |
| `Backspace` / `Delete` | Delete character |

All key bindings are configurable via `FinderKeys` — see [Customization](#customization).

## API

### Types

- **`FinderState`** — main state machine holding input, cursor, results, and config
- **`FinderConfig`** — configuration: `mode`, `initial_path`, `extensions`, `colors`, `keys`
- **`FinderMode`** — filter mode: `Dir`, `File`, `Both`
- **`FinderAction`** — feedback to host: `None`, `Confirm(String)`, `Cancel`, `Redraw`
- **`FinderItem`** — a result item with match positions for highlighting
- **`FinderColors`** — color configuration for all UI elements
- **`FinderKeys`** — key binding configuration for all actions
- **`KeyBinding`** — a single key binding (code + modifiers)

### Functions

- `FinderState::new(config)` — create a new instance
- `state.handle_key(key)` — process a key event, returns `FinderAction`
- `state.refresh()` — force refresh after external input changes
- `render_finder_popup(f, area, &mut state)` — render the finder UI in a ratatui frame

## Customization

### Colors

Override any color by setting `FinderColors` on `FinderConfig`:

```rust
use ratatui_finder::{FinderColors, FinderConfig, FinderMode, FinderState};
use ratatui::style::Color;

let config = FinderConfig {
    mode: FinderMode::Dir,
    initial_path: "~".to_string(),
    colors: FinderColors {
        selected_bg: Color::Blue,
        match_fg: Color::Cyan,
        hint_fg: Color::Gray,
        ..Default::default()
    },
    ..Default::default()
};

let state = FinderState::new(config);
```

All color fields and their defaults:

| Field | Default | Purpose |
|---|---|---|
| `input_fg` | `White` | Input text |
| `input_bg` | `Black` | Input background |
| `hint_fg` | `Cyan` | Tab completion hint |
| `hint_bg` | `Black` | Hint background |
| `selected_bg` | `Blue` | Selected result row background |
| `selected_fg` | `White` | Selected result row text |
| `normal_bg` | `Black` | Unselected result row background |
| `normal_fg` | `White` | Unselected result row text |
| `match_fg` | `Yellow` | Highlighted match characters |
| `border_fg` | `White` | Popup border |
| `border_bg` | `Black` | Popup border background |
| `separator_fg` | `DarkGray` | Separator line between input and results |

### Key Bindings

Override any key binding by setting `FinderKeys` on `FinderConfig`:

```rust
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui_finder::{FinderKeys, KeyBinding, FinderConfig, FinderState};

let config = FinderConfig {
    keys: FinderKeys {
        confirm: vec![
            KeyBinding::new(KeyCode::Enter, KeyModifiers::NONE),
            KeyBinding::new(KeyCode::Char('o'), KeyModifiers::CONTROL),
        ],
        cancel: vec![KeyBinding::new(KeyCode::Char('q'), KeyModifiers::NONE)],
        cursor_up: vec![KeyBinding::new(KeyCode::Up, KeyModifiers::NONE)],
        cursor_down: vec![KeyBinding::new(KeyCode::Down, KeyModifiers::NONE)],
        ..Default::default()
    },
    ..Default::default()
};

let state = FinderState::new(config);
```

Each field is a `Vec<KeyBinding>` — multiple keys can map to the same action.
Unbound actions use defaults. Character input always falls through for any `Char` key not caught by a configured binding.

Available action fields:

| Field | Default | Action |
|---|---|---|
| `confirm` | `Enter` | Confirm selected path |
| `cancel` | `Esc`, `Ctrl-c` | Cancel |
| `tab_complete` | `Tab` | Complete selected item name |
| `cursor_up` | `Up`, `Ctrl-p` | Move selection up |
| `cursor_down` | `Down`, `Ctrl-n` | Move selection down |
| `home` | `Home`, `Ctrl-a` | Cursor to start |
| `end` | `End`, `Ctrl-e` | Cursor to end |
| `cursor_left` | `Left` | Move cursor left |
| `cursor_right` | `Right` | Move cursor right |
| `backspace` | `Backspace` | Delete character before cursor |
| `delete` | `Delete` | Delete character after cursor |
| `parent_dir` | `Ctrl-w` | Go to parent directory |
| `clear_input` | `Ctrl-u` | Clear input |

## Visual Layout

```
┌─ Go to Path ────────────────────┐
│ ~/projects/my_app/              │  ← input line
├─────────────────────────────────┤  ← separator (─)
│ ~/projects/my_app/              │  ← self-item (selected: highlighted bg)
│ my_app/src/                     │  ← matched with fg highlights
│ my_app/tests/                   │
│ my_app/config/                  │
│ my_app/README.md                │
│ my_app/Cargo.toml               │
└─────────────────────────────────┘
```