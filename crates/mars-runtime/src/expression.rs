use crate::output::ExpressionOutput;

pub fn render_program(cells: &[String]) -> String {
    render_program_with_last_output(cells, ExpressionOutput::Debug)
}

pub(crate) fn render_program_with_last_output(
    cells: &[String],
    output: ExpressionOutput,
) -> String {
    let mut program = String::from("fn main() {\n");
    let last_index = cells.len().saturating_sub(1);
    for (index, cell) in cells.iter().enumerate() {
        program.push_str(&format!("    // Mars cell {index}\n"));
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

pub(crate) fn split_final_expression(source: &str) -> Option<(&str, &str)> {
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

fn render_cell_with_output(source: &str, output: ExpressionOutput) -> String {
    let stripped = strip_trailing_line_comments(source);
    if let Some((prefix, expression)) = split_final_expression(&stripped) {
        return match output {
            ExpressionOutput::Debug => {
                format!("{prefix}println!(\"__MARS_RESULT__={{:?}}\", ({expression}));")
            }
            ExpressionOutput::Display => {
                format!("{prefix}println!(\"__MARS_RESULT__={{}}\", ({expression}));")
            }
            ExpressionOutput::Discard => format!("{prefix}let _ = ({expression});"),
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
    fn does_not_wrap_replayed_final_expressions() {
        let cells = vec!["1 + 1".to_owned(), "2 + 2".to_owned()];
        let program = render_program(&cells);

        assert!(program.contains("    // Mars cell 0\n    1 + 1\n"));
        assert!(program.contains("println!(\"__MARS_RESULT__={:?}\", (2 + 2));"));
    }

    #[test]
    fn renders_empty_cells_without_panicking() {
        assert_eq!(render_program(&[]), "fn main() {\n}\n");
    }
}
