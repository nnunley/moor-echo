// Echo Language Grammar for Tree-sitter
// Based on ECHO language design document

module.exports = grammar({
  name: 'echo',

  extras: $ => [
    /\s/,
    $.comment,
  ],

  precedences: $ => [
    [
      'member',
      'call',
      'unary',
      'multiplicative',
      'additive',
      'relational',
      'equality',
      'and',
      'or',
      'assignment',
    ],
  ],

  rules: {
    source_file: $ => repeat($._definition),

    _definition: $ => choice(
      $.object_definition,
      $.event_definition,
      $.capability_definition,
    ),

    // Comments
    comment: $ => token(choice(
      seq('//', /.*/),
      seq(
        '/*',
        /[^*]*\*+([^/*][^*]*\*+)*/,
        '/'
      )
    )),

    // Object definition
    object_definition: $ => seq(
      'object',
      $.identifier,
      optional(seq('extends', $._expression)),
      repeat($._object_member),
      'endobject'
    ),

    _object_member: $ => choice(
      $.property_definition,
      $.function_definition,
      $.verb_definition,
      $.query_definition,
      $.event_handler,
    ),

    // Property definition
    property_definition: $ => seq(
      'property',
      $.identifier,
      optional(seq('(', $.property_options, ')')),
      optional(seq('=', $._expression)),
      ';'
    ),

    property_options: $ => sepBy(',', $.identifier),

    // Function definition
    function_definition: $ => seq(
      optional('secure'),
      'function',
      $.identifier,
      '(',
      optional($.parameter_list),
      ')',
      optional(seq('requires', $.capability_list)),
      $.statement_block,
      'endfunction'
    ),

    // Verb definition
    verb_definition: $ => seq(
      optional('secure'),
      'verb',
      $.string_literal,
      '(',
      $.verb_signature,
      ')',
      optional($.handles_clause),
      optional(seq('requires', $.capability_list)),
      $.statement_block,
      'endverb'
    ),

    verb_signature: $ => seq(
      $._expression,
      ',',
      $._expression,
      ',',
      $._expression
    ),

    handles_clause: $ => seq(
      'handles',
      'intent',
      $.string_literal,
      optional(seq('with', 'confidence', $._expression))
    ),

    // Query definition
    query_definition: $ => seq(
      'query',
      $.query_head,
      ':-',
      $.query_body,
      ';'
    ),

    query_head: $ => seq(
      $.identifier,
      '(',
      optional($.parameter_list),
      ')'
    ),

    query_body: $ => $._expression,

    // Event handler
    event_handler: $ => seq(
      'on',
      choice($.event_pattern, $.error_event_pattern),
      optional(seq('where', $._expression)),
      $.statement_block,
      'endon'
    ),

    event_pattern: $ => seq(
      $.identifier,
      '(',
      optional($.parameter_list),
      ')'
    ),

    error_event_pattern: $ => seq(
      'error',
      'thrown',
      'as',
      $.identifier,
      '(',
      $.identifier,
      ')'
    ),

    // Event and capability definitions
    event_definition: $ => seq(
      'define',
      'event',
      $.identifier,
      '(',
      optional($.parameter_list),
      ')',
      ';'
    ),

    capability_definition: $ => seq(
      'define',
      'capability',
      $.identifier,
      '(',
      optional($.parameter_list),
      ')',
      ';'
    ),

    // Statements
    statement_block: $ => repeat1($._statement),

    _statement: $ => choice(
      $.if_statement,
      $.while_statement,
      $.for_statement,
      $.try_statement,
      $.gather_statement,
      $.return_statement,
      $.emit_statement,
      $.grant_statement,
      $.let_statement,
      $.expression_statement,
    ),

    if_statement: $ => seq(
      'if',
      $._expression,
      $.statement_block,
      repeat($.elseif_clause),
      optional($.else_clause),
      'endif'
    ),

    elseif_clause: $ => seq(
      'elseif',
      $._expression,
      $.statement_block
    ),

    else_clause: $ => seq(
      'else',
      $.statement_block
    ),

    while_statement: $ => seq(
      'while',
      $._expression,
      $.statement_block,
      'endwhile'
    ),

    for_statement: $ => seq(
      'for',
      $.identifier,
      'in',
      $._expression,
      $.statement_block,
      'endfor'
    ),

    try_statement: $ => seq(
      'try',
      $.statement_block,
      repeat($.catch_clause),
      optional(seq('finally', $.statement_block)),
      'endtry'
    ),

    catch_clause: $ => seq(
      'catch',
      $.identifier,
      '(',
      $.identifier,
      ')',
      $.statement_block
    ),

    gather_statement: $ => seq(
      'gather',
      $.statement_block,
      'endgather'
    ),

    return_statement: $ => seq(
      'return',
      optional($._expression),
      ';'
    ),

    emit_statement: $ => seq(
      'emit',
      $._expression,
      ';'
    ),

    grant_statement: $ => seq(
      'grant',
      $._expression,
      'to',
      $._expression,
      ';'
    ),

    let_statement: $ => seq(
      'let',
      $.binding_pattern,
      '=',
      $._expression,
      ';'
    ),

    expression_statement: $ => seq(
      $._expression,
      ';'
    ),

    // Expressions
    _expression: $ => choice(
      $.identifier,
      $.number,
      $.string_literal,
      $.boolean,
      $.null,
      $.object_literal,
      $.list_literal,
      $.lambda_expression,
      $.member_expression,
      $.call_expression,
      $.binary_expression,
      $.unary_expression,
      $.assignment_expression,
      $.parenthesized_expression,
    ),

    member_expression: $ => prec.left('member', seq(
      $._expression,
      choice('.', ':'),
      $.identifier
    )),

    call_expression: $ => prec.left('call', seq(
      $._expression,
      '(',
      optional($.argument_list),
      ')'
    )),

    binary_expression: $ => choice(
      prec.left('multiplicative', seq($._expression, choice('*', '/', '%'), $._expression)),
      prec.left('additive', seq($._expression, choice('+', '-'), $._expression)),
      prec.left('relational', seq($._expression, choice('<', '>', '<=', '>='), $._expression)),
      prec.left('equality', seq($._expression, choice('==', '!='), $._expression)),
      prec.left('and', seq($._expression, '&&', $._expression)),
      prec.left('or', seq($._expression, '||', $._expression)),
    ),

    unary_expression: $ => prec('unary', seq(
      choice('!', '-', '+'),
      $._expression
    )),

    assignment_expression: $ => prec.right('assignment', seq(
      $._expression,
      '=',
      $._expression
    )),

    parenthesized_expression: $ => seq(
      '(',
      $._expression,
      ')'
    ),

    // Literals
    object_literal: $ => seq(
      '{',
      optional(sepBy(',', $.property_assignment)),
      '}'
    ),

    property_assignment: $ => seq(
      $.identifier,
      ':',
      $._expression
    ),

    list_literal: $ => seq(
      '[',
      optional(sepBy(',', $._expression)),
      ']'
    ),

    lambda_expression: $ => seq(
      'fn',
      '(',
      optional($.parameter_list),
      ')',
      $._expression
    ),

    // Patterns
    binding_pattern: $ => choice(
      $.identifier,
      $.destructuring_pattern,
    ),

    destructuring_pattern: $ => seq(
      '{',
      sepBy(',', $.pattern_property),
      '}'
    ),

    pattern_property: $ => choice(
      $.identifier,
      seq('?', $.identifier),
      seq('@', $.identifier),
    ),

    // Basic elements
    parameter_list: $ => sepBy(',', $.parameter),

    parameter: $ => seq(
      optional(choice('?', '@')),
      $.identifier,
      optional(seq(':', $.type_annotation))
    ),

    argument_list: $ => sepBy(',', $._expression),

    capability_list: $ => sepBy(',', $.capability_reference),

    capability_reference: $ => seq(
      $.identifier,
      optional(seq('(', $.argument_list, ')'))
    ),

    type_annotation: $ => $.identifier,

    identifier: $ => /[a-zA-Z_$][a-zA-Z0-9_$]*/,

    number: $ => /\d+(\.\d+)?/,

    string_literal: $ => seq(
      '"',
      repeat(choice(
        /[^"\\]+/,
        /\\./
      )),
      '"'
    ),

    boolean: $ => choice('true', 'false'),

    null: $ => 'null',
  }
});

function sepBy(sep, rule) {
  return optional(sepBy1(sep, rule));
}

function sepBy1(sep, rule) {
  return seq(rule, repeat(seq(sep, rule)));
}