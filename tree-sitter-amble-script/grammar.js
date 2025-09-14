const PREC = {
  COMMENT: -1,
};

module.exports = grammar({
  name: 'amble_dsl',

  extras: $ => [
    $.comment,
    /[\s\r\n\t]/,
  ],

  // no externals

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

    // Treat keywords as words within identifiers for editor word-selection
    word: $ => $.identifier,

    identifier: $ => /[A-Za-z0-9_\-:#]+/,
    number: $ => /-?\d+/, 
    // Strings: single-line '…' and "…", multi-line """…""" and '''…''', and raw r#"…"#
    string: $ => choice(
      token(/\"([^\"\\\n]|\\.)*\"/),
      token(/'([^'\\\n]|\\.)*'/),
      token(seq('"""', repeat(choice(/[^\"]/ , /\"[^\"]/ , /\"\"[^\"]/ , /\\./)), '"""')),
      token(seq("'''", repeat(choice(/[^']/ , /'[^']/ , /''[^']/ , /\\./)), "'''")),
      token(seq('r#"', repeat(choice(/[^\"]/ , /\"[^#]/)), '"#'))
    ),
    boolean: $ => choice('true', 'false'),

    set_decl: $ => seq('let', 'set', $.identifier, '=', $.set_list),
    set_list: $ => seq('(', sep1($.identifier, ','), ')'),

    // Triggers: loose structure to support highlighting of when/if/do and nested blocks
    trigger: $ => seq('trigger', choice($.identifier, $.string), repeat($.trigger_mod), $.trigger_block),
    trigger_mod: $ => choice(seq('only', 'once'), seq('when', $.cond_line)),
    trigger_block: $ => seq('{', repeat($.trigger_stmt), '}'),
    trigger_stmt: $ => choice($.if_block, $.do_stmt),
    if_block: $ => seq('if', $.cond_line_ext, $.braced_block),
    do_stmt: $ => seq('do', repeat1(choice($.identifier, $.string, $.number)), optional($.braced_block)),
    braced_block: $ => seq('{', repeat($.trigger_stmt), '}'),
    cond_line: $ => repeat1(choice($.identifier, $.string, $.number)),
    // Extended cond line permits grouping tokens
    cond_line_ext: $ => repeat1(choice($.identifier, $.string, $.number, '(', ')', ',')),

    room_def: $ => seq('room', $.identifier, $.room_block),
    room_block: $ => seq('{', repeat($.room_stmt), '}'),
    room_stmt: $ => choice(
      $.room_name,
      $.room_desc,
      $.room_visited,
      $.overlay_stmt,
      $.exit_stmt
    ),
    room_name: $ => seq('name', $.string),
    room_desc: $ => seq(choice('desc', 'description'), $.string),
    room_visited: $ => seq('visited', $.boolean),
    // Overlay statements
    overlay_stmt: $ => seq('overlay', 'if', $.cond_line_ext, $.overlay_block),
    overlay_block: $ => seq('{', repeat($.overlay_entry), '}'),
    overlay_entry: $ => choice(
      seq('set', $.string),
      seq('unset', $.string),
      seq('text', $.string),
      seq('normal', $.string),
      seq('happy', $.string),
      seq('bored', $.string)
    ),
    // Exit with optional attribute block
    exit_stmt: $ => seq('exit', choice($.identifier, $.string), '->', $.identifier, optional($.exit_block)),
    exit_block: $ => seq('{', repeat($.exit_attr), '}'),
    exit_attr: $ => seq(choice(
      $.exit_required_flags,
      $.exit_required_items,
      $.exit_barred
    ), optional(',')),
    exit_required_flags: $ => seq('required_flags', '(', sep1($.identifier, ','), ')'),
    exit_required_items: $ => seq('required_items', '(', sep1($.identifier, ','), ')'),
    exit_barred: $ => seq('barred', $.string),

    item_def: $ => seq('item', $.identifier, $.item_block),
    item_block: $ => seq('{', repeat($.item_stmt), '}'),
    item_stmt: $ => choice(
      $.item_name,
      $.item_desc,
      $.item_portable,
      $.item_text,
      $.item_location,
      $.item_ability,
      $.item_container_state,
      $.item_restricted
    ),
    item_name: $ => seq('name', $.string),
    item_desc: $ => seq(choice('desc', 'description'), $.string),
    item_portable: $ => seq('portable', $.boolean),
    item_text: $ => seq('text', $.string),
    item_location: $ => $.location,
    item_ability: $ => seq('ability', $.identifier),
    item_container_state: $ => seq('container', 'state', choice('open', 'closed')),
    item_restricted: $ => seq('restricted', $.boolean),

    spinner_def: $ => seq('spinner', $.identifier, $.spinner_block),
    spinner_block: $ => seq('{', repeat($.wedge_stmt), '}'),
    wedge_stmt: $ => seq('wedge', $.string, optional(seq('width', $.number))),

    npc_def: $ => seq('npc', $.identifier, $.npc_block),
    npc_block: $ => seq('{', repeat($.npc_stmt), '}'),
    npc_stmt: $ => choice(
      $.npc_name,
      $.npc_desc,
      $.location,
      $.npc_state,
      $.movement_stmt,
      $.dialogue_stmt
    ),
    npc_name: $ => seq('name', $.string),
    npc_desc: $ => seq(choice('desc', 'description'), $.string),
    npc_state: $ => choice(
      seq('state', choice('normal','happy','bored','mad')),
      seq('state', 'custom', $.identifier)
    ),

    // Movement: movement random|route rooms (id, id, ...) [timing <ident>] [active true|false]
    movement_stmt: $ => seq(
      'movement',
      choice('random','route'),
      'rooms', '(', sep1($.identifier, ','), ')',
      optional(seq('timing', $.identifier)),
      optional(seq('active', $.boolean))
    ),

    // Dialogue blocks: dialogue <state|custom ident> { "..."* }
    dialogue_stmt: $ => seq(
      'dialogue',
      choice(
        'normal','happy','bored','mad',
        seq('custom', $.identifier)
      ),
      '{', repeat($.string), '}'
    ),
    // Reusable location rule (for NPCs and Items)
    location: $ => seq('location', choice(
      seq('room', $.identifier),
      seq('npc', $.identifier),
      seq('chest', $.identifier),
      seq('inventory', 'player'),
      seq('nowhere', $.string)
    )),

    // Goals are a header line followed by zero or more goal statements on subsequent lines.
    // Inline the repetition here so we don't have a named rule that can match empty.
    goal_def: $ => seq('goal', $.identifier, $.string, repeat($.goal_stmt)),
    goal_stmt: $ => choice($.goal_desc, $.goal_group, $.goal_done, $.goal_start),
    goal_desc: $ => seq('desc', $.string),
    goal_group: $ => seq('group', choice('required', 'optional', 'status-effect')),
    goal_done: $ => seq('done', 'when', $.goal_cond),
    goal_start: $ => seq('start', 'when', $.goal_cond),
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
