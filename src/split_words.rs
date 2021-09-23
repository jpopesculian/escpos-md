pub fn split_words(
    content: &mut Vec<u8>,
    cur_offset: usize,
    printer_width: usize,
    char_size: usize,
) -> usize {
    const WHITESPACE_CHARS: &[u8] = &[b'\n', b'\r', b' '];
    let mut new_offset = cur_offset;
    let mut content_idx = 0;
    while content_idx < content.len() {
        // if newline continue
        match content[content_idx] {
            // reset offset to beginning of line
            b'\n' | b'\r' => {
                new_offset = 0;
                content_idx += 1;
            }
            // continue or wrap line if width reached
            b' ' => {
                new_offset = (new_offset + char_size) % printer_width;
                content_idx += 1;
            }
            // split word or append if unnecessary
            _ => {
                let next_whitespace = content[content_idx..]
                    .iter()
                    .enumerate()
                    .find_map(|(idx, ch)| {
                        if WHITESPACE_CHARS.contains(ch) {
                            Some(idx + content_idx)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| content.len());
                let next_word = &content[content_idx..next_whitespace];
                let next_word_len = next_word.len() * char_size;
                if next_word_len + new_offset > printer_width {
                    content.insert(content_idx, b'\n');
                    new_offset = next_word_len % printer_width;
                    content_idx = next_whitespace + 1;
                } else {
                    new_offset += next_word_len;
                    content_idx = next_whitespace;
                }
            }
        }
    }
    new_offset
}
