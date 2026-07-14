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

    f.render_widget(Clear, popup_area);

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

    render_input_line(f, input_area, state, &colors);
    render_separator(f, separator_area, &colors);
    render_results_list(f, results_area, state, &colors);

    f.render_widget(block, popup_area);

    let cursor_x = x + 1 + state.cursor as u16;
    let cursor_x = cursor_x.min(x + popup_width - 2);
    f.set_cursor_position(ratatui::layout::Position::new(cursor_x, y + 1));
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
        .style(Style::default().bg(colors.input_bg));
    f.render_widget(paragraph, area);
}

fn render_separator(f: &mut Frame, area: Rect, colors: &FinderColors) {
    let sep = "─".repeat(area.width as usize);
    let paragraph = Paragraph::new(Line::from(vec![Span::styled(
        sep,
        Style::default().fg(colors.separator_fg),
    )]));
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

    for i in 0..visible_count {
        let item_idx = scroll_offset + i;
        if item_idx >= total_items {
            break;
        }

        let item = &state.items[item_idx];
        let is_selected = item_idx == state.selected;

        let line_y = area.y + i as u16;
        let line_area = Rect {
            x: area.x,
            y: line_y,
            width: area.width,
            height: 1,
        };

        let (fg, bg) = if is_selected {
            (colors.selected_fg, colors.selected_bg)
        } else {
            (colors.normal_fg, colors.normal_bg)
        };
        let base_style = Style::default().bg(bg).fg(fg);

        // Build line content spans
        let mut spans = Vec::new();
        let display_text = &item.display;

        if item.match_positions.is_empty() {
            spans.push(Span::styled(display_text.clone(), base_style));
        } else {
            let mut last_end = 0;
            let sorted_positions = {
                let mut p = item.match_positions.clone();
                p.sort();
                p.dedup();
                p
            };

            for &pos in &sorted_positions {
                if pos > last_end && last_end < display_text.len() {
                    spans.push(Span::styled(
                        display_text[last_end..pos].to_string(),
                        base_style,
                    ));
                }

                if pos < display_text.len() {
                    let ch = display_text[pos..].chars().next().unwrap_or_default();
                    let ch_len = ch.len_utf8();
                    let end = pos + ch_len;
                    spans.push(Span::styled(
                        ch.to_string(),
                        base_style.fg(colors.match_fg).add_modifier(Modifier::BOLD),
                    ));
                    last_end = end;
                }
            }

            if last_end < display_text.len() {
                spans.push(Span::styled(
                    display_text[last_end..].to_string(),
                    base_style,
                ));
            }
        }

        // Pad to fill full line width for selection background
        spans.push(Span::styled(
            " ".repeat(area.width as usize),
            base_style,
        ));

        let paragraph = Paragraph::new(Line::from(spans));
        f.render_widget(paragraph, line_area);
    }
}