use unicode_segmentation::UnicodeSegmentation;

pub(in crate::tui) fn clamp_cursor_index(value: &str, index: usize) -> usize {
    let mut index = index.min(value.len());
    while index > 0 && !value.is_char_boundary(index) {
        index -= 1;
    }
    index
}

pub(in crate::tui) fn previous_char_boundary(value: &str, index: usize) -> usize {
    let index = clamp_cursor_index(value, index);
    value[..index]
        .grapheme_indices(true)
        .next_back()
        .map(|(start, _)| start)
        .unwrap_or(0)
}

pub(in crate::tui) fn next_char_boundary(value: &str, index: usize) -> usize {
    let index = clamp_cursor_index(value, index);
    value[index..]
        .grapheme_indices(true)
        .nth(1)
        .map(|(offset, _)| index + offset)
        .unwrap_or(value.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_boundaries_step_over_graphemes() {
        let value = "a🇰🇷e\u{301}z";
        let flag_end = "a🇰🇷".len();
        let accent_end = "a🇰🇷e\u{301}".len();

        assert_eq!(next_char_boundary(value, 0), "a".len());
        assert_eq!(next_char_boundary(value, "a".len()), flag_end);
        assert_eq!(previous_char_boundary(value, flag_end), "a".len());
        assert_eq!(previous_char_boundary(value, accent_end), flag_end);
    }
}
