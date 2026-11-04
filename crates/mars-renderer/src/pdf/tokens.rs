pub fn pdf_escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

pub fn rust_keywords() -> &'static [&'static str] {
    &[
        "fn",
        "let",
        "mut",
        "const",
        "static",
        "struct",
        "enum",
        "trait",
        "impl",
        "pub",
        "use",
        "mod",
        "crate",
        "self",
        "super",
        "Self",
        "if",
        "else",
        "match",
        "loop",
        "while",
        "for",
        "in",
        "break",
        "continue",
        "return",
        "async",
        "await",
        "unsafe",
        "extern",
        "macro",
        "macro_rules",
        "where",
        "dyn",
        "box",
        "move",
        "ref",
        "type",
        "as",
        "true",
        "false",
        "None",
        "Some",
        "Ok",
        "Err",
    ]
}

pub fn rust_types() -> &'static [&'static str] {
    &[
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64", "bool", "char", "str", "String", "Vec", "Option", "Result", "Box", "Rc",
        "Arc", "Cell", "RefCell", "Mutex", "RwLock",
    ]
}

/// Tokenize a single line of Rust source into (text, color) pairs for PDF rendering.
pub fn highlight_rust_tokens(source: &str) -> Vec<(&str, (f64, f64, f64))> {
    let keyword = (0.776, 0.490, 0.870); // #c678dd
    let r#type = (0.878, 0.423, 0.458); // #e06c75
    let comment = (0.360, 0.388, 0.439); // #5c6370
    let string = (0.765, 0.910, 0.553); // #c3e88d bright green
    let number = (0.819, 0.603, 0.400); // #d19a66
    let macro_ = (0.886, 0.752, 0.553); // #e2c08d
    let default = (0.898, 0.898, 0.898); // #e5e5e5

    let mut tokens: Vec<(&str, (f64, f64, f64))> = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];

        if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            let end = source[i..].find('\n').map(|n| i + n).unwrap_or(chars.len());
            tokens.push((&source[i..end], comment));
            i = end;
            continue;
        }

        if ch == '"' {
            let mut end = i + 1;
            while end < chars.len() {
                if chars[end] == '\\' {
                    end += 2;
                    continue;
                }
                if chars[end] == '"' {
                    break;
                }
                end += 1;
            }
            end = end.min(chars.len() - 1);
            if chars[end] == '"' {
                end += 1;
            }
            tokens.push((&source[i..end], string));
            i = end;
            continue;
        }

        if ch == '\'' {
            let mut end = i + 1;
            while end < chars.len() && chars[end] != '\'' {
                end += 1;
            }
            end = end.min(chars.len() - 1);
            if chars[end] == '\'' {
                end += 1;
            }
            tokens.push((&source[i..end], string));
            i = end;
            continue;
        }

        if ch.is_alphabetic() || ch == '_' {
            let mut end = i;
            while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                end += 1;
            }
            let word = &source[i..end];
            let color = if rust_keywords().contains(&word) {
                keyword
            } else if rust_types().contains(&word) {
                r#type
            } else if end < chars.len() && chars[end] == '!' {
                macro_
            } else {
                default
            };
            tokens.push((word, color));
            i = end;
            continue;
        }

        if ch.is_ascii_digit() {
            let mut end = i;
            while end < chars.len()
                && (chars[end].is_ascii_digit() || chars[end] == '.' || chars[end] == '_')
            {
                end += 1;
            }
            tokens.push((&source[i..end], number));
            i = end;
            continue;
        }

        tokens.push((&source[i..i + 1], default));
        i += 1;
    }
    tokens
}
