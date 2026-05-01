use sourceview5 as source;
use source::prelude::*;

pub fn apply_markup(buffer: &source::Buffer, symbol: &str) {
    let (mut start, mut end) = buffer.selection_bounds().unwrap_or_else(|| {
        let it = buffer.iter_at_offset(buffer.cursor_position());
        (it.clone(), it)
    });

    // Check if this is a "list" style symbol (prefix-based)
    let is_list = symbol == "- " || symbol == "1. " || symbol == "- [ ] " || symbol.ends_with(' ');

    if is_list {
        buffer.begin_user_action();
        let start_line = start.line();
        let end_line = end.line();
        
        // Collect all lines in selection
        let mut lines = Vec::new();
        for i in start_line..=end_line {
            let line_start = buffer.iter_at_line(i).expect("Valid line");
            let mut line_end = buffer.iter_at_line(i).expect("Valid line");
            if !line_end.ends_line() {
                line_end.forward_to_line_end();
            }
            lines.push((line_start, line_end));
        }

        // Determine if we are adding or removing
        let all_have_prefix = lines.iter().all(|(s, e)| {
            let text = buffer.text(s, e, false).to_string();
            text.trim_start().starts_with(symbol)
        });

        if all_have_prefix {
            // Remove prefix from each line
            for (s, e) in lines.into_iter().rev() {
                let text = buffer.text(&s, &e, false).to_string();
                if let Some(pos) = text.find(symbol) {
                    let mut start_del = buffer.iter_at_line(s.line()).expect("Valid line");
                    start_del.forward_chars(pos as i32);
                    let mut end_del = start_del.clone();
                    end_del.forward_chars(symbol.chars().count() as i32);
                    buffer.delete(&mut start_del, &mut end_del);
                }
            }
        } else {
            // Add prefix to each line
            for (mut s, _) in lines {
                buffer.insert(&mut s, symbol);
            }
        }
        buffer.end_user_action();
        return;
    }

    if start == end {
        // No selection, just insert symbol
        buffer.insert(&mut start, symbol);
    } else {
        let text = buffer.text(&start, &end, false).to_string();
        let symbol_char_count = symbol.chars().count() as i32;
        let start_offset = start.offset();
        let end_offset = end.offset();

        // Check if the selection is immediately surrounded by the symbol
        let has_outer_wrap = if start_offset >= symbol_char_count {
            let outer_start = buffer.iter_at_offset(start_offset - symbol_char_count);
            let outer_end = buffer.iter_at_offset(end_offset + symbol_char_count);
            
            let outer_text_start = buffer.text(&outer_start, &start, false).to_string();
            let outer_text_end = buffer.text(&end, &outer_end, false).to_string();
            
            outer_text_start == symbol && outer_text_end == symbol
        } else {
            false
        };

        if has_outer_wrap {
            // Unwrap outer
            buffer.begin_user_action();
            
            // Delete trailing symbol
            let mut del_end_start = buffer.iter_at_offset(end_offset);
            let mut del_end_end = buffer.iter_at_offset(end_offset + symbol_char_count);
            buffer.delete(&mut del_end_start, &mut del_end_end);
            
            // Delete leading symbol
            let mut del_start_start = buffer.iter_at_offset(start_offset - symbol_char_count);
            let mut del_start_end = buffer.iter_at_offset(start_offset);
            buffer.delete(&mut del_start_start, &mut del_start_end);
            
            // Restore selection
            let sel_start = buffer.iter_at_offset(start_offset - symbol_char_count);
            let sel_end = buffer.iter_at_offset(end_offset - symbol_char_count);
            buffer.select_range(&sel_start, &sel_end);
            
            buffer.end_user_action();
        } else if text.starts_with(symbol) && text.ends_with(symbol) && text.len() >= symbol.len() * 2 {
            // Unwrap inner
            let inner_text = &text[symbol.len()..text.len() - symbol.len()];
            buffer.begin_user_action();
            buffer.delete(&mut start, &mut end);
            let mut it = buffer.iter_at_offset(start_offset);
            buffer.insert(&mut it, inner_text);
            
            let sel_start = buffer.iter_at_offset(start_offset);
            let sel_end = buffer.iter_at_offset(start_offset + inner_text.chars().count() as i32);
            buffer.select_range(&sel_start, &sel_end);
            
            buffer.end_user_action();
        } else {
            // Wrap: snapshot end offset before any modification
            buffer.begin_user_action();
            let mut start_it = buffer.iter_at_offset(start_offset);
            buffer.insert(&mut start_it, symbol);
            
            let mut end_it = buffer.iter_at_offset(end_offset + symbol_char_count);
            buffer.insert(&mut end_it, symbol);
            
            let sel_start = buffer.iter_at_offset(start_offset + symbol_char_count);
            let sel_end = buffer.iter_at_offset(end_offset + symbol_char_count);
            buffer.select_range(&sel_start, &sel_end);
            
            buffer.end_user_action();
        }
    }
}