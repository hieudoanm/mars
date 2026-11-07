use crate::output::ExpressionOutput;

pub fn render_program(cells: &[String]) -> String {
    render_program_with_last_output(cells, ExpressionOutput::Debug)
}

const PROMPT_HELPER: &str = r#"fn mars_prompt(question: &str) -> String {
    println!("__MARS_PROMPT__{question}");
    let mut stdout = std::io::stdout();
    let _ = std::io::Write::flush(&mut stdout);
    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line);
    line.trim_end_matches(['\r', '\n']).to_owned()
}

fn mars_table<C: AsRef<str>, R: AsRef<str>>(columns: &[C], rows: &[Vec<R>]) {
    fn esc(text: &str) -> String {
        let mut out = String::with_capacity(text.len() + 2);
        for ch in text.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out
    }
    let mut line = String::from("__MARS_TABLE__={\"columns\":[");
    for (index, column) in columns.iter().enumerate() {
        if index > 0 {
            line.push(',');
        }
        line.push('"');
        line.push_str(&esc(column.as_ref()));
        line.push('"');
    }
    line.push_str("],\"rows\":[");
    for (row_index, row) in rows.iter().enumerate() {
        if row_index > 0 {
            line.push(',');
        }
        line.push('[');
        for (cell_index, cell) in row.iter().enumerate() {
            if cell_index > 0 {
                line.push(',');
            }
            line.push('"');
            line.push_str(&esc(cell.as_ref()));
            line.push('"');
        }
        line.push(']');
    }
    line.push_str("]}");
    println!("{line}");
}

"#;

/// Stable hash of a cell's source, embedded in the generated `// Mars cell`
/// marker so runtime diagnostics can be mapped back to notebook cells even
/// when cells execute out of order.
pub(crate) fn cell_source_hash(source: &str) -> u32 {
    let mut hash: u32 = 5381;
    for byte in source.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(u32::from(byte));
    }
    hash
}

pub(crate) fn render_program_with_last_output(
    cells: &[String],
    output: ExpressionOutput,
) -> String {
    let mut program = String::from(PROMPT_HELPER);
    program.push_str("fn main() {\n");
    let last_index = cells.len().saturating_sub(1);
    for (index, cell) in cells.iter().enumerate() {
        program.push_str(&format!(
            "    // Mars cell {index} {}\n",
            cell_source_hash(cell)
        ));
        // Only the current cell should auto-display its final expression.
        // Replayed cells keep their own stored outputs, so printing them again
        // would duplicate earlier results in every later execution. Their final
        // expressions are discarded, which also keeps the generated program
        // syntactically valid when a cell ends without a semicolon.
        let cell_output = if index == last_index {
            output
        } else {
            ExpressionOutput::Discard
        };
        let rendered_cell = render_cell_with_output(cell, cell_output);
        for line in rendered_cell.lines() {
            program.push_str("    ");
            program.push_str(line);
            program.push('\n');
        }
    }
    program.push_str("}\n");
    program
}

pub fn split_final_expression(source: &str) -> Option<(&str, &str)> {
    let trimmed = source.trim_end();
    let semicolon = find_last_top_level_semicolon(trimmed);
    let brace = find_last_top_level_closing_brace(trimmed).filter(|index| {
        let suffix = trimmed[index + 1..].trim_start();
        !suffix.is_empty() && !suffix.starts_with(';')
    });
    let boundary = match (semicolon, brace) {
        (Some(semicolon), Some(brace)) => Some(semicolon.max(brace)),
        (Some(semicolon), None) => Some(semicolon),
        (None, Some(brace)) => Some(brace),
        (None, None) => None,
    };
    let (prefix, expression) = boundary.map_or(("", trimmed), |index| trimmed.split_at(index + 1));
    let expression = expression.trim();
    if expression.is_empty() || !is_expression_candidate(expression) {
        return None;
    }
    Some((prefix, expression))
}

fn is_expression_candidate(expression: &str) -> bool {
    let expression = expression.trim();
    let excluded_prefixes = [
        "dbg!",
        "eprintln!",
        "fn ",
        "impl ",
        "let ",
        "macro_rules!",
        "mod ",
        "panic!",
        "println!",
        "return ",
        "struct ",
        "todo!",
        "trait ",
        "type ",
        "use ",
    ];
    !excluded_prefixes
        .iter()
        .any(|prefix| expression.starts_with(prefix))
        && !expression.ends_with(';')
}

fn find_last_top_level_semicolon(source: &str) -> Option<usize> {
    let mut depth = 0_i32;
    let mut string = false;
    let mut escaped = false;
    let mut last = None;
    for (index, character) in source.char_indices() {
        if string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                string = false;
            }
            continue;
        }
        match character {
            '"' => string = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ';' if depth == 0 => last = Some(index),
            _ => {}
        }
    }
    last
}

fn find_last_top_level_closing_brace(source: &str) -> Option<usize> {
    let mut depth = 0_i32;
    let mut string = false;
    let mut escaped = false;
    let mut last = None;
    for (index, character) in source.char_indices() {
        if string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                string = false;
            }
            continue;
        }
        match character {
            '"' => string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    last = Some(index);
                }
            }
            _ => {}
        }
    }
    last
}

fn strip_trailing_line_comments(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut string = false;
    let mut escaped = false;
    let mut chars = source.chars().peekable();
    while let Some(character) = chars.next() {
        if string {
            output.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                string = false;
            }
            continue;
        }
        if character == '"' {
            string = true;
            output.push(character);
        } else if character == '/' && chars.peek() == Some(&'/') {
            let _ = chars.next();
            for trailing in chars.by_ref() {
                if trailing == '\n' {
                    output.push('\n');
                    break;
                }
            }
        } else {
            output.push(character);
        }
    }
    output
}

pub(crate) fn render_cell_with_output(source: &str, output: ExpressionOutput) -> String {
    let stripped = strip_trailing_line_comments(source);
    if let Some((prefix, expression)) = split_final_expression(&stripped) {
        return match output {
            ExpressionOutput::Debug => {
                format!("{prefix}println!(\"__MARS_RESULT__={{:?}}\", ({expression}));")
            }
            ExpressionOutput::Display => {
                format!("{prefix}println!(\"__MARS_RESULT__={{}}\", ({expression}));")
            }
            ExpressionOutput::Discard => format!("{prefix}({expression});"),
        };
    }
    source.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_only_the_current_cell_with_automatic_output() {
        let cells = vec![
            "let numbers = vec![1, 2, 3];".to_owned(),
            "numbers.len()".to_owned(),
        ];
        let program = render_program(&cells);

        assert_eq!(program.matches("__MARS_RESULT__").count(), 1);
        assert!(program.contains("println!(\"__MARS_RESULT__={:?}\", (numbers.len()));"));
    }

    #[test]
    fn evaluates_replayed_final_expressions_without_displaying() {
        let cells = vec!["1 + 1".to_owned(), "2 + 2".to_owned()];
        let program = render_program(&cells);

        let hash = cell_source_hash("1 + 1");
        assert!(program.contains(&format!("    // Mars cell 0 {hash}\n    (1 + 1);\n")));
        assert!(program.contains("println!(\"__MARS_RESULT__={:?}\", (2 + 2));"));
        assert_eq!(program.matches("__MARS_RESULT__").count(), 1);
    }

    #[test]
    fn renders_empty_cells_without_panicking() {
        assert!(render_program(&[]).ends_with("fn main() {\n}\n"));
    }
}
