use super::rows::MESSAGE_ROW_GAP;

pub(in crate::tui) fn standalone_message_rendered_height(
    content_rows: usize,
    reaction_rows: usize,
    preview_rows: usize,
) -> usize {
    1 + content_rows + reaction_rows + preview_rows + MESSAGE_ROW_GAP
}
