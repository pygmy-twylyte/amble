const PREC = {
  COMMENT: -1,
};

module.exports = grammar({
  name: 'amble_dsl',

  extras: $ => [
    $.comment,
    /[\s\r\n\t]/,
  ],

  rules: {
    program: $ => repeat(choice(
      $.set_decl,
      $.trigger,
      $.room_def,
      $.item_def,
      $.spinner_def,
      $.npc_def,
      $.goal_def
    )),

    comment: $ => token(seq('#', /.*/)),

    identifier: $ => /[A-Za-z0-9_\-:#]+/,
    number: $ => /-?\d+/, 
    string: $ => choice(
      /"([^"\\]|\\.)*"/,
      /'([^'\\]|\\.)*'/,
      seq('"""', repeat(/[^"\\]|\\.|\"(?!\"\")/), '"""')
    ),
    boolean: $ => choice('true', 'false'),

    set_decl: $ => seq('let', 'set', $.identifier, '=', $.set_list),
    set_list: $ => seq('(', sep1($.identifier, ','), ')'),

    trigger: $ => seq('trigger', $.identifier, '{', repeat($._trigger_stmt), '}'),
    _trigger_stmt: $ => seq($.identifier, $.string), // placeholder

    room_def: $ => seq('room', $.identifier, $.room_block),
    room_block: $ => seq('{', repeat($.room_stmt), '}'),
    room_stmt: $ => choice(
      $.room_name,
      $.room_desc,
      $.room_visited,
      $.exit_stmt
    ),
    room_name: $ => seq('name', $.string),
    room_desc: $ => seq(choice('desc', 'description'), $.string),
    room_visited: $ => seq('visited', $.boolean),
    exit_stmt: $ => seq('exit', $.identifier, '->', $.identifier),

    item_def: $ => seq('item', $.identifier, $.item_block),
    item_block: $ => seq('{', repeat($.item_stmt), '}'),
    item_stmt: $ => choice($.item_name, $.item_desc, $.item_portable),
    item_name: $ => seq('name', $.string),
    item_desc: $ => seq(choice('desc', 'description'), $.string),
    item_portable: $ => seq('portable', $.boolean),

    spinner_def: $ => seq('spinner', $.identifier, $.spinner_block),
    spinner_block: $ => seq('{', repeat($.wedge_stmt), '}'),
    wedge_stmt: $ => seq('wedge', $.string, optional(seq('width', $.number))),

    npc_def: $ => seq('npc', $.identifier, $.npc_block),
    npc_block: $ => seq('{', repeat($.npc_stmt), '}'),
    npc_stmt: $ => choice($.npc_name, $.npc_desc, $.npc_location),
    npc_name: $ => seq('name', $.string),
    npc_desc: $ => seq(choice('desc', 'description'), $.string),
    npc_location: $ => seq('location', choice(seq('room', $.identifier), seq('nowhere', $.string))),

    goal_def: $ => seq('goal', $.identifier, $.string, $.goal_block),
    goal_block: $ => repeat($.goal_stmt),
    goal_stmt: $ => choice($.goal_desc, $.goal_group, $.goal_done),
    goal_desc: $ => seq('desc', $.string),
    goal_group: $ => seq('group', choice('required', 'optional', 'status-effect')),
    goal_done: $ => seq('done', 'when', $.goal_cond),
    goal_cond: $ => choice(
      seq('has', 'flag', $.identifier),
      seq('missing', 'flag', $.identifier),
      seq('has', 'item', $.identifier),
      seq('reached', 'room', $.identifier),
      seq('goal', 'complete', $.identifier),
      seq('flag', 'in', 'progress', $.identifier),
      seq('flag', 'complete', $.identifier)
    ),
  }
});

function sep1(rule, delimiter) {
  return seq(rule, repeat(seq(delimiter, rule)));
}
