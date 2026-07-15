use anyhow::Result;
use std::io::stdout;

use crossterm::{
    event::{
        self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use ratatui_finder::{FinderAction, FinderConfig, FinderState, render_finder_popup};

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    execute!(stdout, EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut state = FinderState::new(FinderConfig::default());

    let res = run(&mut terminal, &mut state);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableBracketedPaste)?;
    terminal.show_cursor()?;

    if let Ok(FinderAction::Confirm(path)) = res {
        println!("Confirmed: {path}");
    }

    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, state: &mut FinderState) -> Result<FinderAction> {
    loop {
        terminal.draw(|f| {
            render_finder_popup(f, f.area(), state);
        })?;

        match event::read()? {
            Event::Key(key) => {
                let action = state.handle_key(key);
                match action {
                    FinderAction::Confirm(_) | FinderAction::Cancel => return Ok(action),
                    FinderAction::Redraw => {}
                    FinderAction::None => {
                        if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                            return Ok(FinderAction::Cancel);
                        }
                    }
                }
            }
            Event::Paste(text) => {
                state.handle_paste(&text);
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
}