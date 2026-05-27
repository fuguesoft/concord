use super::*;

pub(in crate::tui::ui) fn render_message_url_picker(
    frame: &mut Frame,
    area: Rect,
    state: &DashboardState,
) {
    if !state.is_active_modal_popup(ActiveModalPopupKind::MessageUrlPicker) {
        return;
    }

    let urls = state.selected_message_url_items();
    if urls.is_empty() {
        return;
    }
    let selected = state.selected_message_url_index().unwrap_or(0);
    let popup = centered_rect(area, 54, (urls.len() as u16).saturating_add(2));
    let lines = truncate_message_url_picker_lines(
        message_url_picker_lines(&urls, selected),
        popup.width.saturating_sub(2) as usize,
    );
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel_block("Open URL", true))
            .wrap(Wrap { trim: false }),
        popup,
    );
}

pub(in crate::tui::ui) fn message_url_picker_lines(
    urls: &[MessageUrlItem],
    selected: usize,
) -> Vec<Line<'static>> {
    urls.iter()
        .enumerate()
        .map(|(index, item)| {
            let marker = if index == selected { "› " } else { "  " };
            let shortcut = shortcut_prefix(
                crate::tui::keybindings::KeyBindings::default().indexed_shortcut(index),
            );
            let mut style = Style::default();
            if index == selected {
                style = style
                    .bg(Color::Rgb(40, 45, 90))
                    .add_modifier(Modifier::BOLD);
            }
            Line::from(vec![
                Span::styled(marker, Style::default().fg(ACCENT)),
                Span::styled(shortcut, Style::default().fg(DIM)),
                Span::styled(item.label.to_owned(), style),
            ])
        })
        .collect()
}

#[cfg(test)]
pub(in crate::tui::ui) fn message_url_picker_lines_for_width(
    urls: &[MessageUrlItem],
    selected: usize,
    width: usize,
) -> Vec<Line<'static>> {
    truncate_message_url_picker_lines(message_url_picker_lines(urls, selected), width)
}

fn truncate_message_url_picker_lines(
    lines: Vec<Line<'static>>,
    width: usize,
) -> Vec<Line<'static>> {
    lines
        .into_iter()
        .map(|line| truncate_line_to_display_width(line, width.max(1)))
        .collect()
}
