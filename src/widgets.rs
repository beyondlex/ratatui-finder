use crate::{FinderColors, FinderState};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_finder_popup(f: &mut Frame, area: Rect, state: &mut FinderState) {
    let colors = state.config.colors;

    let popup_width = area.width.min(80).min((area.width as f64 * 0.65) as u16);
    let popup_height = area.height.min(14).min((area.height as f64 * 0.35) as u16).max(6);

    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect { x, y, width: popup_width, height: popup_height };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Go to Path ")
        .style(Style::default().fg(colors.border_fg).bg(colors.border_bg));
    let inner = block.inner(popup_area);

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner);
    let input_area = chunks[0];
    let separator_area = chunks[1];
    let results_area = chunks[2];

    f.render_widget(Clear, popup_area);
    f.render_widget(block, popup_area);

    render_input_line(f, input_area, state, &colors);
    render_separator(f, separator_area, &colors);
    render_results_list(f, results_area, state, &colors);
}

fn render_input_line(f: &mut Frame, area: Rect, state: &mut FinderState, colors: &FinderColors) {
    let input_text = &state.input;

    let hint = if !state.items.is_empty() && state.selected < state.items.len() {
        let selected_item = &state.items[state.selected];
        if !selected_item.is_self {
            let hint_name = &selected_item.name;
            if !input_text.ends_with(hint_name) {
                Some(hint_name.clone())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let mut spans = Vec::new();
    spans.push(Span::styled(
        input_text.clone(),
        Style::default().fg(colors.input_fg).bg(colors.input_bg),
    ));

    if let Some(hint) = hint {
        spans.push(Span::styled(
            format!(" → {hint}"),
            Style::default()
                .fg(colors.hint_fg)
                .bg(colors.hint_bg)
                .add_modifier(Modifier::ITALIC),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans))
        .style(Style::default().fg(colors.input_fg).bg(colors.input_bg));
    f.render_widget(paragraph, area);

    let cursor_x = area.x + state.cursor as u16;
    let cursor_x = cursor_x.min(area.x + area.width - 1);
    f.set_cursor_position(ratatui::layout::Position::new(cursor_x, area.y));
}

fn render_separator(f: &mut Frame, area: Rect, colors: &FinderColors) {
    let sep = "─".repeat(area.width as usize);
    let spans = vec![Span::styled(sep, Style::default().fg(colors.separator_fg))];
    let paragraph = Paragraph::new(Line::from(spans));
    f.render_widget(paragraph, area);
}

fn render_results_list(
    f: &mut Frame,
    area: Rect,
    state: &mut FinderState,
    colors: &FinderColors,
) {
    if area.height == 0 || state.items.is_empty() {
        return;
    }

    let visible_count = area.height as usize;
    let total_items = state.items.len();

    let scroll_offset = if state.selected >= visible_count {
        state.selected - visible_count + 1
    } else {
        0
    };

    let buf = f.buffer_mut();

    for i in 0..visible_count {
        let item_idx = scroll_offset + i;
        if item_idx >= total_items {
            break;
        }

        let item = &state.items[item_idx];
        let is_selected = item_idx == state.selected;

        let (fg, bg) = if is_selected {
            (colors.selected_fg, colors.selected_bg)
        } else {
            (colors.normal_fg, colors.normal_bg)
        };

        let line_y = area.y + i as u16;
        let display_text = &item.display;
        let chars: Vec<char> = display_text.chars().collect();

        // Build a set of match positions for quick lookup
        let is_match: Vec<bool> = if item.match_positions.is_empty() {
            vec![false; chars.len()]
        } else {
            let mut set = vec![false; chars.len()];
            for &p in &item.match_positions {
                if p < set.len() {
                    set[p] = true;
                }
            }
            set
        };

        for col in 0..area.width {
            let cell = &mut buf[(area.x + col, line_y)];
            let col_u = col as usize;
            if col_u < chars.len() {
                let ch = chars[col_u];
                if col_u < is_match.len() && is_match[col_u] {
                    cell.set_fg(colors.match_fg);
                    cell.set_bg(bg);
                    cell.modifier = Modifier::BOLD;
                } else {
                    cell.set_fg(fg);
                    cell.set_bg(bg);
                }
                cell.set_char(ch);
            } else {
                cell.set_fg(fg);
                cell.set_bg(bg);
                cell.set_char(' ');
            }
        }
    }
}