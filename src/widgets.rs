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
        .border_type(state.config.border_type)
        .title(state.config.title.as_str())
        .title_style(colors.title_fg)
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
    let available = area.width as usize;

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

    let hint_str = hint.as_ref().map(|h| format!(" → {h}"));
    let hint_len = hint_str.as_ref().map(|s| s.len()).unwrap_or(0);
    let input_len = input_text.len();

    let input_width = if hint.is_some() {
        available.saturating_sub(hint_len).max(4)
    } else {
        available
    };

    let (display_input, display_hint) = if input_len <= input_width {
        (input_text.clone(), hint_str)
    } else {
        let prefix = 2;
        let visible = input_width.saturating_sub(prefix);
        let start = input_len - visible;
        (format!("..{}", &input_text[start..]), hint_str)
    };

    let mut spans = Vec::new();
    spans.push(Span::styled(
        display_input,
        Style::default().fg(colors.input_fg).bg(colors.input_bg),
    ));

    if let Some(h) = display_hint {
        spans.push(Span::styled(
            h,
            Style::default()
                .fg(colors.hint_fg)
                .bg(colors.hint_bg)
                .add_modifier(Modifier::ITALIC),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans))
        .style(Style::default().fg(colors.input_fg).bg(colors.input_bg));
    f.render_widget(paragraph, area);

    let cursor_x = if input_len > input_width {
        let visible = input_width.saturating_sub(2);
        let start = input_len - visible;
        if state.cursor >= start {
            area.x + 2 + (state.cursor - start) as u16
        } else {
            area.x + 2
        }
    } else {
        area.x + state.cursor as u16
    };
    let cursor_x = cursor_x.min(area.x + area.width - 1);
    f.set_cursor_position(ratatui::layout::Position::new(cursor_x, area.y));
}

fn render_separator(f: &mut Frame, area: Rect, colors: &FinderColors) {
    let sep = "─".repeat(area.width as usize);
    let spans = vec![Span::styled(sep, Style::default().fg(colors.separator_fg))];
    let paragraph = Paragraph::new(Line::from(spans));
    f.render_widget(paragraph, area);
}

fn compress_path(path: &str, max_width: usize) -> String {
    if path.len() <= max_width || max_width < 6 {
        return path.to_string();
    }

    let segments: Vec<&str> = path.split('/').collect();
    if segments.len() <= 1 {
        let visible = max_width.saturating_sub(2);
        return format!("..{}", &path[path.len() - visible..]);
    }

    let first = segments[0];
    let last = segments.last().unwrap();

    let prefix = if first.is_empty() {
        "/".to_string()
    } else if first.len() + 1 < max_width {
        format!("{}/", first)
    } else {
        "..".to_string()
    };
    let right_budget = max_width.saturating_sub(prefix.len());

    let mut right = last.to_string();

    for i in (1..segments.len() - 1).rev() {
        let seg = segments[i];
        let compressed = if seg.len() > 3 {
            format!("{}..", &seg[..3])
        } else {
            seg.to_string()
        };

        let candidate = format!("{}/{}", compressed, right);
        if candidate.len() <= right_budget {
            right = candidate;
        } else {
            break;
        }
    }

    format!("{}{}", prefix, right)
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

        let (fg, bg) = if is_selected {
            (colors.selected_fg, colors.selected_bg)
        } else {
            (colors.normal_fg, colors.normal_bg)
        };

        let line_y = area.y + i as u16;
        let display_text = &item.display;
        let max_width = area.width as usize;

        let (chars, is_match) = if display_text.len() > max_width {
            let compressed = compress_path(display_text, max_width);
            let c: Vec<char> = compressed.chars().collect();
            let clen = c.len();
            (c, vec![false; clen])
        } else {
            let c: Vec<char> = display_text.chars().collect();
            let mut set = vec![false; c.len()];
            for &p in &item.match_positions {
                if p < set.len() {
                    set[p] = true;
                }
            }
            (c, set)
        };

        let last_slash = chars.iter().rposition(|&c| c == '/');

        let mut spans: Vec<Span> = Vec::new();
        let mut segment = String::new();
        let mut segment_style: Option<Style> = None;

        for (col_u, &ch) in chars.iter().enumerate() {
            let style = if col_u < is_match.len() && is_match[col_u] {
                let match_fg = if is_selected { fg } else { colors.match_fg };
                Style::default().fg(match_fg).bg(bg).add_modifier(Modifier::BOLD)
            } else {
                let is_path = last_slash.map_or(false, |slash| col_u <= slash);
                let text_fg = if is_path { colors.path_fg } else { fg };
                Style::default().fg(text_fg).bg(bg)
            };

            match segment_style {
                Some(prev) if prev == style => segment.push(ch),
                Some(prev) => {
                    spans.push(Span::styled(std::mem::take(&mut segment), prev));
                    segment.push(ch);
                    segment_style = Some(style);
                }
                None => {
                    segment.push(ch);
                    segment_style = Some(style);
                }
            }
        }

        if let Some(style) = segment_style {
            spans.push(Span::styled(segment, style));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).style(Style::default().fg(fg).bg(bg));
        f.render_widget(
            paragraph,
            Rect { x: area.x, y: line_y, width: area.width, height: 1 },
        );
    }
}
