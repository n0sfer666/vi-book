module.exports = grammar({
  name: 'toy',
  extras: $ => [/\s/, $.comment],
  rules: {
    program: $ => repeat($.statement),
    statement: $ => seq('let', $.identifier, '=', choice($.number, $.string), ';'),
    identifier: _ => /[a-z]+/,
    number: _ => /[0-9]+/,
    string: _ => /"[^"\n]*"/,
    comment: _ => /#[^\n]*/,
  },
});
