# ratatui-finder

A macOS Finder-style "Go to Path" directory navigation component for [ratatui](https://ratatui.rs) TUI applications.

![ratatui-finder](https://github.com/beyondlex/images/blob/main/ratatui-finder.png)

## Features

- **Interactive path browser** — type a path, get instant listings and fuzzy-matched results
- **Three modes**: listing (`/`-ending), auto-listing (existing directory), matching (partial name)
- **Fuzzy matching** with first-character filter and `*`-wildcard substring search
- **Tab completion** — autocomplete the selected item name
- **Parent navigation** — `Ctrl-w` jumps to parent directory
- **Customizable colors** — override any color in the UI
- **Customizable key bindings** — rebind all actions to your preferred keys
- **Rounded or straight borders** — configurable `BorderType` (Rounded, Plain, Double, Thick)
- **Custom title** — set your own popup title text
- **Smart path entry** — directory paths auto-append `/` on initialization
- **Fully self-contained** — no dependency on any framework, just pure data structures + `Widget` rendering

## Quick Start

```toml
[dependencies]
ratatui-finder = "0.5.0"
crossterm = "0.28"
```

```rust
use crossterm::event::{self, Event, DisableBracketedPaste, EnableBracketedPaste};
use ratatui_finder::{FinderState, FinderConfig, FinderAction, render_finder_popup};

let mut state = FinderState::new(FinderConfig::default());

// In your setup:
execute!(stdout, EnableBracketedPaste)?;

// In your event loop:
loop {
    match event::read()? {
        Event::Key(key) => match state.handle_key(key) {
            FinderAction::Confirm(path) => { /* use path */ break; }
            FinderAction::Cancel => break,
            FinderAction::Redraw => {}
            _ => {}
        },
        Event::Paste(text) => state.handle_paste(&text),
        _ => {}
    }
}

// In your render function:
render_finder_popup(f, f.area(), &mut state);

// On cleanup:
execute!(stdout, DisableBracketedPaste)?;
```

A runnable demo is available:

```sh
cargo run --example demo
```

## Default Key Bindings

| Key | Action |
|---|---|
| Character input | Append to input, refresh results |
| `Enter` | Confirm selected path |
| `Esc` / `Ctrl-c` | Cancel |
| `Tab` | Complete selected item name |
| `Up` / `Down` or `Ctrl-p` / `Ctrl-n` | Navigate results |
| `Ctrl-w` or `Option+Backspace` | Go to parent directory |
| `Ctrl-u` | Clear input |
| `Home` / `End` or `Ctrl-a` / `Ctrl-e` | Cursor to start/end |
| `Left` / `Right` | Move cursor |
| `Option+Left` | Move cursor left by one word |
| `Option+Right` | Move cursor right by one word |
| `Backspace` / `Delete` | Delete character |

All key bindings are configurable via `FinderKeys` — see [Customization](#customization).

## API

### Types

- **`FinderState`** — main state machine holding input, cursor, results, and config
- **`FinderConfig`** — configuration: `mode`, `initial_path`, `extensions`, `colors`, `keys`, `border_type`, `title`
- **`FinderMode`** — filter mode: `Dir`, `File`, `Both`
- **`FinderAction`** — feedback to host: `None`, `Confirm(String)`, `Cancel`, `Redraw`
- **`FinderItem`** — a result item with match positions for highlighting
- **`FinderColors`** — color configuration for all UI elements
- **`FinderKeys`** — key binding configuration for all actions
- **`KeyBinding`** — a single key binding (code + modifiers)

### Methods

| Method | Returns | Description |
|---|---|---|
| `FinderState::new(config)` | `Self` | Create a new instance |
| `state.handle_key(key)` | `FinderAction` | Process a key event from your event loop |
| `state.handle_paste(text)` | — | Paste text at cursor (single refresh). Wire to `Event::Paste` |
| `state.word_left()` | — | Move cursor left by one word (bound to `Alt+b` / `Esc b`) |
| `state.word_right()` | — | Move cursor right by one word (bound to `Alt+f` / `Esc f`) |
| `state.refresh()` | — | Force refresh the results list |
| `render_finder_popup(f, area, &mut state)` | — | Render the finder popup in a ratatui frame |

### Integration Pattern

The finder is a pure state machine — it renders via ratatui and returns actions via `handle_key`.
No callbacks, no channels. Typical integration:

1. Create `FinderState` with your config
2. In your event loop, pass `Event::Key` to `state.handle_key()`
3. Match the returned `FinderAction`:
   - `Confirm(path)` — user selected or typed a path
   - `Cancel` — user dismissed the popup
   - `Redraw` — state changed, render again
   - `None` — unhandled key, you can process it further
4. In your render pass, call `render_finder_popup(f, area, &mut state)`

### Paste Support

Enable bracketed paste mode on the terminal so that pasted text arrives as a single `Event::Paste` instead of many individual key events:

```rust
use crossterm::event::{EnableBracketedPaste, DisableBracketedPaste};

// After entering raw mode:
execute!(stdout, EnableBracketedPaste)?;

// In your event loop:
Event::Paste(text) => state.handle_paste(&text),

// On cleanup:
execute!(stdout, DisableBracketedPaste)?;
```

## Customization

### Colors

Override any color by setting `FinderColors` on `FinderConfig`:

```rust
use ratatui_finder::{FinderColors, FinderConfig, FinderMode, FinderState};
use ratatui::style::Color;

let config = FinderConfig {
    mode: FinderMode::Dir,
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

| Field          | Default | Purpose                                  |
|----------------|---|------------------------------------------|
| `input_fg`     | `White` | Input text                               |
| `input_bg`     | `Black` | Input background                         |
| `hint_fg`      | `Cyan` | Tab completion hint                      |
| `hint_bg`      | `Black` | Hint background                          |
| `selected_bg`  | `Blue` | Selected result row background           |
| `selected_fg`  | `White` | Selected result row text                 |
| `normal_bg`    | `Black` | Unselected result row background         |
| `normal_fg`    | `White` | Unselected result row text               |
| `match_fg`     | `Green` | Highlighted match characters             |
| `border_fg`    | `White` | Popup border                             |
| `border_bg`    | `Black` | Popup border background                  |
| `separator_fg` | `DarkGray` | Separator line between input and results |
| `title_fg`     | `Gray` | Title text                               |

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

### Border & Title

Customize the popup border style and title:

```rust
use ratatui::widgets::BorderType;
use ratatui_finder::{FinderConfig, FinderState};

let config = FinderConfig {
    title: " Navigate ".to_string(),
    border_type: BorderType::Plain,
    ..Default::default()
};

let state = FinderState::new(config);
```

| Field | Default | Options |
|---|---|---|
| `title` | `" Go to: "` | Any string |
| `title_fg` | `Gray` | Title text color (set on `FinderColors`) |
| `border_type` | `BorderType::Rounded` | `Plain`, `Rounded`, `Double`, `Thick`, `QuadrantInside`, `QuadrantOutside` |

