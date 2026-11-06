module.exports = grammar({
  name: 'mars',

  rules: {
    document: $ => $._value,

    _value: $ =>
      choice(
        $.object,
        $.array,
        $.string,
        $.number,
        $.boolean,
        $.null,
      ),

    object: $ =>
      seq(
        '{',
        optional(commaSep($.pair)),
        '}',
      ),

    pair: $ =>
      seq(
        $.string,
        ':',
        $._value,
      ),

    array: $ =>
      seq(
        '[',
        optional(commaSep($._value)),
        ']',
      ),

    string: $ =>
      seq(
        '"',
        repeat(choice(
          $.escape_sequence,
          /[^"\\]+/,
        )),
        '"',
      ),

    escape_sequence: $ =>
      seq(
        '\\',
        choice(
          '"', '\\', '/', 'b', 'f', 'n', 'r', 't',
          seq('u', repeat(/, /, hex_digit, hex_digit, hex_digit, hex_digit)),
        ),
      ),

    number: $ =>
      choice(
        seq(
          optional('-'),
          choice(
            '0',
            /[1-9][0-9]*/,
          ),
          optional(seq('.', /[0-9]+/)),
          optional(choice('e', 'E')),
          optional(choice('+', '-')),
          /\d+/,
        ),
      ),

    boolean: $ => choice('true', 'false'),

    null: $ => 'null',

    // Mars-specific keywords for semantic highlighting
    format_version: $ => seq('"format_version"', ':', $.number),
    metadata: $ => seq('"metadata"', ':', $.object),
    cells: $ => seq('"cells"', ':', $.array),
    cell_type: $ => seq('"cell_type"', ':', $.string),
    source: $ => seq('"source"', ':', $.string),
    execution_count: $ => seq('"execution_count"', ':', choice($.number, 'null')),
    language: $ => seq('"language"', ':', $.string),
    title: $ => seq('"title"', ':', $.string),
    dependencies: $ => seq('"dependencies"', ':', $.object),
  },
});

function commaSep(rule) {
  return seq(rule, repeat(seq(',', rule)));
}

function hex_digit() {
  return /[0-9a-fA-F]/;
}
