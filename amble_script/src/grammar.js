/**
 * @file DSL for the Amble game engine.
 * @author djvb <djvbmd@protonmail.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "amble",

  word: ($) => $.identifier,
  extras: ($) => [$.comment, /[\s\r\n\t]/],

  rules: {
    // TODO: add the actual grammar rules
    source_file: ($) =>
      repeat1(
        choice(
          $.set_decl,
          $.room_def,
          $.item_def,
          $.npc_def,
          $.trigger_def,
          $.spinner_def,
          $.goal_def,
        ),
      ),

    comment: ($) => token(seq("#", /.*/)),

    // Lower precedence so keywords like 'has', 'ambient', etc. win over identifiers
    identifier: ($) => token(prec(-1, /[a-zA-Z0-9_\-:#]+/)),

    number: ($) => /-?\d+/,

    // Strings: single-line '…' and "…", multi-line """…""" and '''…''', and raw r#"…"#
    string: ($) =>
      choice(
        token(/\"([^\"\\\n]|\\.)*\"/),
        token(/'([^'\\\n]|\\.)*'/),
        token(
          seq(
            '"""',
            repeat(choice(/[^\"]/, /\"[^\"]/, /\"\"[^\"]/, /\\./)),
            '"""',
          ),
        ),
        token(
          seq("'''", repeat(choice(/[^']/, /'[^']/, /''[^']/, /\\./)), "'''"),
        ),
        token(seq('r#"', repeat(choice(/[^\"]/, /\"[^#]/)), '"#')),
      ),

    boolean: ($) => choice("true", "false"),

    //
    //
    //
    //
    //
    // ROOM SET DECLARATIONS
    //
    //
    //
    //
    //
    set_decl: ($) => seq("let", "set", $.identifier, "=", $.set_list),
    set_list: ($) => seq("(", sep1(alias($.identifier, $.room_id), ","), ")"),

    //
    //
    //
    //
    //
    // ROOM DEFINITIONS
    //
    //
    //
    //
    //
    room_def: ($) =>
      seq(
        "room",
        field("room_id", alias($.identifier, $.room_id)),
        $.room_block,
      ),

    room_block: ($) => seq("{", repeat($._room_stmt), "}"),

    _room_stmt: ($) =>
      choice(
        $.room_name,
        $.room_desc,
        $.room_visited,
        $.room_exit,
        $.ovl_flag_binary,
        $.ovl_presence_pair,
        $.ovl_npc_state_set,
        $.overlay_stmt,
      ),

    room_name: ($) =>
      seq("name", field("name", alias($.string, $.entity_name))),

    room_desc: ($) =>
      seq(
        choice("desc", "description"),
        field("description", alias($.string, $.entity_desc)),
      ),

    room_visited: ($) =>
      seq("visited", field("visited", alias($.boolean, $.room_visited))),

    // ----------------- room exits ----------------
    room_exit: ($) =>
      seq(
        "exit",
        field("dir", alias(choice($.identifier, $.string), $.exit_dir)),
        "->",
        field("dest", alias($.identifier, $.exit_dest)),
        optional($.exit_block),
      ),

    exit_block: ($) => seq("{", repeat1(seq($.exit_stmt, optional(","))), "}"),
    exit_stmt: ($) =>
      choice(
        $.required_items_stmt,
        $.required_flags_stmt,
        $.barred_stmt,
        "hidden",
        "locked",
      ),
    required_items_stmt: ($) =>
      seq(
        "required_items",
        "(",
        sep1(field("item_id", alias($.identifier, $.item_id)), ","),
        ")",
      ),
    required_flags_stmt: ($) =>
      seq(
        "required_flags",
        "(",
        sep1(field("flag_id", alias($.identifier, $.flag_name)), ","),
        ")",
      ),
    barred_stmt: ($) =>
      seq("barred", field("msg", alias($.string, $.player_message))),

    // --------------------- room overlays ---------------------
    overlay_stmt: ($) => seq("overlay", "if", $._ovl_cond_list, $.ovl_block),

    ovl_block: ($) => seq("{", $.ovl_text_stmt, "}"),

    ovl_text_stmt: ($) =>
      seq("text", field("text", alias($.string, $.ovl_text))),

    _ovl_cond_list: ($) =>
      choice(
        $._ovl_cond,
        sep1($._ovl_cond, ","),
        seq("(", sep1($._ovl_cond, ","), ")"),
      ),

    _ovl_cond: ($) =>
      choice(
        $.ovl_flag_status,
        $.ovl_item_presence,
        $.ovl_item_posession,
        $.ovl_npc_presence,
        $.ovl_npc_state,
        $.ovl_item_in_room,
      ),

    ovl_flag_status: ($) =>
      seq(
        "flag",
        choice("set", "unset", "complete"),
        field("flag_name", alias($.identifier, $.flag_name)),
      ),

    ovl_item_presence: ($) =>
      seq(
        "item",
        choice("present", "absent"),
        field("item_id", alias($.identifier, $.item_id)),
      ),

    ovl_item_posession: ($) =>
      seq(
        "player",
        choice("has", "missing"),
        "item",
        field("item_id", alias($.identifier, $.item_id)),
      ),

    ovl_npc_presence: ($) =>
      seq(
        "npc",
        choice("present", "absent"),
        field("npc_id", alias($.identifier, $.npc_id)),
      ),

    ovl_npc_state: ($) =>
      seq(
        "npc",
        "in",
        "state",
        field("npc_id", alias($.identifier, $.npc_id)),
        field("npc_state", $.npc_state),
      ),

    npc_state: ($) => choice($.npc_state_builtin, $.npc_state_custom),

    npc_state_builtin: ($) =>
      choice("normal", "happy", "bored", "tired", "sad", "mad"),

    npc_state_custom: ($) =>
      seq("custom", field("custom_state", alias($.identifier, $.custom_state))),

    ovl_item_in_room: ($) =>
      seq(
        "item",
        "in",
        "room",
        field("item_id", alias($.identifier, $.item_id)),
        field("room_id", alias($.identifier, $.room_id)),
      ),

    // ---------------- overlay flag sets --------------------
    ovl_flag_binary: ($) =>
      seq(
        "overlay",
        "if",
        "flag",
        field("flag_name", alias($.identifier, $.flag_name)),
        $.flag_binary_block,
      ),
    flag_binary_block: ($) =>
      seq(
        "{",
        "set",
        field("set_text", alias($.string, $.ovl_text)),
        "unset",
        field("unset_text", alias($.string, $.ovl_text)),
        "}",
      ),

    ovl_presence_pair: ($) =>
      seq(
        "overlay",
        "if",
        choice(
          seq("item", field("item_id", alias($.identifier, $.item_id))),
          seq("npc", field("npc_id", alias($.identifier, $.npc_id))),
        ),
        $.presence_pair_block,
      ),

    presence_pair_block: ($) =>
      seq(
        "{",
        "present",
        field("present_text", alias($.string, $.ovl_text)),
        "absent",
        field("absent_text", alias($.string, $.ovl_text)),
        "}",
      ),

    ovl_npc_state_set: ($) =>
      seq(
        "overlay",
        "if",
        "npc",
        field("npc_id", alias($.identifier, $.npc_id)),
        "here",
        $.npc_state_set_block,
      ),
    npc_state_set_block: ($) => seq("{", repeat1($.npc_state_set_line), "}"),
    npc_state_set_line: ($) =>
      seq(
        choice($.npc_state_builtin, $.npc_state_set_custom),
        field("text", alias($.string, $.ovl_text)),
      ),
    npc_state_set_custom: ($) =>
      seq(
        "custom",
        "(",
        field("state", alias($.identifier, $.custom_state)),
        ")",
      ),

    //
    //
    //
    //
    //
    // ITEM DEFINITIONS
    //
    //
    //
    //
    //
    item_def: ($) =>
      seq(
        "item",
        field("item_id", alias($.identifier, $.item_id)),
        $.item_block,
      ),
    item_block: ($) => seq("{", repeat($._item_stmt), "}"),
    _item_stmt: ($) =>
      choice(
        $.item_name_stmt,
        $.item_desc_stmt,
        $.item_loc_stmt,
        $.item_portable_stmt,
        $.item_ability_stmt,
        $.item_text_stmt,
        $.item_restricted_stmt,
        $.item_container_stmt,
        $.item_requires_stmt,
      ),
    item_name_stmt: ($) =>
      seq("name", field("item_name", alias($.string, $.entity_name))),
    item_desc_stmt: ($) =>
      seq(
        choice("desc", "description"),
        field("item_description", alias($.string, $.entity_desc)),
      ),
    item_loc_stmt: ($) => seq("location", $.item_location),
    item_location: ($) =>
      choice(
        seq("inventory", "player"),
        seq("room", field("room_id", alias($.identifier, $.room_id))),
        seq("chest", field("chest_id", alias($.identifier, $.item_id))),
        seq("npc", field("npc_id", alias($.identifier, $.npc_id))),
        seq("nowhere", field("spawn_note", alias($.string, $.dev_note))),
      ),
    item_portable_stmt: ($) => seq("portable", field("portable", $.boolean)),
    item_restricted_stmt: ($) =>
      seq("restricted", field("restricted", $.boolean)),
    item_ability_stmt: ($) =>
      seq(
        "ability",
        field("ability", alias($.identifier, $.item_ability)),
        // ability_target = id of something targeted by this ability e.g. unlock <which_item>
        optional(field("target_id", alias($.identifier, $.ability_target))),
      ),
    item_text_stmt: ($) =>
      seq("text", field("item_text", alias($.string, $.entity_desc))),
    item_container_stmt: ($) => seq("container", "state", $.container_state),
    container_state: ($) =>
      choice(
        "open",
        "closed",
        "locked",
        "transparentClosed",
        "transparentLocked",
        "none",
      ),
    item_requires_stmt: ($) =>
      seq(
        "requires",
        field("ability", alias($.identifier, $.item_ability)),
        "to",
        field("interaction", alias($.identifier, $.item_interaction)),
      ),

    //
    //
    //
    //
    // NPC DEFINITIONS
    //
    //
    //
    //
    npc_def: ($) =>
      seq("npc", field("npc_id", alias($.identifier, $.npc_id)), $.npc_block),
    npc_block: ($) => seq("{", repeat($._npc_stmt), "}"),
    _npc_stmt: ($) =>
      choice(
        $.npc_name_stmt,
        $.npc_desc_stmt,
        $.npc_loc_stmt,
        $.npc_state_stmt,
        $.npc_movement_stmt,
        $.npc_dialogue_block,
      ),
    npc_name_stmt: ($) =>
      seq("name", field("npc_name", alias($.string, $.entity_name))),
    npc_desc_stmt: ($) =>
      seq(
        choice("desc", "description"),
        field("npc_description", alias($.string, $.entity_desc)),
      ),
    npc_loc_stmt: ($) => seq("location", $.npc_location),
    npc_location: ($) =>
      choice(
        seq("nowhere", field("spawn_note", alias($.string, $.dev_note))),
        seq("room", field("room_id", alias($.identifier, $.room_id))),
      ),
    npc_state_stmt: ($) => seq("state", $.npc_state),
    npc_movement_stmt: ($) =>
      seq(
        "movement",
        $.movement_type,
        "rooms",
        $.room_list,
        optional($.timing_stmt),
        optional($.active_stmt),
      ),
    movement_type: ($) => choice("route", "random"),
    room_list: ($) => seq("(", sep1(alias($.identifier, $.room_id), ","), ")"),
    timing_stmt: ($) =>
      seq("timing", field("timing", alias($.identifier, $.timing))),
    active_stmt: ($) => seq("active", field("active", $.boolean)),
    npc_dialogue_block: ($) =>
      seq(
        "dialogue",
        $.npc_state,
        "{",
        repeat1(field("dialogue", alias($.string, $.npc_dialogue))),
        "}",
      ),

    //
    //
    //
    //
    //
    // TRIGGER DEFINITIONS
    //
    //
    //
    //
    //
    trigger_def: ($) =>
      seq(
        "trigger",
        field("name", alias($.string, $.entity_name)),
        repeat(choice(field("once", $.only_once_kw), $.trigger_note)),
        "when",
        $.when_cond,
        $.trigger_block,
      ),
    only_once_kw: ($) => seq("only", "once"),
    trigger_note: ($) =>
      seq("note", field("trigger_note", alias($.string, $.dev_note))),

    // "when" conditions / triggering events
    when_cond: ($) =>
      choice(
        $.always_event,
        $.enter_room,
        $.take_item,
        $.talk_to_npc,
        $.open_item,
        $.leave_room,
        $.look_at_item,
        $.use_item,
        $.give_to_npc,
        $.use_item_on_item,
        $.act_on_item,
        $.take_from_npc,
        $.insert_item_into,
        $.drop_item,
        $.unlock_item,
      ),
    always_event: ($) => "always",
    enter_room: ($) =>
      seq("enter", "room", field("room_id", alias($.identifier, $.room_id))),
    leave_room: ($) =>
      seq("leave", "room", field("room_id", alias($.identifier, $.room_id))),
    take_item: ($) =>
      seq("take", "item", field("item_id", alias($.identifier, $.item_id))),
    drop_item: ($) =>
      seq("drop", "item", field("item_id", alias($.identifier, $.item_id))),
    open_item: ($) =>
      seq("open", "item", field("item_id", alias($.identifier, $.item_id))),
    unlock_item: ($) =>
      seq("unlock", "item", field("item_id", alias($.identifier, $.item_id))),
    talk_to_npc: ($) =>
      seq("talk", "to", "npc", field("npc_id", alias($.identifier, $.npc_id))),
    look_at_item: ($) =>
      seq(
        "look",
        "at",
        "item",
        field("item_id", alias($.identifier, $.item_id)),
      ),
    use_item: ($) =>
      seq(
        "use",
        "item",
        field("item_id", alias($.identifier, $.item_id)),
        "ability",
        field("ability", alias($.identifier, $.item_ability)),
      ),
    give_to_npc: ($) =>
      seq(
        "give",
        "item",
        field("item_id", alias($.identifier, $.item_id)),
        "to",
        "npc",
        field("npc_id", alias($.identifier, $.npc_id)),
      ),
    use_item_on_item: ($) =>
      seq(
        "use",
        "item",
        field("tool_id", alias($.identifier, $.item_id)),
        "on",
        "item",
        field("target_id", alias($.identifier, $.item_id)),
        "interaction",
        field("interaction", alias($.identifier, $.item_interaction)),
      ),
    act_on_item: ($) =>
      seq(
        "act",
        field("action", alias($.identifier, $.item_interaction)),
        "on",
        "item",
        field("item_id", alias($.identifier, $.item_id)),
      ),
    take_from_npc: ($) =>
      seq(
        "take",
        "item",
        field("item_id", alias($.identifier, $.item_id)),
        "from",
        "npc",
        field("npc_id", alias($.identifier, $.npc_id)),
      ),
    insert_item_into: ($) =>
      seq(
        "insert",
        "item",
        field("item_id", alias($.identifier, $.item_id)),
        "into",
        "item",
        field("item_id", alias($.identifier, $.item_id)),
      ),

    // start trigger action block, with if and do statements
    trigger_block: ($) => seq("{", repeat1($._trigger_stmt), "}"),
    _trigger_stmt: ($) => choice($.do_action, $.cond_block),

    cond_block: ($) =>
      seq("if", $.trigger_cond, "{", repeat1($.do_action), "}"),
    trigger_cond: ($) =>
      choice(
        $.cond_any_group,
        $.cond_all_group,
        $.cond_has_flag,
        $.cond_missing_flag,
        $.cond_has_item,
        $.cond_missing_item,
        $.cond_visited_room,
        $.cond_flag_in_progress,
        $.cond_flag_complete,
        $.cond_with_npc,
        $.cond_npc_has_item,
        $.cond_npc_in_state,
        $.cond_player_in_room,
        $.cond_container_has_item,
        $.cond_chance,
        $.cond_ambient,
        $.cond_in_rooms,
      ),
    cond_any_group: ($) => seq("any", "(", sep1($.trigger_cond, ","), ")"),
    cond_all_group: ($) => seq("all", "(", sep1($.trigger_cond, ","), ")"),
    cond_has_flag: ($) =>
      seq("has", "flag", field("flag_name", alias($.identifier, $.flag_name))),
    cond_missing_flag: ($) =>
      seq(
        "missing",
        "flag",
        field("flag_name", alias($.identifier, $.flag_name)),
      ),
    cond_has_item: ($) =>
      seq("has", "item", field("item_id", alias($.identifier, $.item_id))),
    cond_missing_item: ($) =>
      seq("missing", "item", field("item_id", alias($.identifier, $.item_id))),
    cond_visited_room: ($) =>
      seq(
        "has",
        "visited",
        "room",
        field("room_id", alias($.identifier, $.room_id)),
      ),
    cond_flag_in_progress: ($) =>
      seq(
        "flag",
        "in",
        "progress",
        field("flag_name", alias($.identifier, $.flag_name)),
      ),
    cond_flag_complete: ($) =>
      seq(
        "flag",
        "complete",
        field("flag_name", alias($.identifier, $.flag_name)),
      ),
    cond_with_npc: ($) =>
      seq("with", "npc", field("npc_id", alias($.identifier, $.npc_id))),
    cond_npc_has_item: ($) =>
      seq(
        "npc",
        "has",
        "item",
        field("npc_id", alias($.identifier, $.npc_id)),
        field("item_id", alias($.identifier, $.item_id)),
      ),
    cond_npc_in_state: ($) =>
      seq(
        "npc",
        "in",
        "state",
        field("npc_id", alias($.identifier, $.npc_id)),
        field("state", alias($.identifier, $.custom_state)),
      ),
    cond_player_in_room: ($) =>
      seq(
        "player",
        "in",
        "room",
        field("room_id", alias($.identifier, $.room_id)),
      ),
    cond_container_has_item: ($) =>
      seq(
        "container",
        field("container_id", alias($.identifier, $.item_id)),
        "has",
        "item",
        field("item_id", alias($.identifier, $.item_id)),
      ),
    cond_chance: ($) => seq("chance", field("pct", $.number), "%"),
    cond_ambient: ($) =>
      seq(
        "ambient",
        field("spinner", alias($.identifier, $.spinner)),
        // If present, prefer consuming commas as part of the inner room list
        // when nested inside grouped conditions like any(...) or all(...).
        optional(
          prec.left(
            seq(
              "in",
              "rooms",
              sep1(field("room_id", alias($.identifier, $.room_id)), ","),
            ),
          ),
        ),
      ),
    // Prefer consuming commas as part of the inner room list when nested
    // inside grouped conditions like any(...) or all(...). Use a restricted
    // identifier that excludes reserved keywords, so outer conditions like
    // 'has flag …' are not swallowed into the room list.
    cond_in_rooms: ($) =>
      prec.left(
        seq(
          "in",
          "rooms",
          sep1(field("room_id", alias($.identifier, $.room_id)), ","),
        ),
      ),

    do_action: ($) => seq("do", $._action_type),
    _action_type: ($) =>
      choice(
        $.action_show,
        $.action_add_wedge,
        $.action_add_seq,
        $.action_replace_item,
        $.action_replace_drop_item,
        $.action_add_flag,
        $.action_reset_flag,
        $.action_remove_flag,
        $.action_advance_flag,
        $.action_spawn_room,
        $.action_spawn_container,
        $.action_spawn_inventory,
        $.action_spawn_current_room,
        $.action_despawn_item,
        $.action_award_points,
        $.action_lock_item,
        $.action_unlock_item,
        $.action_lock_exit,
        $.action_unlock_exit,
        $.action_reveal_exit,
        $.action_push_player,
        $.action_set_item_desc,
        $.action_npc_random_dialogue,
        $.action_npc_says,
        $.action_npc_refuse_item,
        $.action_set_npc_state,
        $.action_deny_read,
        $.action_restrict_item,
        $.action_give_to_player,
        $.action_set_barred_msg,
        $.action_set_container_state,
        $.action_spinner_msg,
        $.action_schedule_in_or_on,
        $.action_schedule_in_if,
      ),
    action_show: ($) =>
      seq("show", field("text", alias($.string, $.player_message))),
    action_add_wedge: ($) =>
      seq(
        "add",
        "wedge",
        field("text", alias($.string, $.wedge_text)),
        optional(seq("width", $.number)),
        "spinner",
        field("spinner", alias($.identifier, $.spinner)),
      ),
    action_add_seq: ($) =>
      seq(
        "add",
        "seq",
        "flag",
        field("flag_name", alias($.identifier, $.flag_name)),
        optional(seq("limit", $.number)),
      ),
    action_replace_item: ($) =>
      seq(
        "replace",
        "item",
        field("item_id", alias($.identifier, $.item_id)),
        "with",
        field("item_id", alias($.identifier, $.item_id)),
      ),
    action_replace_drop_item: ($) =>
      seq(
        "replace",
        "drop",
        "item",
        field("item_id", alias($.identifier, $.item_id)),
        "with",
        field("item_id", alias($.identifier, $.item_id)),
      ),
    action_add_flag: ($) =>
      seq("add", "flag", field("flag", alias($.identifier, $.flag_name))),
    action_reset_flag: ($) =>
      seq("reset", "flag", field("flag", alias($.identifier, $.flag_name))),
    action_remove_flag: ($) =>
      seq("remove", "flag", field("flag", alias($.identifier, $.flag_name))),
    action_advance_flag: ($) =>
      seq("advance", "flag", field("flag", alias($.identifier, $.flag_name))),
    spawn_action_stem: ($) =>
      seq("spawn", "item", field("item_id", alias($.identifier, $.item_id))),
    action_spawn_room: ($) =>
      seq(
        $.spawn_action_stem,
        "into",
        "room",
        field("room", alias($.identifier, $.room_id)),
      ),
    action_spawn_container: ($) =>
      seq(
        $.spawn_action_stem,
        choice("into", "in"),
        "container",
        field("container_id", alias($.identifier, $.item_id)),
      ),
    action_spawn_inventory: ($) => seq($.spawn_action_stem, "in", "inventory"),
    action_spawn_current_room: ($) =>
      seq($.spawn_action_stem, "in", "current", "room"),
    action_despawn_item: ($) =>
      seq("despawn", "item", field("item_id", alias($.identifier, $.item_id))),
    action_award_points: ($) =>
      seq("award", "points", field("points", $.number)),
    action_lock_item: ($) =>
      seq("lock", "item", field("item_id", alias($.identifier, $.item_id))),
    action_unlock_item: ($) =>
      seq("unlock", "item", field("item_id", alias($.identifier, $.item_id))),
    action_lock_exit: ($) =>
      seq(
        "lock",
        "exit",
        "from",
        field("room_id", alias($.identifier, $.room_id)),
        "direction",
        field("direction", alias(choice($.identifier, $.string), $.exit_dir)),
      ),
    action_unlock_exit: ($) =>
      seq(
        "unlock",
        "exit",
        "from",
        field("room_id", alias($.identifier, $.room_id)),
        "direction",
        field("direction", alias(choice($.identifier, $.string), $.exit_dir)),
      ),
    action_reveal_exit: ($) =>
      seq(
        "reveal",
        "exit",
        "from",
        field("from_room", alias($.identifier, $.room_id)),
        "to",
        field("to_room", alias($.identifier, $.room_id)),
        "direction",
        field("direction", alias(choice($.identifier, $.string), $.exit_dir)),
      ),
    action_push_player: ($) =>
      seq(
        "push",
        "player",
        "to",
        field("room_id", alias($.identifier, $.room_id)),
      ),
    action_set_item_desc: ($) =>
      seq(
        "set",
        "item",
        "description",
        field("item_id", alias($.identifier, $.item_id)),
        field("text", alias($.string, $.entity_desc)),
      ),
    action_npc_random_dialogue: ($) =>
      seq(
        "npc",
        "random",
        "dialogue",
        field("npc_id", alias($.identifier, $.npc_id)),
      ),
    action_npc_says: ($) =>
      seq(
        "npc",
        "says",
        field("npc_id", alias($.identifier, $.npc_id)),
        field("text", alias($.string, $.quote)),
      ),
    action_npc_refuse_item: ($) =>
      seq(
        "npc",
        "refuse",
        "item",
        field("npc_id", alias($.identifier, $.npc_id)),
        field("reason", alias($.string, $.player_message)),
      ),
    action_set_npc_state: ($) =>
      seq(
        "set",
        "npc",
        "state",
        field("npc_id", alias($.identifier, $.npc_id)),
        field("state", alias($.identifier, $.custom_state)),
      ),
    action_deny_read: ($) =>
      seq("deny", "read", field("reason", alias($.string, $.player_message))),
    action_restrict_item: ($) =>
      seq("restrict", "item", field("item_id", alias($.identifier, $.item_id))),
    action_give_to_player: ($) =>
      seq(
        "give",
        "item",
        field("item_id", alias($.identifier, $.item_id)),
        "to",
        "player",
        "from",
        "npc",
        field("npc_id", alias($.identifier, $.npc_id)),
      ),
    action_set_barred_msg: ($) =>
      seq(
        "set",
        "barred",
        "message",
        "from",
        field("room_id", alias($.identifier, $.room_id)),
        "to",
        field("room_id", alias($.identifier, $.room_id)),
        field("msg", alias($.string, $.player_message)),
      ),
    action_set_container_state: ($) =>
      seq(
        "set",
        "container",
        "state",
        field("item_id", alias($.identifier, $.item_id)),
        $.container_state,
      ),
    action_spinner_msg: ($) =>
      seq(
        "spinner",
        "message",
        field("spinner", alias($.identifier, $.spinner)),
      ),

    // scheduler actions
    schedule_note: ($) =>
      seq("note", field("text", alias($.string, $.schedule_note_text))),
    retry_type: ($) =>
      seq(
        "onFalse",
        field(
          "policy",
          alias(
            choice("cancel", "retryNextTurn", seq("retryAfter", $.number)),
            $.retry_policy,
          ),
        ),
      ),
    action_schedule_in_or_on: ($) =>
      seq(
        "schedule",
        choice("in", "on"),
        field("turns", $.number),
        optional($.schedule_note),
        $.trigger_block,
      ),
    action_schedule_in_if: ($) =>
      seq(
        "schedule",
        choice("in", "on"),
        field("turns", $.number),
        "if",
        $.trigger_cond,
        optional($.retry_type),
        optional($.schedule_note),
        $.trigger_block,
      ),

    //
    //
    //
    //
    //
    // SPINNER DEFINITIONS
    //
    //
    //
    //
    //
    spinner_def: ($) =>
      seq(
        "spinner",
        field("name", alias($.identifier, $.spinner)),
        $.spinner_block,
      ),
    spinner_block: ($) => seq("{", repeat1($.spinner_stmt), "}"),
    spinner_stmt: ($) =>
      seq(
        "wedge",
        field("spinner_text", alias($.string, $.spinner_text)),
        optional(seq("width", field("width", $.number))),
      ),

    //
    //
    //
    //
    //
    // GOAL DEFINITIONS
    //
    //
    //
    //
    //
    goal_def: ($) =>
      seq(
        "goal",
        field("goal_id", alias($.identifier, $.goal_id)),
        $.goal_block,
      ),
    goal_block: ($) => seq("{", repeat1($._goal_stmt), "}"),
    _goal_stmt: ($) =>
      choice(
        $.goal_name_stmt,
        $.goal_desc_stmt,
        $.goal_group_stmt,
        $.goal_start_stmt,
        $.goal_done_stmt,
      ),
    goal_name_stmt: ($) =>
      seq("name", field("goal_name", alias($.string, $.entity_name))),
    goal_desc_stmt: ($) =>
      seq("desc", field("goal_description", alias($.string, $.entity_desc))),
    goal_group_stmt: ($) =>
      seq(
        "group",
        field(
          "goal_group",
          alias(choice("required", "optional", "status-effect"), $.goal_group),
        ),
      ),
    goal_start_stmt: ($) =>
      seq("start", "when", field("start_condition", $._goal_cond)),
    goal_done_stmt: ($) =>
      seq("done", "when", field("done_condition", $._goal_cond)),
    _goal_cond: ($) =>
      choice(
        $.gc_has_flag,
        $.gc_missing_flag,
        $.gc_has_item,
        $.gc_reached_room,
        $.gc_goal_complete,
        $.gc_flag_progress,
        $.gc_flag_complete,
      ),
    gc_has_flag: ($) =>
      seq("has", "flag", field("flag_name", alias($.identifier, $.flag_name))),
    gc_missing_flag: ($) =>
      seq(
        "missing",
        "flag",
        field("flag_name", alias($.identifier, $.flag_name)),
      ),
    gc_has_item: ($) =>
      seq("has", "item", field("item_id", alias($.identifier, $.item_id))),
    gc_reached_room: ($) =>
      seq("reached", "room", field("room_id", alias($.identifier, $.room_id))),
    gc_goal_complete: ($) =>
      seq("goal", "complete", field("goal_id", alias($.identifier, $.goal_id))),
    gc_flag_progress: ($) =>
      seq(
        "flag",
        "in",
        "progress",
        field("flag_name", alias($.identifier, $.flag_name)),
      ),
    gc_flag_complete: ($) =>
      seq(
        "flag",
        "complete",
        field("flag_name", alias($.identifier, $.flag_name)),
      ),
  },
});

function sep1(rule, delimiter) {
  return seq(rule, repeat(seq(delimiter, rule)));
}
