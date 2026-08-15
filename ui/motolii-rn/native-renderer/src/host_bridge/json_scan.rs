pub(super) fn json_string_value(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let mut search = json;
    let mut abs = 0usize;
    while let Some(at) = search.find(&needle) {
        abs += at;
        let after = &json[abs + needle.len()..];
        let trimmed = after.trim_start();
        if let Some(rest) = trimmed.strip_prefix(':') {
            let rest = rest.trim_start();
            if rest.starts_with("null") {
                return None;
            }
            if let Some(body) = rest.strip_prefix('"') {
                let (decoded, _) = scan_json_string(body)?;
                return Some(decoded);
            }
        }
        abs += needle.len();
        search = &json[abs..];
    }
    None
}

pub(super) fn scan_json_string(input: &str) -> Option<(String, usize)> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut out = String::new();
    let mut i = 0usize;
    while i < len {
        let b = bytes[i];
        if b == b'\\' {
            if i + 1 >= len {
                return None;
            }
            let esc = bytes[i + 1];
            match esc {
                b'"' => {
                    out.push('"');
                    i += 2;
                }
                b'\\' => {
                    out.push('\\');
                    i += 2;
                }
                b'/' => {
                    out.push('/');
                    i += 2;
                }
                b'b' => {
                    out.push('\u{0008}');
                    i += 2;
                }
                b'f' => {
                    out.push('\u{000C}');
                    i += 2;
                }
                b'n' => {
                    out.push('\n');
                    i += 2;
                }
                b'r' => {
                    out.push('\r');
                    i += 2;
                }
                b't' => {
                    out.push('\t');
                    i += 2;
                }
                b'u' => {
                    if i + 6 > len {
                        return None;
                    }
                    let mut codepoint = parse_hex_u16(input, i + 2)? as u32;
                    i += 6;
                    if (0xD800..=0xDBFF).contains(&(codepoint as u16)) {
                        if i + 6 > len || bytes[i] != b'\\' || bytes[i + 1] != b'u' {
                            return None;
                        }
                        let next_codepoint = parse_hex_u16(input, i + 2)?;
                        if !(0xDC00..=0xDFFF).contains(&next_codepoint) {
                            return None;
                        }
                        let high = codepoint as u32 - 0xD800;
                        let low = next_codepoint as u32 - 0xDC00;
                        codepoint = 0x10000 + ((high << 10) | low);
                        i += 6;
                    }
                    let Some(ch) = std::char::from_u32(codepoint) else {
                        return None;
                    };
                    out.push(ch);
                }
                _ => return None,
            }
            continue;
        }
        if b == b'"' {
            return Some((out, i));
        }
        let next = input[i..].chars().next()?;
        out.push(next);
        i += next.len_utf8();
    }
    None
}

pub(super) fn parse_hex_u16(input: &str, start: usize) -> Option<u16> {
    let end = start + 4;
    if start >= input.len() || end > input.len() {
        return None;
    }
    u16::from_str_radix(&input[start..end], 16).ok()
}

pub(super) fn json_u32_value(json: &str, key: &str) -> Option<u32> {
    let value = json_i64_value(json, key)?;
    u32::try_from(value).ok()
}

pub(super) fn json_f64_value(json: &str, key: &str) -> Option<f64> {
    let needle = format!("\"{key}\"");
    let mut search = json;
    let mut abs = 0usize;
    while let Some(at) = search.find(&needle) {
        abs += at;
        let after = &json[abs + needle.len()..];
        let trimmed = after.trim_start();
        if let Some(rest) = trimmed.strip_prefix(':') {
            let (value, _) = parse_json_f64(rest.trim_start())?;
            return Some(value);
        }
        abs += needle.len();
        search = &json[abs..];
    }
    None
}

pub(super) fn is_finite_f32_compatible(value: f64) -> bool {
    value.is_finite() && value.abs() <= f64::from(f32::MAX)
}

