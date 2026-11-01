use crate::util::escape_html;

const RUST_KEYWORDS: &[&str] = &[
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
    "ref mut",
    "type",
    "as",
    "box",
    "true",
    "false",
    "None",
    "Some",
    "Ok",
    "Err",
];

const RUST_TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char", "str", "String", "Vec", "Option", "Result", "Box", "Rc", "Arc", "Cell",
    "RefCell", "Mutex", "RwLock",
];

pub fn highlight_rust(source: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];

        if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            let end = source[i..].find('\n').map(|n| i + n).unwrap_or(chars.len());
            out.push_str(&format!(
                "<span class=\"rust-comment\">{}</span>",
                escape_html(&source[i..end])
            ));
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
            out.push_str(&format!(
                "<span class=\"rust-string\">{}</span>",
                escape_html(&source[i..end])
            ));
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
            out.push_str(&format!(
                "<span class=\"rust-string\">{}</span>",
                escape_html(&source[i..end])
            ));
            i = end;
            continue;
        }

        if ch.is_alphabetic() || ch == '_' {
            let mut end = i;
            while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                end += 1;
            }
            let word = &source[i..end];
            let span_class = if RUST_KEYWORDS.contains(&word) {
                "rust-keyword"
            } else if RUST_TYPES.contains(&word) {
                "rust-type"
            } else if end < chars.len() && chars[end] == '!' {
                out.push_str(&format!(
                    "<span class=\"rust-macro\">{}</span>",
                    escape_html(word)
                ));
                i = end;
                continue;
            } else {
                "rust-ident"
            };
            out.push_str(&format!(
                "<span class=\"{span_class}\">{}</span>",
                escape_html(word)
            ));
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
            out.push_str(&format!(
                "<span class=\"rust-number\">{}</span>",
                escape_html(&source[i..end])
            ));
            i = end;
            continue;
        }

        out.push(ch);
        i += 1;
    }
    out
}
