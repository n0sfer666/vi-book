module.exports = grammar({
  name: 'toy',
  extras: $ => [/\s/, $.comment],
  rules: {
    program: $ => repeat($.statement),
    statement: $ => seq('let', $.identifier, '=', $.number, ';'),
    identifier: _ => /[a-z]+/,
    number: _ => /[0-9]+/,
    comment: _ => /#[^\n]*/,
  },
});