pub(super) fn parse_json_f64(input: &str) -> Option<(f64, &str)> {
    let trimmed = input.trim_start();
    let end = trimmed
        .find(|ch: char| !(ch.is_ascii_digit() || matches!(ch, '-' | '+' | '.' | 'e' | 'E')))
        .unwrap_or(trimmed.len());
    if end == 0 {
        return None;
    }
    let value = trimmed[..end].parse().ok()?;
    Some((value, &trimmed[end..]))
}

pub(super) fn json_bool_value(json: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{key}\"");
    let mut search = json;
    let mut abs = 0usize;
    while let Some(at) = search.find(&needle) {
        abs += at;
        let after = &json[abs + needle.len()..];
        let trimmed = after.trim_start();
        if let Some(rest) = trimmed.strip_prefix(':') {
            let rest = rest.trim_start();
            if rest.starts_with("true") {
                return Some(true);
            }
            if rest.starts_with("false") {
                return Some(false);
            }
            return None;
        }
        abs += needle.len();
        search = &json[abs..];
    }
    None
}

pub(super) fn find_matching_bracket(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    for (i, ch) in s.char_indices() {
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

pub(super) fn parse_optional_vec2(obj: &str) -> Option<[f64; 2]> {
    let marker = "\"value\"";
    let at = obj.find(marker)?;
    let after = obj[at + marker.len()..].trim_start().strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let (x, after_x) = parse_json_f64(after)?;
    let after_x = after_x.trim_start();
    if !after_x.starts_with(',') {
        return None;
    }
    let (y, after_y) = parse_json_f64(&after_x[1..])?;
    let after_y = after_y.trim_start();
    if !after_y.starts_with(']') {
        return None;
    }
    Some([x, y])
}

pub(super) fn find_key_object<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\"");
    let mut search = json;
    let mut abs = 0usize;
    while let Some(at) = search.find(&needle) {
        abs += at;
        let after = &json[abs + needle.len()..];
        let trimmed = after.trim_start();
        if let Some(rest) = trimmed.strip_prefix(':') {
            let rest = rest.trim_start();
            if rest.starts_with('{') {
                let end = find_matching_brace(rest)?;
                return Some(&rest[..=end]);
            }
        }
        abs += needle.len();
        search = &json[abs..];
    }
    None
}

pub(super) fn find_root_key_array<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\"");
    let bytes = json.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    let mut i = 0usize;
    while i < bytes.len() {
        if in_string {
            if escape {
                escape = false;
            } else if bytes[i] == b'\\' {
                escape = true;
            } else if bytes[i] == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b'"' if depth == 1 && json[i..].starts_with(&needle) => {
                let after = json[i + needle.len()..].trim_start().strip_prefix(':')?;
                let array = after.trim_start();
                let end = find_matching_bracket(array)?;
                return Some(&array[..=end]);
            }
            b'"' => in_string = true,
            _ => {}
        }
        i += 1;
    }
    None
}

pub(super) fn json_rational(json: &str, key: &str) -> Option<(i64, i64)> {
    let obj = find_key_object(json, key)?;
    let num = json_i64_value(obj, "num")?;
    let den = json_i64_value(obj, "den")?;
    Some((num, den))
}

pub(super) fn json_i64_value(json: &str, key: &str) -> Option<i64> {
    let needle = format!("\"{key}\"");
    let mut search = json;
    let mut abs = 0usize;
    while let Some(at) = search.find(&needle) {
        abs += at;
        let after = &json[abs + needle.len()..];
        let trimmed = after.trim_start();
        if let Some(rest) = trimmed.strip_prefix(':') {
            let rest = rest.trim_start();
            let end = rest
                .find(|ch: char| !(ch.is_ascii_digit() || ch == '-' || ch == '+'))
                .unwrap_or(rest.len());
            if end == 0 {
                return None;
            }
            return rest[..end].parse().ok();
        }
        abs += needle.len();
        search = &json[abs..];
    }
    None
}

pub(super) fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    for (i, ch) in s.char_indices() {
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}
