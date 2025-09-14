#include <tree_sitter/parser.h>

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#ifdef _MSC_VER
#pragma optimize("", off)
#elif defined(__clang__)
#pragma clang optimize off
#elif defined(__GNUC__)
#pragma GCC optimize ("O0")
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 223
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 154
#define ALIAS_COUNT 0
#define TOKEN_COUNT 81
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 0
#define MAX_ALIAS_SEQUENCE_LENGTH 11
#define PRODUCTION_ID_COUNT 1

enum {
  sym_comment = 1,
  sym_identifier = 2,
  sym_number = 3,
  aux_sym_string_token1 = 4,
  aux_sym_string_token2 = 5,
  aux_sym_string_token3 = 6,
  aux_sym_string_token4 = 7,
  aux_sym_string_token5 = 8,
  anon_sym_true = 9,
  anon_sym_false = 10,
  anon_sym_let = 11,
  anon_sym_set = 12,
  anon_sym_EQ = 13,
  anon_sym_LPAREN = 14,
  anon_sym_COMMA = 15,
  anon_sym_RPAREN = 16,
  anon_sym_trigger = 17,
  anon_sym_only = 18,
  anon_sym_once = 19,
  anon_sym_when = 20,
  anon_sym_LBRACE = 21,
  anon_sym_RBRACE = 22,
  anon_sym_if = 23,
  anon_sym_do = 24,
  anon_sym_room = 25,
  anon_sym_name = 26,
  anon_sym_desc = 27,
  anon_sym_description = 28,
  anon_sym_visited = 29,
  anon_sym_overlay = 30,
  anon_sym_unset = 31,
  anon_sym_text = 32,
  anon_sym_normal = 33,
  anon_sym_happy = 34,
  anon_sym_bored = 35,
  anon_sym_exit = 36,
  anon_sym_DASH_GT = 37,
  anon_sym_required_flags = 38,
  anon_sym_required_items = 39,
  anon_sym_barred = 40,
  anon_sym_item = 41,
  anon_sym_portable = 42,
  anon_sym_ability = 43,
  anon_sym_container = 44,
  anon_sym_state = 45,
  anon_sym_open = 46,
  anon_sym_closed = 47,
  anon_sym_restricted = 48,
  anon_sym_spinner = 49,
  anon_sym_wedge = 50,
  anon_sym_width = 51,
  anon_sym_npc = 52,
  anon_sym_mad = 53,
  anon_sym_custom = 54,
  anon_sym_movement = 55,
  anon_sym_random = 56,
  anon_sym_route = 57,
  anon_sym_rooms = 58,
  anon_sym_timing = 59,
  anon_sym_active = 60,
  anon_sym_dialogue = 61,
  anon_sym_location = 62,
  anon_sym_chest = 63,
  anon_sym_inventory = 64,
  anon_sym_player = 65,
  anon_sym_nowhere = 66,
  anon_sym_goal = 67,
  anon_sym_group = 68,
  anon_sym_required = 69,
  anon_sym_optional = 70,
  anon_sym_status_DASHeffect = 71,
  anon_sym_done = 72,
  anon_sym_start = 73,
  anon_sym_has = 74,
  anon_sym_flag = 75,
  anon_sym_missing = 76,
  anon_sym_reached = 77,
  anon_sym_complete = 78,
  anon_sym_in = 79,
  anon_sym_progress = 80,
  sym_program = 81,
  sym_string = 82,
  sym_boolean = 83,
  sym_set_decl = 84,
  sym_set_list = 85,
  sym_trigger = 86,
  sym_trigger_mod = 87,
  sym_trigger_block = 88,
  sym_trigger_stmt = 89,
  sym_if_block = 90,
  sym_do_stmt = 91,
  sym_braced_block = 92,
  sym_cond_line = 93,
  sym_cond_line_ext = 94,
  sym_room_def = 95,
  sym_room_block = 96,
  sym_room_stmt = 97,
  sym_room_name = 98,
  sym_room_desc = 99,
  sym_room_visited = 100,
  sym_overlay_stmt = 101,
  sym_overlay_block = 102,
  sym_overlay_entry = 103,
  sym_exit_stmt = 104,
  sym_exit_block = 105,
  sym_exit_attr = 106,
  sym_exit_required_flags = 107,
  sym_exit_required_items = 108,
  sym_exit_barred = 109,
  sym_item_def = 110,
  sym_item_block = 111,
  sym_item_stmt = 112,
  sym_item_name = 113,
  sym_item_desc = 114,
  sym_item_portable = 115,
  sym_item_text = 116,
  sym_item_location = 117,
  sym_item_ability = 118,
  sym_item_container_state = 119,
  sym_item_restricted = 120,
  sym_spinner_def = 121,
  sym_spinner_block = 122,
  sym_wedge_stmt = 123,
  sym_npc_def = 124,
  sym_npc_block = 125,
  sym_npc_stmt = 126,
  sym_npc_name = 127,
  sym_npc_desc = 128,
  sym_npc_state = 129,
  sym_movement_stmt = 130,
  sym_dialogue_stmt = 131,
  sym_location = 132,
  sym_goal_def = 133,
  sym_goal_stmt = 134,
  sym_goal_desc = 135,
  sym_goal_group = 136,
  sym_goal_done = 137,
  sym_goal_start = 138,
  sym_goal_cond = 139,
  aux_sym_program_repeat1 = 140,
  aux_sym_set_list_repeat1 = 141,
  aux_sym_trigger_repeat1 = 142,
  aux_sym_trigger_block_repeat1 = 143,
  aux_sym_do_stmt_repeat1 = 144,
  aux_sym_cond_line_ext_repeat1 = 145,
  aux_sym_room_block_repeat1 = 146,
  aux_sym_overlay_block_repeat1 = 147,
  aux_sym_exit_block_repeat1 = 148,
  aux_sym_item_block_repeat1 = 149,
  aux_sym_spinner_block_repeat1 = 150,
  aux_sym_npc_block_repeat1 = 151,
  aux_sym_dialogue_stmt_repeat1 = 152,
  aux_sym_goal_def_repeat1 = 153,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [sym_comment] = "comment",
  [sym_identifier] = "identifier",
  [sym_number] = "number",
  [aux_sym_string_token1] = "string_token1",
  [aux_sym_string_token2] = "string_token2",
  [aux_sym_string_token3] = "string_token3",
  [aux_sym_string_token4] = "string_token4",
  [aux_sym_string_token5] = "string_token5",
  [anon_sym_true] = "true",
  [anon_sym_false] = "false",
  [anon_sym_let] = "let",
  [anon_sym_set] = "set",
  [anon_sym_EQ] = "=",
  [anon_sym_LPAREN] = "(",
  [anon_sym_COMMA] = ",",
  [anon_sym_RPAREN] = ")",
  [anon_sym_trigger] = "trigger",
  [anon_sym_only] = "only",
  [anon_sym_once] = "once",
  [anon_sym_when] = "when",
  [anon_sym_LBRACE] = "{",
  [anon_sym_RBRACE] = "}",
  [anon_sym_if] = "if",
  [anon_sym_do] = "do",
  [anon_sym_room] = "room",
  [anon_sym_name] = "name",
  [anon_sym_desc] = "desc",
  [anon_sym_description] = "description",
  [anon_sym_visited] = "visited",
  [anon_sym_overlay] = "overlay",
  [anon_sym_unset] = "unset",
  [anon_sym_text] = "text",
  [anon_sym_normal] = "normal",
  [anon_sym_happy] = "happy",
  [anon_sym_bored] = "bored",
  [anon_sym_exit] = "exit",
  [anon_sym_DASH_GT] = "->",
  [anon_sym_required_flags] = "required_flags",
  [anon_sym_required_items] = "required_items",
  [anon_sym_barred] = "barred",
  [anon_sym_item] = "item",
  [anon_sym_portable] = "portable",
  [anon_sym_ability] = "ability",
  [anon_sym_container] = "container",
  [anon_sym_state] = "state",
  [anon_sym_open] = "open",
  [anon_sym_closed] = "closed",
  [anon_sym_restricted] = "restricted",
  [anon_sym_spinner] = "spinner",
  [anon_sym_wedge] = "wedge",
  [anon_sym_width] = "width",
  [anon_sym_npc] = "npc",
  [anon_sym_mad] = "mad",
  [anon_sym_custom] = "custom",
  [anon_sym_movement] = "movement",
  [anon_sym_random] = "random",
  [anon_sym_route] = "route",
  [anon_sym_rooms] = "rooms",
  [anon_sym_timing] = "timing",
  [anon_sym_active] = "active",
  [anon_sym_dialogue] = "dialogue",
  [anon_sym_location] = "location",
  [anon_sym_chest] = "chest",
  [anon_sym_inventory] = "inventory",
  [anon_sym_player] = "player",
  [anon_sym_nowhere] = "nowhere",
  [anon_sym_goal] = "goal",
  [anon_sym_group] = "group",
  [anon_sym_required] = "required",
  [anon_sym_optional] = "optional",
  [anon_sym_status_DASHeffect] = "status-effect",
  [anon_sym_done] = "done",
  [anon_sym_start] = "start",
  [anon_sym_has] = "has",
  [anon_sym_flag] = "flag",
  [anon_sym_missing] = "missing",
  [anon_sym_reached] = "reached",
  [anon_sym_complete] = "complete",
  [anon_sym_in] = "in",
  [anon_sym_progress] = "progress",
  [sym_program] = "program",
  [sym_string] = "string",
  [sym_boolean] = "boolean",
  [sym_set_decl] = "set_decl",
  [sym_set_list] = "set_list",
  [sym_trigger] = "trigger",
  [sym_trigger_mod] = "trigger_mod",
  [sym_trigger_block] = "trigger_block",
  [sym_trigger_stmt] = "trigger_stmt",
  [sym_if_block] = "if_block",
  [sym_do_stmt] = "do_stmt",
  [sym_braced_block] = "braced_block",
  [sym_cond_line] = "cond_line",
  [sym_cond_line_ext] = "cond_line_ext",
  [sym_room_def] = "room_def",
  [sym_room_block] = "room_block",
  [sym_room_stmt] = "room_stmt",
  [sym_room_name] = "room_name",
  [sym_room_desc] = "room_desc",
  [sym_room_visited] = "room_visited",
  [sym_overlay_stmt] = "overlay_stmt",
  [sym_overlay_block] = "overlay_block",
  [sym_overlay_entry] = "overlay_entry",
  [sym_exit_stmt] = "exit_stmt",
  [sym_exit_block] = "exit_block",
  [sym_exit_attr] = "exit_attr",
  [sym_exit_required_flags] = "exit_required_flags",
  [sym_exit_required_items] = "exit_required_items",
  [sym_exit_barred] = "exit_barred",
  [sym_item_def] = "item_def",
  [sym_item_block] = "item_block",
  [sym_item_stmt] = "item_stmt",
  [sym_item_name] = "item_name",
  [sym_item_desc] = "item_desc",
  [sym_item_portable] = "item_portable",
  [sym_item_text] = "item_text",
  [sym_item_location] = "item_location",
  [sym_item_ability] = "item_ability",
  [sym_item_container_state] = "item_container_state",
  [sym_item_restricted] = "item_restricted",
  [sym_spinner_def] = "spinner_def",
  [sym_spinner_block] = "spinner_block",
  [sym_wedge_stmt] = "wedge_stmt",
  [sym_npc_def] = "npc_def",
  [sym_npc_block] = "npc_block",
  [sym_npc_stmt] = "npc_stmt",
  [sym_npc_name] = "npc_name",
  [sym_npc_desc] = "npc_desc",
  [sym_npc_state] = "npc_state",
  [sym_movement_stmt] = "movement_stmt",
  [sym_dialogue_stmt] = "dialogue_stmt",
  [sym_location] = "location",
  [sym_goal_def] = "goal_def",
  [sym_goal_stmt] = "goal_stmt",
  [sym_goal_desc] = "goal_desc",
  [sym_goal_group] = "goal_group",
  [sym_goal_done] = "goal_done",
  [sym_goal_start] = "goal_start",
  [sym_goal_cond] = "goal_cond",
  [aux_sym_program_repeat1] = "program_repeat1",
  [aux_sym_set_list_repeat1] = "set_list_repeat1",
  [aux_sym_trigger_repeat1] = "trigger_repeat1",
  [aux_sym_trigger_block_repeat1] = "trigger_block_repeat1",
  [aux_sym_do_stmt_repeat1] = "do_stmt_repeat1",
  [aux_sym_cond_line_ext_repeat1] = "cond_line_ext_repeat1",
  [aux_sym_room_block_repeat1] = "room_block_repeat1",
  [aux_sym_overlay_block_repeat1] = "overlay_block_repeat1",
  [aux_sym_exit_block_repeat1] = "exit_block_repeat1",
  [aux_sym_item_block_repeat1] = "item_block_repeat1",
  [aux_sym_spinner_block_repeat1] = "spinner_block_repeat1",
  [aux_sym_npc_block_repeat1] = "npc_block_repeat1",
  [aux_sym_dialogue_stmt_repeat1] = "dialogue_stmt_repeat1",
  [aux_sym_goal_def_repeat1] = "goal_def_repeat1",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [sym_comment] = sym_comment,
  [sym_identifier] = sym_identifier,
  [sym_number] = sym_number,
  [aux_sym_string_token1] = aux_sym_string_token1,
  [aux_sym_string_token2] = aux_sym_string_token2,
  [aux_sym_string_token3] = aux_sym_string_token3,
  [aux_sym_string_token4] = aux_sym_string_token4,
  [aux_sym_string_token5] = aux_sym_string_token5,
  [anon_sym_true] = anon_sym_true,
  [anon_sym_false] = anon_sym_false,
  [anon_sym_let] = anon_sym_let,
  [anon_sym_set] = anon_sym_set,
  [anon_sym_EQ] = anon_sym_EQ,
  [anon_sym_LPAREN] = anon_sym_LPAREN,
  [anon_sym_COMMA] = anon_sym_COMMA,
  [anon_sym_RPAREN] = anon_sym_RPAREN,
  [anon_sym_trigger] = anon_sym_trigger,
  [anon_sym_only] = anon_sym_only,
  [anon_sym_once] = anon_sym_once,
  [anon_sym_when] = anon_sym_when,
  [anon_sym_LBRACE] = anon_sym_LBRACE,
  [anon_sym_RBRACE] = anon_sym_RBRACE,
  [anon_sym_if] = anon_sym_if,
  [anon_sym_do] = anon_sym_do,
  [anon_sym_room] = anon_sym_room,
  [anon_sym_name] = anon_sym_name,
  [anon_sym_desc] = anon_sym_desc,
  [anon_sym_description] = anon_sym_description,
  [anon_sym_visited] = anon_sym_visited,
  [anon_sym_overlay] = anon_sym_overlay,
  [anon_sym_unset] = anon_sym_unset,
  [anon_sym_text] = anon_sym_text,
  [anon_sym_normal] = anon_sym_normal,
  [anon_sym_happy] = anon_sym_happy,
  [anon_sym_bored] = anon_sym_bored,
  [anon_sym_exit] = anon_sym_exit,
  [anon_sym_DASH_GT] = anon_sym_DASH_GT,
  [anon_sym_required_flags] = anon_sym_required_flags,
  [anon_sym_required_items] = anon_sym_required_items,
  [anon_sym_barred] = anon_sym_barred,
  [anon_sym_item] = anon_sym_item,
  [anon_sym_portable] = anon_sym_portable,
  [anon_sym_ability] = anon_sym_ability,
  [anon_sym_container] = anon_sym_container,
  [anon_sym_state] = anon_sym_state,
  [anon_sym_open] = anon_sym_open,
  [anon_sym_closed] = anon_sym_closed,
  [anon_sym_restricted] = anon_sym_restricted,
  [anon_sym_spinner] = anon_sym_spinner,
  [anon_sym_wedge] = anon_sym_wedge,
  [anon_sym_width] = anon_sym_width,
  [anon_sym_npc] = anon_sym_npc,
  [anon_sym_mad] = anon_sym_mad,
  [anon_sym_custom] = anon_sym_custom,
  [anon_sym_movement] = anon_sym_movement,
  [anon_sym_random] = anon_sym_random,
  [anon_sym_route] = anon_sym_route,
  [anon_sym_rooms] = anon_sym_rooms,
  [anon_sym_timing] = anon_sym_timing,
  [anon_sym_active] = anon_sym_active,
  [anon_sym_dialogue] = anon_sym_dialogue,
  [anon_sym_location] = anon_sym_location,
  [anon_sym_chest] = anon_sym_chest,
  [anon_sym_inventory] = anon_sym_inventory,
  [anon_sym_player] = anon_sym_player,
  [anon_sym_nowhere] = anon_sym_nowhere,
  [anon_sym_goal] = anon_sym_goal,
  [anon_sym_group] = anon_sym_group,
  [anon_sym_required] = anon_sym_required,
  [anon_sym_optional] = anon_sym_optional,
  [anon_sym_status_DASHeffect] = anon_sym_status_DASHeffect,
  [anon_sym_done] = anon_sym_done,
  [anon_sym_start] = anon_sym_start,
  [anon_sym_has] = anon_sym_has,
  [anon_sym_flag] = anon_sym_flag,
  [anon_sym_missing] = anon_sym_missing,
  [anon_sym_reached] = anon_sym_reached,
  [anon_sym_complete] = anon_sym_complete,
  [anon_sym_in] = anon_sym_in,
  [anon_sym_progress] = anon_sym_progress,
  [sym_program] = sym_program,
  [sym_string] = sym_string,
  [sym_boolean] = sym_boolean,
  [sym_set_decl] = sym_set_decl,
  [sym_set_list] = sym_set_list,
  [sym_trigger] = sym_trigger,
  [sym_trigger_mod] = sym_trigger_mod,
  [sym_trigger_block] = sym_trigger_block,
  [sym_trigger_stmt] = sym_trigger_stmt,
  [sym_if_block] = sym_if_block,
  [sym_do_stmt] = sym_do_stmt,
  [sym_braced_block] = sym_braced_block,
  [sym_cond_line] = sym_cond_line,
  [sym_cond_line_ext] = sym_cond_line_ext,
  [sym_room_def] = sym_room_def,
  [sym_room_block] = sym_room_block,
  [sym_room_stmt] = sym_room_stmt,
  [sym_room_name] = sym_room_name,
  [sym_room_desc] = sym_room_desc,
  [sym_room_visited] = sym_room_visited,
  [sym_overlay_stmt] = sym_overlay_stmt,
  [sym_overlay_block] = sym_overlay_block,
  [sym_overlay_entry] = sym_overlay_entry,
  [sym_exit_stmt] = sym_exit_stmt,
  [sym_exit_block] = sym_exit_block,
  [sym_exit_attr] = sym_exit_attr,
  [sym_exit_required_flags] = sym_exit_required_flags,
  [sym_exit_required_items] = sym_exit_required_items,
  [sym_exit_barred] = sym_exit_barred,
  [sym_item_def] = sym_item_def,
  [sym_item_block] = sym_item_block,
  [sym_item_stmt] = sym_item_stmt,
  [sym_item_name] = sym_item_name,
  [sym_item_desc] = sym_item_desc,
  [sym_item_portable] = sym_item_portable,
  [sym_item_text] = sym_item_text,
  [sym_item_location] = sym_item_location,
  [sym_item_ability] = sym_item_ability,
  [sym_item_container_state] = sym_item_container_state,
  [sym_item_restricted] = sym_item_restricted,
  [sym_spinner_def] = sym_spinner_def,
  [sym_spinner_block] = sym_spinner_block,
  [sym_wedge_stmt] = sym_wedge_stmt,
  [sym_npc_def] = sym_npc_def,
  [sym_npc_block] = sym_npc_block,
  [sym_npc_stmt] = sym_npc_stmt,
  [sym_npc_name] = sym_npc_name,
  [sym_npc_desc] = sym_npc_desc,
  [sym_npc_state] = sym_npc_state,
  [sym_movement_stmt] = sym_movement_stmt,
  [sym_dialogue_stmt] = sym_dialogue_stmt,
  [sym_location] = sym_location,
  [sym_goal_def] = sym_goal_def,
  [sym_goal_stmt] = sym_goal_stmt,
  [sym_goal_desc] = sym_goal_desc,
  [sym_goal_group] = sym_goal_group,
  [sym_goal_done] = sym_goal_done,
  [sym_goal_start] = sym_goal_start,
  [sym_goal_cond] = sym_goal_cond,
  [aux_sym_program_repeat1] = aux_sym_program_repeat1,
  [aux_sym_set_list_repeat1] = aux_sym_set_list_repeat1,
  [aux_sym_trigger_repeat1] = aux_sym_trigger_repeat1,
  [aux_sym_trigger_block_repeat1] = aux_sym_trigger_block_repeat1,
  [aux_sym_do_stmt_repeat1] = aux_sym_do_stmt_repeat1,
  [aux_sym_cond_line_ext_repeat1] = aux_sym_cond_line_ext_repeat1,
  [aux_sym_room_block_repeat1] = aux_sym_room_block_repeat1,
  [aux_sym_overlay_block_repeat1] = aux_sym_overlay_block_repeat1,
  [aux_sym_exit_block_repeat1] = aux_sym_exit_block_repeat1,
  [aux_sym_item_block_repeat1] = aux_sym_item_block_repeat1,
  [aux_sym_spinner_block_repeat1] = aux_sym_spinner_block_repeat1,
  [aux_sym_npc_block_repeat1] = aux_sym_npc_block_repeat1,
  [aux_sym_dialogue_stmt_repeat1] = aux_sym_dialogue_stmt_repeat1,
  [aux_sym_goal_def_repeat1] = aux_sym_goal_def_repeat1,
};

static const TSSymbolMetadata ts_symbol_metadata[] = {
  [ts_builtin_sym_end] = {
    .visible = false,
    .named = true,
  },
  [sym_comment] = {
    .visible = true,
    .named = true,
  },
  [sym_identifier] = {
    .visible = true,
    .named = true,
  },
  [sym_number] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_string_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_token2] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_token3] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_token4] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_token5] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_true] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_false] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_let] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_set] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COMMA] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_trigger] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_only] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_once] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_when] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_if] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_do] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_room] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_name] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_desc] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_description] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_visited] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_overlay] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_unset] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_text] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_normal] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_happy] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_bored] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_exit] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DASH_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_required_flags] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_required_items] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_barred] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_item] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_portable] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ability] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_container] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_state] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_open] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_closed] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_restricted] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_spinner] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_wedge] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_width] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_npc] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_mad] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_custom] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_movement] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_random] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_route] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_rooms] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_timing] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_active] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_dialogue] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_location] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_chest] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_inventory] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_player] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_nowhere] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_goal] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_group] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_required] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_optional] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_status_DASHeffect] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_done] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_start] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_has] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_flag] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_missing] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_reached] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_complete] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_in] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_progress] = {
    .visible = true,
    .named = false,
  },
  [sym_program] = {
    .visible = true,
    .named = true,
  },
  [sym_string] = {
    .visible = true,
    .named = true,
  },
  [sym_boolean] = {
    .visible = true,
    .named = true,
  },
  [sym_set_decl] = {
    .visible = true,
    .named = true,
  },
  [sym_set_list] = {
    .visible = true,
    .named = true,
  },
  [sym_trigger] = {
    .visible = true,
    .named = true,
  },
  [sym_trigger_mod] = {
    .visible = true,
    .named = true,
  },
  [sym_trigger_block] = {
    .visible = true,
    .named = true,
  },
  [sym_trigger_stmt] = {
    .visible = true,
    .named = true,
  },
  [sym_if_block] = {
    .visible = true,
    .named = true,
  },
  [sym_do_stmt] = {
    .visible = true,
    .named = true,
  },
  [sym_braced_block] = {
    .visible = true,
    .named = true,
  },
  [sym_cond_line] = {
    .visible = true,
    .named = true,
  },
  [sym_cond_line_ext] = {
    .visible = true,
    .named = true,
  },
  [sym_room_def] = {
    .visible = true,
    .named = true,
  },
  [sym_room_block] = {
    .visible = true,
    .named = true,
  },
  [sym_room_stmt] = {
    .visible = true,
    .named = true,
  },
  [sym_room_name] = {
    .visible = true,
    .named = true,
  },
  [sym_room_desc] = {
    .visible = true,
    .named = true,
  },
  [sym_room_visited] = {
    .visible = true,
    .named = true,
  },
  [sym_overlay_stmt] = {
    .visible = true,
    .named = true,
  },
  [sym_overlay_block] = {
    .visible = true,
    .named = true,
  },
  [sym_overlay_entry] = {
    .visible = true,
    .named = true,
  },
  [sym_exit_stmt] = {
    .visible = true,
    .named = true,
  },
  [sym_exit_block] = {
    .visible = true,
    .named = true,
  },
  [sym_exit_attr] = {
    .visible = true,
    .named = true,
  },
  [sym_exit_required_flags] = {
    .visible = true,
    .named = true,
  },
  [sym_exit_required_items] = {
    .visible = true,
    .named = true,
  },
  [sym_exit_barred] = {
    .visible = true,
    .named = true,
  },
  [sym_item_def] = {
    .visible = true,
    .named = true,
  },
  [sym_item_block] = {
    .visible = true,
    .named = true,
  },
  [sym_item_stmt] = {
    .visible = true,
    .named = true,
  },
  [sym_item_name] = {
    .visible = true,
    .named = true,
  },
  [sym_item_desc] = {
    .visible = true,
    .named = true,
  },
  [sym_item_portable] = {
    .visible = true,
    .named = true,
  },
  [sym_item_text] = {
    .visible = true,
    .named = true,
  },
  [sym_item_location] = {
    .visible = true,
    .named = true,
  },
  [sym_item_ability] = {
    .visible = true,
    .named = true,
  },
  [sym_item_container_state] = {
    .visible = true,
    .named = true,
  },
  [sym_item_restricted] = {
    .visible = true,
    .named = true,
  },
  [sym_spinner_def] = {
    .visible = true,
    .named = true,
  },
  [sym_spinner_block] = {
    .visible = true,
    .named = true,
  },
  [sym_wedge_stmt] = {
    .visible = true,
    .named = true,
  },
  [sym_npc_def] = {
    .visible = true,
    .named = true,
  },
  [sym_npc_block] = {
    .visible = true,
    .named = true,
  },
  [sym_npc_stmt] = {
    .visible = true,
    .named = true,
  },
  [sym_npc_name] = {
    .visible = true,
    .named = true,
  },
  [sym_npc_desc] = {
    .visible = true,
    .named = true,
  },
  [sym_npc_state] = {
    .visible = true,
    .named = true,
  },
  [sym_movement_stmt] = {
    .visible = true,
    .named = true,
  },
  [sym_dialogue_stmt] = {
    .visible = true,
    .named = true,
  },
  [sym_location] = {
    .visible = true,
    .named = true,
  },
  [sym_goal_def] = {
    .visible = true,
    .named = true,
  },
  [sym_goal_stmt] = {
    .visible = true,
    .named = true,
  },
  [sym_goal_desc] = {
    .visible = true,
    .named = true,
  },
  [sym_goal_group] = {
    .visible = true,
    .named = true,
  },
  [sym_goal_done] = {
    .visible = true,
    .named = true,
  },
  [sym_goal_start] = {
    .visible = true,
    .named = true,
  },
  [sym_goal_cond] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_program_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_set_list_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_trigger_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_trigger_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_do_stmt_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_cond_line_ext_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_room_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_overlay_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_exit_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_item_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_spinner_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_npc_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_dialogue_stmt_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_goal_def_repeat1] = {
    .visible = false,
    .named = false,
  },
};

static const TSSymbol ts_alias_sequences[PRODUCTION_ID_COUNT][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
};

static const uint16_t ts_non_terminal_alias_map[] = {
  0,
};

static const TSStateId ts_primary_state_ids[STATE_COUNT] = {
  [0] = 0,
  [1] = 1,
  [2] = 2,
  [3] = 3,
  [4] = 4,
  [5] = 5,
  [6] = 6,
  [7] = 7,
  [8] = 8,
  [9] = 9,
  [10] = 10,
  [11] = 2,
  [12] = 12,
  [13] = 13,
  [14] = 14,
  [15] = 15,
  [16] = 16,
  [17] = 17,
  [18] = 18,
  [19] = 19,
  [20] = 20,
  [21] = 21,
  [22] = 22,
  [23] = 23,
  [24] = 24,
  [25] = 25,
  [26] = 26,
  [27] = 27,
  [28] = 28,
  [29] = 29,
  [30] = 30,
  [31] = 31,
  [32] = 32,
  [33] = 33,
  [34] = 24,
  [35] = 2,
  [36] = 2,
  [37] = 37,
  [38] = 38,
  [39] = 39,
  [40] = 40,
  [41] = 41,
  [42] = 42,
  [43] = 43,
  [44] = 44,
  [45] = 45,
  [46] = 2,
  [47] = 47,
  [48] = 48,
  [49] = 49,
  [50] = 50,
  [51] = 51,
  [52] = 52,
  [53] = 53,
  [54] = 54,
  [55] = 55,
  [56] = 56,
  [57] = 57,
  [58] = 58,
  [59] = 59,
  [60] = 60,
  [61] = 61,
  [62] = 62,
  [63] = 63,
  [64] = 64,
  [65] = 65,
  [66] = 66,
  [67] = 67,
  [68] = 68,
  [69] = 69,
  [70] = 70,
  [71] = 71,
  [72] = 72,
  [73] = 73,
  [74] = 74,
  [75] = 75,
  [76] = 76,
  [77] = 77,
  [78] = 78,
  [79] = 79,
  [80] = 80,
  [81] = 81,
  [82] = 82,
  [83] = 83,
  [84] = 84,
  [85] = 85,
  [86] = 86,
  [87] = 87,
  [88] = 88,
  [89] = 89,
  [90] = 90,
  [91] = 91,
  [92] = 92,
  [93] = 93,
  [94] = 94,
  [95] = 95,
  [96] = 96,
  [97] = 97,
  [98] = 98,
  [99] = 99,
  [100] = 100,
  [101] = 101,
  [102] = 102,
  [103] = 103,
  [104] = 104,
  [105] = 105,
  [106] = 106,
  [107] = 107,
  [108] = 108,
  [109] = 109,
  [110] = 110,
  [111] = 111,
  [112] = 112,
  [113] = 113,
  [114] = 114,
  [115] = 115,
  [116] = 116,
  [117] = 117,
  [118] = 118,
  [119] = 119,
  [120] = 120,
  [121] = 121,
  [122] = 122,
  [123] = 123,
  [124] = 124,
  [125] = 125,
  [126] = 126,
  [127] = 127,
  [128] = 128,
  [129] = 129,
  [130] = 130,
  [131] = 131,
  [132] = 132,
  [133] = 133,
  [134] = 134,
  [135] = 135,
  [136] = 136,
  [137] = 137,
  [138] = 138,
  [139] = 139,
  [140] = 140,
  [141] = 141,
  [142] = 142,
  [143] = 143,
  [144] = 144,
  [145] = 145,
  [146] = 146,
  [147] = 147,
  [148] = 148,
  [149] = 149,
  [150] = 150,
  [151] = 151,
  [152] = 152,
  [153] = 153,
  [154] = 154,
  [155] = 155,
  [156] = 156,
  [157] = 157,
  [158] = 158,
  [159] = 159,
  [160] = 160,
  [161] = 161,
  [162] = 162,
  [163] = 163,
  [164] = 164,
  [165] = 165,
  [166] = 166,
  [167] = 167,
  [168] = 168,
  [169] = 169,
  [170] = 170,
  [171] = 171,
  [172] = 172,
  [173] = 173,
  [174] = 174,
  [175] = 175,
  [176] = 176,
  [177] = 177,
  [178] = 178,
  [179] = 179,
  [180] = 180,
  [181] = 181,
  [182] = 182,
  [183] = 183,
  [184] = 184,
  [185] = 185,
  [186] = 186,
  [187] = 187,
  [188] = 188,
  [189] = 189,
  [190] = 190,
  [191] = 191,
  [192] = 192,
  [193] = 193,
  [194] = 194,
  [195] = 195,
  [196] = 196,
  [197] = 197,
  [198] = 198,
  [199] = 199,
  [200] = 200,
  [201] = 201,
  [202] = 202,
  [203] = 203,
  [204] = 204,
  [205] = 205,
  [206] = 206,
  [207] = 207,
  [208] = 208,
  [209] = 209,
  [210] = 210,
  [211] = 211,
  [212] = 212,
  [213] = 213,
  [214] = 214,
  [215] = 215,
  [216] = 216,
  [217] = 217,
  [218] = 218,
  [219] = 219,
  [220] = 220,
  [221] = 221,
  [222] = 222,
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(314);
      if (lookahead == '"') ADVANCE(4);
      if (lookahead == '#') ADVANCE(316);
      if (lookahead == '\'') ADVANCE(18);
      if (lookahead == '(') ADVANCE(344);
      if (lookahead == ')') ADVANCE(346);
      if (lookahead == ',') ADVANCE(345);
      if (lookahead == '-') ADVANCE(27);
      if (lookahead == '=') ADVANCE(343);
      if (lookahead == 'a') ADVANCE(50);
      if (lookahead == 'b') ADVANCE(29);
      if (lookahead == 'c') ADVANCE(138);
      if (lookahead == 'd') ADVANCE(91);
      if (lookahead == 'e') ADVANCE(301);
      if (lookahead == 'f') ADVANCE(30);
      if (lookahead == 'g') ADVANCE(218);
      if (lookahead == 'h') ADVANCE(31);
      if (lookahead == 'i') ADVANCE(124);
      if (lookahead == 'l') ADVANCE(90);
      if (lookahead == 'm') ADVANCE(32);
      if (lookahead == 'n') ADVANCE(33);
      if (lookahead == 'o') ADVANCE(186);
      if (lookahead == 'p') ADVANCE(166);
      if (lookahead == 'r') ADVANCE(14);
      if (lookahead == 's') ADVANCE(94);
      if (lookahead == 't') ADVANCE(87);
      if (lookahead == 'u') ADVANCE(192);
      if (lookahead == 'v') ADVANCE(147);
      if (lookahead == 'w') ADVANCE(92);
      if (lookahead == '{') ADVANCE(353);
      if (lookahead == '}') ADVANCE(354);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(0)
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(329);
      END_STATE();
    case 1:
      if (lookahead == '"') ADVANCE(4);
      if (lookahead == '#') ADVANCE(315);
      if (lookahead == '\'') ADVANCE(18);
      if (lookahead == '(') ADVANCE(344);
      if (lookahead == ')') ADVANCE(346);
      if (lookahead == ',') ADVANCE(345);
      if (lookahead == '-') ADVANCE(327);
      if (lookahead == 'r') ADVANCE(318);
      if (lookahead == '{') ADVANCE(353);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(1)
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(327);
      if (lookahead == ':' ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 2:
      if (lookahead == '"') ADVANCE(4);
      if (lookahead == '#') ADVANCE(315);
      if (lookahead == '\'') ADVANCE(18);
      if (lookahead == '-') ADVANCE(327);
      if (lookahead == 'd') ADVANCE(325);
      if (lookahead == 'i') ADVANCE(320);
      if (lookahead == 'r') ADVANCE(318);
      if (lookahead == '{') ADVANCE(353);
      if (lookahead == '}') ADVANCE(354);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(2)
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(327);
      if (lookahead == ':' ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 3:
      if (lookahead == '"') ADVANCE(4);
      if (lookahead == '#') ADVANCE(315);
      if (lookahead == '\'') ADVANCE(18);
      if (lookahead == '-') ADVANCE(327);
      if (lookahead == 'o') ADVANCE(323);
      if (lookahead == 'r') ADVANCE(318);
      if (lookahead == 'w') ADVANCE(321);
      if (lookahead == '{') ADVANCE(353);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(3)
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(327);
      if (lookahead == ':' ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 4:
      if (lookahead == '"') ADVANCE(331);
      if (lookahead == '\\') ADVANCE(311);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(5);
      END_STATE();
    case 5:
      if (lookahead == '"') ADVANCE(330);
      if (lookahead == '\\') ADVANCE(311);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(5);
      END_STATE();
    case 6:
      if (lookahead == '"') ADVANCE(7);
      END_STATE();
    case 7:
      if (lookahead == '"') ADVANCE(16);
      if (lookahead != 0) ADVANCE(7);
      END_STATE();
    case 8:
      if (lookahead == '"') ADVANCE(334);
      if (lookahead != 0) ADVANCE(10);
      END_STATE();
    case 9:
      if (lookahead == '"') ADVANCE(335);
      if (lookahead != 0) ADVANCE(10);
      END_STATE();
    case 10:
      if (lookahead == '"') ADVANCE(11);
      if (lookahead == '\\') ADVANCE(309);
      if (lookahead != 0) ADVANCE(10);
      END_STATE();
    case 11:
      if (lookahead == '"') ADVANCE(8);
      if (lookahead != 0) ADVANCE(10);
      END_STATE();
    case 12:
      if (lookahead == '"') ADVANCE(9);
      if (lookahead == '\\') ADVANCE(309);
      if (lookahead != 0) ADVANCE(10);
      END_STATE();
    case 13:
      if (lookahead == '#') ADVANCE(316);
      if (lookahead == 'c') ADVANCE(213);
      if (lookahead == 'i') ADVANCE(191);
      if (lookahead == 'o') ADVANCE(226);
      if (lookahead == 'r') ADVANCE(123);
      if (lookahead == 's') ADVANCE(284);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(13)
      END_STATE();
    case 14:
      if (lookahead == '#') ADVANCE(6);
      if (lookahead == 'a') ADVANCE(193);
      if (lookahead == 'e') ADVANCE(40);
      if (lookahead == 'o') ADVANCE(209);
      END_STATE();
    case 15:
      if (lookahead == '#') ADVANCE(6);
      if (lookahead == 'e') ADVANCE(228);
      if (lookahead == 'o') ADVANCE(208);
      END_STATE();
    case 16:
      if (lookahead == '#') ADVANCE(338);
      if (lookahead != 0) ADVANCE(7);
      END_STATE();
    case 17:
      if (lookahead == '#') ADVANCE(315);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(17)
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 18:
      if (lookahead == '\'') ADVANCE(333);
      if (lookahead == '\\') ADVANCE(312);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(19);
      END_STATE();
    case 19:
      if (lookahead == '\'') ADVANCE(332);
      if (lookahead == '\\') ADVANCE(312);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(19);
      END_STATE();
    case 20:
      if (lookahead == '\'') ADVANCE(23);
      if (lookahead == '\\') ADVANCE(310);
      if (lookahead != 0) ADVANCE(20);
      END_STATE();
    case 21:
      if (lookahead == '\'') ADVANCE(336);
      if (lookahead != 0) ADVANCE(20);
      END_STATE();
    case 22:
      if (lookahead == '\'') ADVANCE(337);
      if (lookahead != 0) ADVANCE(20);
      END_STATE();
    case 23:
      if (lookahead == '\'') ADVANCE(21);
      if (lookahead != 0) ADVANCE(20);
      END_STATE();
    case 24:
      if (lookahead == '\'') ADVANCE(22);
      if (lookahead == '\\') ADVANCE(310);
      if (lookahead != 0) ADVANCE(20);
      END_STATE();
    case 25:
      if (lookahead == '-') ADVANCE(85);
      END_STATE();
    case 26:
      if (lookahead == '>') ADVANCE(372);
      END_STATE();
    case 27:
      if (lookahead == '>') ADVANCE(372);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(329);
      END_STATE();
    case 28:
      if (lookahead == '_') ADVANCE(127);
      END_STATE();
    case 29:
      if (lookahead == 'a') ADVANCE(248);
      if (lookahead == 'o') ADVANCE(235);
      END_STATE();
    case 30:
      if (lookahead == 'a') ADVANCE(173);
      if (lookahead == 'l') ADVANCE(34);
      END_STATE();
    case 31:
      if (lookahead == 'a') ADVANCE(223);
      END_STATE();
    case 32:
      if (lookahead == 'a') ADVANCE(60);
      if (lookahead == 'i') ADVANCE(256);
      if (lookahead == 'o') ADVANCE(298);
      END_STATE();
    case 33:
      if (lookahead == 'a') ADVANCE(181);
      if (lookahead == 'o') ADVANCE(234);
      if (lookahead == 'p') ADVANCE(52);
      END_STATE();
    case 34:
      if (lookahead == 'a') ADVANCE(128);
      END_STATE();
    case 35:
      if (lookahead == 'a') ADVANCE(240);
      END_STATE();
    case 36:
      if (lookahead == 'a') ADVANCE(51);
      END_STATE();
    case 37:
      if (lookahead == 'a') ADVANCE(308);
      END_STATE();
    case 38:
      if (lookahead == 'a') ADVANCE(167);
      END_STATE();
    case 39:
      if (lookahead == 'a') ADVANCE(163);
      END_STATE();
    case 40:
      if (lookahead == 'a') ADVANCE(58);
      if (lookahead == 'q') ADVANCE(294);
      if (lookahead == 's') ADVANCE(280);
      END_STATE();
    case 41:
      if (lookahead == 'a') ADVANCE(222);
      END_STATE();
    case 42:
      if (lookahead == 'a') ADVANCE(306);
      END_STATE();
    case 43:
      if (lookahead == 'a') ADVANCE(164);
      END_STATE();
    case 44:
      if (lookahead == 'a') ADVANCE(165);
      END_STATE();
    case 45:
      if (lookahead == 'a') ADVANCE(158);
      END_STATE();
    case 46:
      if (lookahead == 'a') ADVANCE(134);
      END_STATE();
    case 47:
      if (lookahead == 'a') ADVANCE(276);
      END_STATE();
    case 48:
      if (lookahead == 'a') ADVANCE(290);
      END_STATE();
    case 49:
      if (lookahead == 'a') ADVANCE(239);
      END_STATE();
    case 50:
      if (lookahead == 'b') ADVANCE(142);
      if (lookahead == 'c') ADVANCE(278);
      END_STATE();
    case 51:
      if (lookahead == 'b') ADVANCE(171);
      END_STATE();
    case 52:
      if (lookahead == 'c') ADVANCE(387);
      END_STATE();
    case 53:
      if (lookahead == 'c') ADVANCE(362);
      END_STATE();
    case 54:
      if (lookahead == 'c') ADVANCE(361);
      END_STATE();
    case 55:
      if (lookahead == 'c') ADVANCE(48);
      END_STATE();
    case 56:
      if (lookahead == 'c') ADVANCE(74);
      if (lookahead == 'l') ADVANCE(303);
      END_STATE();
    case 57:
      if (lookahead == 'c') ADVANCE(273);
      END_STATE();
    case 58:
      if (lookahead == 'c') ADVANCE(141);
      END_STATE();
    case 59:
      if (lookahead == 'c') ADVANCE(289);
      END_STATE();
    case 60:
      if (lookahead == 'd') ADVANCE(388);
      END_STATE();
    case 61:
      if (lookahead == 'd') ADVANCE(370);
      END_STATE();
    case 62:
      if (lookahead == 'd') ADVANCE(375);
      END_STATE();
    case 63:
      if (lookahead == 'd') ADVANCE(382);
      END_STATE();
    case 64:
      if (lookahead == 'd') ADVANCE(413);
      END_STATE();
    case 65:
      if (lookahead == 'd') ADVANCE(364);
      END_STATE();
    case 66:
      if (lookahead == 'd') ADVANCE(405);
      END_STATE();
    case 67:
      if (lookahead == 'd') ADVANCE(383);
      END_STATE();
    case 68:
      if (lookahead == 'd') ADVANCE(28);
      END_STATE();
    case 69:
      if (lookahead == 'd') ADVANCE(404);
      END_STATE();
    case 70:
      if (lookahead == 'd') ADVANCE(135);
      END_STATE();
    case 71:
      if (lookahead == 'd') ADVANCE(274);
      END_STATE();
    case 72:
      if (lookahead == 'd') ADVANCE(212);
      END_STATE();
    case 73:
      if (lookahead == 'e') ADVANCE(360);
      END_STATE();
    case 74:
      if (lookahead == 'e') ADVANCE(350);
      END_STATE();
    case 75:
      if (lookahead == 'e') ADVANCE(339);
      END_STATE();
    case 76:
      if (lookahead == 'e') ADVANCE(340);
      END_STATE();
    case 77:
      if (lookahead == 'e') ADVANCE(392);
      END_STATE();
    case 78:
      if (lookahead == 'e') ADVANCE(380);
      if (lookahead == 'u') ADVANCE(249);
      END_STATE();
    case 79:
      if (lookahead == 'e') ADVANCE(385);
      END_STATE();
    case 80:
      if (lookahead == 'e') ADVANCE(395);
      END_STATE();
    case 81:
      if (lookahead == 'e') ADVANCE(401);
      END_STATE();
    case 82:
      if (lookahead == 'e') ADVANCE(414);
      END_STATE();
    case 83:
      if (lookahead == 'e') ADVANCE(396);
      END_STATE();
    case 84:
      if (lookahead == 'e') ADVANCE(377);
      END_STATE();
    case 85:
      if (lookahead == 'e') ADVANCE(125);
      END_STATE();
    case 86:
      if (lookahead == 'e') ADVANCE(408);
      END_STATE();
    case 87:
      if (lookahead == 'e') ADVANCE(302);
      if (lookahead == 'i') ADVANCE(180);
      if (lookahead == 'r') ADVANCE(144);
      END_STATE();
    case 88:
      if (lookahead == 'e') ADVANCE(302);
      if (lookahead == 'r') ADVANCE(143);
      END_STATE();
    case 89:
      if (lookahead == 'e') ADVANCE(265);
      END_STATE();
    case 90:
      if (lookahead == 'e') ADVANCE(265);
      if (lookahead == 'o') ADVANCE(55);
      END_STATE();
    case 91:
      if (lookahead == 'e') ADVANCE(254);
      if (lookahead == 'i') ADVANCE(38);
      if (lookahead == 'o') ADVANCE(357);
      END_STATE();
    case 92:
      if (lookahead == 'e') ADVANCE(70);
      if (lookahead == 'h') ADVANCE(98);
      if (lookahead == 'i') ADVANCE(71);
      END_STATE();
    case 93:
      if (lookahead == 'e') ADVANCE(187);
      if (lookahead == 't') ADVANCE(150);
      END_STATE();
    case 94:
      if (lookahead == 'e') ADVANCE(266);
      if (lookahead == 'p') ADVANCE(145);
      if (lookahead == 't') ADVANCE(35);
      END_STATE();
    case 95:
      if (lookahead == 'e') ADVANCE(266);
      if (lookahead == 'p') ADVANCE(145);
      if (lookahead == 't') ADVANCE(49);
      END_STATE();
    case 96:
      if (lookahead == 'e') ADVANCE(176);
      END_STATE();
    case 97:
      if (lookahead == 'e') ADVANCE(236);
      END_STATE();
    case 98:
      if (lookahead == 'e') ADVANCE(188);
      END_STATE();
    case 99:
      if (lookahead == 'e') ADVANCE(61);
      END_STATE();
    case 100:
      if (lookahead == 'e') ADVANCE(198);
      END_STATE();
    case 101:
      if (lookahead == 'e') ADVANCE(62);
      END_STATE();
    case 102:
      if (lookahead == 'e') ADVANCE(230);
      END_STATE();
    case 103:
      if (lookahead == 'e') ADVANCE(63);
      END_STATE();
    case 104:
      if (lookahead == 'e') ADVANCE(231);
      END_STATE();
    case 105:
      if (lookahead == 'e') ADVANCE(64);
      END_STATE();
    case 106:
      if (lookahead == 'e') ADVANCE(182);
      END_STATE();
    case 107:
      if (lookahead == 'e') ADVANCE(232);
      END_STATE();
    case 108:
      if (lookahead == 'e') ADVANCE(65);
      END_STATE();
    case 109:
      if (lookahead == 'e') ADVANCE(66);
      END_STATE();
    case 110:
      if (lookahead == 'e') ADVANCE(233);
      END_STATE();
    case 111:
      if (lookahead == 'e') ADVANCE(67);
      END_STATE();
    case 112:
      if (lookahead == 'e') ADVANCE(68);
      END_STATE();
    case 113:
      if (lookahead == 'e') ADVANCE(271);
      END_STATE();
    case 114:
      if (lookahead == 'e') ADVANCE(69);
      END_STATE();
    case 115:
      if (lookahead == 'e') ADVANCE(260);
      END_STATE();
    case 116:
      if (lookahead == 'e') ADVANCE(185);
      END_STATE();
    case 117:
      if (lookahead == 'e') ADVANCE(241);
      END_STATE();
    case 118:
      if (lookahead == 'e') ADVANCE(257);
      if (lookahead == 'o') ADVANCE(200);
      END_STATE();
    case 119:
      if (lookahead == 'e') ADVANCE(258);
      END_STATE();
    case 120:
      if (lookahead == 'e') ADVANCE(57);
      END_STATE();
    case 121:
      if (lookahead == 'e') ADVANCE(199);
      END_STATE();
    case 122:
      if (lookahead == 'e') ADVANCE(286);
      END_STATE();
    case 123:
      if (lookahead == 'e') ADVANCE(229);
      if (lookahead == 'o') ADVANCE(220);
      END_STATE();
    case 124:
      if (lookahead == 'f') ADVANCE(355);
      if (lookahead == 'n') ADVANCE(416);
      if (lookahead == 't') ADVANCE(96);
      END_STATE();
    case 125:
      if (lookahead == 'f') ADVANCE(126);
      END_STATE();
    case 126:
      if (lookahead == 'f') ADVANCE(120);
      END_STATE();
    case 127:
      if (lookahead == 'f') ADVANCE(169);
      if (lookahead == 'i') ADVANCE(287);
      END_STATE();
    case 128:
      if (lookahead == 'g') ADVANCE(411);
      END_STATE();
    case 129:
      if (lookahead == 'g') ADVANCE(394);
      END_STATE();
    case 130:
      if (lookahead == 'g') ADVANCE(412);
      END_STATE();
    case 131:
      if (lookahead == 'g') ADVANCE(295);
      END_STATE();
    case 132:
      if (lookahead == 'g') ADVANCE(242);
      END_STATE();
    case 133:
      if (lookahead == 'g') ADVANCE(136);
      END_STATE();
    case 134:
      if (lookahead == 'g') ADVANCE(251);
      END_STATE();
    case 135:
      if (lookahead == 'g') ADVANCE(79);
      END_STATE();
    case 136:
      if (lookahead == 'g') ADVANCE(107);
      END_STATE();
    case 137:
      if (lookahead == 'h') ADVANCE(115);
      END_STATE();
    case 138:
      if (lookahead == 'h') ADVANCE(115);
      if (lookahead == 'l') ADVANCE(219);
      if (lookahead == 'o') ADVANCE(175);
      if (lookahead == 'u') ADVANCE(255);
      END_STATE();
    case 139:
      if (lookahead == 'h') ADVANCE(386);
      END_STATE();
    case 140:
      if (lookahead == 'h') ADVANCE(117);
      END_STATE();
    case 141:
      if (lookahead == 'h') ADVANCE(105);
      END_STATE();
    case 142:
      if (lookahead == 'i') ADVANCE(172);
      END_STATE();
    case 143:
      if (lookahead == 'i') ADVANCE(133);
      END_STATE();
    case 144:
      if (lookahead == 'i') ADVANCE(133);
      if (lookahead == 'u') ADVANCE(75);
      END_STATE();
    case 145:
      if (lookahead == 'i') ADVANCE(194);
      END_STATE();
    case 146:
      if (lookahead == 'i') ADVANCE(227);
      END_STATE();
    case 147:
      if (lookahead == 'i') ADVANCE(262);
      END_STATE();
    case 148:
      if (lookahead == 'i') ADVANCE(59);
      END_STATE();
    case 149:
      if (lookahead == 'i') ADVANCE(267);
      END_STATE();
    case 150:
      if (lookahead == 'i') ADVANCE(214);
      END_STATE();
    case 151:
      if (lookahead == 'i') ADVANCE(195);
      END_STATE();
    case 152:
      if (lookahead == 'i') ADVANCE(196);
      END_STATE();
    case 153:
      if (lookahead == 'i') ADVANCE(288);
      END_STATE();
    case 154:
      if (lookahead == 'i') ADVANCE(277);
      END_STATE();
    case 155:
      if (lookahead == 'i') ADVANCE(299);
      END_STATE();
    case 156:
      if (lookahead == 'i') ADVANCE(216);
      END_STATE();
    case 157:
      if (lookahead == 'i') ADVANCE(217);
      END_STATE();
    case 158:
      if (lookahead == 'i') ADVANCE(204);
      END_STATE();
    case 159:
      if (lookahead == 'i') ADVANCE(245);
      END_STATE();
    case 160:
      if (lookahead == 'i') ADVANCE(246);
      END_STATE();
    case 161:
      if (lookahead == 'i') ADVANCE(247);
      END_STATE();
    case 162:
      if (lookahead == 'l') ADVANCE(303);
      END_STATE();
    case 163:
      if (lookahead == 'l') ADVANCE(402);
      END_STATE();
    case 164:
      if (lookahead == 'l') ADVANCE(368);
      END_STATE();
    case 165:
      if (lookahead == 'l') ADVANCE(406);
      END_STATE();
    case 166:
      if (lookahead == 'l') ADVANCE(37);
      if (lookahead == 'o') ADVANCE(243);
      if (lookahead == 'r') ADVANCE(207);
      END_STATE();
    case 167:
      if (lookahead == 'l') ADVANCE(211);
      END_STATE();
    case 168:
      if (lookahead == 'l') ADVANCE(42);
      END_STATE();
    case 169:
      if (lookahead == 'l') ADVANCE(46);
      END_STATE();
    case 170:
      if (lookahead == 'l') ADVANCE(122);
      END_STATE();
    case 171:
      if (lookahead == 'l') ADVANCE(84);
      END_STATE();
    case 172:
      if (lookahead == 'l') ADVANCE(154);
      END_STATE();
    case 173:
      if (lookahead == 'l') ADVANCE(261);
      END_STATE();
    case 174:
      if (lookahead == 'm') ADVANCE(225);
      END_STATE();
    case 175:
      if (lookahead == 'm') ADVANCE(225);
      if (lookahead == 'n') ADVANCE(281);
      END_STATE();
    case 176:
      if (lookahead == 'm') ADVANCE(376);
      END_STATE();
    case 177:
      if (lookahead == 'm') ADVANCE(359);
      END_STATE();
    case 178:
      if (lookahead == 'm') ADVANCE(389);
      END_STATE();
    case 179:
      if (lookahead == 'm') ADVANCE(391);
      END_STATE();
    case 180:
      if (lookahead == 'm') ADVANCE(151);
      END_STATE();
    case 181:
      if (lookahead == 'm') ADVANCE(73);
      END_STATE();
    case 182:
      if (lookahead == 'm') ADVANCE(252);
      END_STATE();
    case 183:
      if (lookahead == 'm') ADVANCE(253);
      END_STATE();
    case 184:
      if (lookahead == 'm') ADVANCE(43);
      END_STATE();
    case 185:
      if (lookahead == 'm') ADVANCE(121);
      END_STATE();
    case 186:
      if (lookahead == 'n') ADVANCE(56);
      if (lookahead == 'p') ADVANCE(93);
      if (lookahead == 'v') ADVANCE(97);
      END_STATE();
    case 187:
      if (lookahead == 'n') ADVANCE(381);
      END_STATE();
    case 188:
      if (lookahead == 'n') ADVANCE(351);
      END_STATE();
    case 189:
      if (lookahead == 'n') ADVANCE(397);
      END_STATE();
    case 190:
      if (lookahead == 'n') ADVANCE(363);
      END_STATE();
    case 191:
      if (lookahead == 'n') ADVANCE(415);
      END_STATE();
    case 192:
      if (lookahead == 'n') ADVANCE(259);
      END_STATE();
    case 193:
      if (lookahead == 'n') ADVANCE(72);
      END_STATE();
    case 194:
      if (lookahead == 'n') ADVANCE(202);
      END_STATE();
    case 195:
      if (lookahead == 'n') ADVANCE(129);
      END_STATE();
    case 196:
      if (lookahead == 'n') ADVANCE(130);
      END_STATE();
    case 197:
      if (lookahead == 'n') ADVANCE(162);
      END_STATE();
    case 198:
      if (lookahead == 'n') ADVANCE(282);
      END_STATE();
    case 199:
      if (lookahead == 'n') ADVANCE(272);
      END_STATE();
    case 200:
      if (lookahead == 'n') ADVANCE(86);
      END_STATE();
    case 201:
      if (lookahead == 'n') ADVANCE(300);
      if (lookahead == 't') ADVANCE(96);
      END_STATE();
    case 202:
      if (lookahead == 'n') ADVANCE(104);
      END_STATE();
    case 203:
      if (lookahead == 'n') ADVANCE(44);
      END_STATE();
    case 204:
      if (lookahead == 'n') ADVANCE(110);
      END_STATE();
    case 205:
      if (lookahead == 'o') ADVANCE(234);
      if (lookahead == 'p') ADVANCE(52);
      END_STATE();
    case 206:
      if (lookahead == 'o') ADVANCE(292);
      END_STATE();
    case 207:
      if (lookahead == 'o') ADVANCE(132);
      END_STATE();
    case 208:
      if (lookahead == 'o') ADVANCE(177);
      END_STATE();
    case 209:
      if (lookahead == 'o') ADVANCE(177);
      if (lookahead == 'u') ADVANCE(285);
      END_STATE();
    case 210:
      if (lookahead == 'o') ADVANCE(178);
      END_STATE();
    case 211:
      if (lookahead == 'o') ADVANCE(131);
      END_STATE();
    case 212:
      if (lookahead == 'o') ADVANCE(179);
      END_STATE();
    case 213:
      if (lookahead == 'o') ADVANCE(174);
      END_STATE();
    case 214:
      if (lookahead == 'o') ADVANCE(203);
      END_STATE();
    case 215:
      if (lookahead == 'o') ADVANCE(237);
      END_STATE();
    case 216:
      if (lookahead == 'o') ADVANCE(189);
      END_STATE();
    case 217:
      if (lookahead == 'o') ADVANCE(190);
      END_STATE();
    case 218:
      if (lookahead == 'o') ADVANCE(39);
      if (lookahead == 'r') ADVANCE(206);
      END_STATE();
    case 219:
      if (lookahead == 'o') ADVANCE(264);
      END_STATE();
    case 220:
      if (lookahead == 'o') ADVANCE(183);
      END_STATE();
    case 221:
      if (lookahead == 'p') ADVANCE(403);
      END_STATE();
    case 222:
      if (lookahead == 'p') ADVANCE(224);
      END_STATE();
    case 223:
      if (lookahead == 'p') ADVANCE(224);
      if (lookahead == 's') ADVANCE(410);
      END_STATE();
    case 224:
      if (lookahead == 'p') ADVANCE(304);
      END_STATE();
    case 225:
      if (lookahead == 'p') ADVANCE(170);
      END_STATE();
    case 226:
      if (lookahead == 'p') ADVANCE(279);
      END_STATE();
    case 227:
      if (lookahead == 'p') ADVANCE(291);
      END_STATE();
    case 228:
      if (lookahead == 'q') ADVANCE(296);
      END_STATE();
    case 229:
      if (lookahead == 'q') ADVANCE(297);
      END_STATE();
    case 230:
      if (lookahead == 'r') ADVANCE(400);
      END_STATE();
    case 231:
      if (lookahead == 'r') ADVANCE(384);
      END_STATE();
    case 232:
      if (lookahead == 'r') ADVANCE(347);
      END_STATE();
    case 233:
      if (lookahead == 'r') ADVANCE(379);
      END_STATE();
    case 234:
      if (lookahead == 'r') ADVANCE(184);
      if (lookahead == 'w') ADVANCE(140);
      END_STATE();
    case 235:
      if (lookahead == 'r') ADVANCE(99);
      END_STATE();
    case 236:
      if (lookahead == 'r') ADVANCE(168);
      END_STATE();
    case 237:
      if (lookahead == 'r') ADVANCE(307);
      END_STATE();
    case 238:
      if (lookahead == 'r') ADVANCE(148);
      END_STATE();
    case 239:
      if (lookahead == 'r') ADVANCE(270);
      END_STATE();
    case 240:
      if (lookahead == 'r') ADVANCE(270);
      if (lookahead == 't') ADVANCE(78);
      END_STATE();
    case 241:
      if (lookahead == 'r') ADVANCE(81);
      END_STATE();
    case 242:
      if (lookahead == 'r') ADVANCE(119);
      END_STATE();
    case 243:
      if (lookahead == 'r') ADVANCE(283);
      END_STATE();
    case 244:
      if (lookahead == 'r') ADVANCE(101);
      END_STATE();
    case 245:
      if (lookahead == 'r') ADVANCE(109);
      END_STATE();
    case 246:
      if (lookahead == 'r') ADVANCE(112);
      END_STATE();
    case 247:
      if (lookahead == 'r') ADVANCE(114);
      END_STATE();
    case 248:
      if (lookahead == 'r') ADVANCE(244);
      END_STATE();
    case 249:
      if (lookahead == 's') ADVANCE(25);
      END_STATE();
    case 250:
      if (lookahead == 's') ADVANCE(417);
      END_STATE();
    case 251:
      if (lookahead == 's') ADVANCE(373);
      END_STATE();
    case 252:
      if (lookahead == 's') ADVANCE(374);
      END_STATE();
    case 253:
      if (lookahead == 's') ADVANCE(393);
      END_STATE();
    case 254:
      if (lookahead == 's') ADVANCE(53);
      END_STATE();
    case 255:
      if (lookahead == 's') ADVANCE(275);
      END_STATE();
    case 256:
      if (lookahead == 's') ADVANCE(263);
      END_STATE();
    case 257:
      if (lookahead == 's') ADVANCE(54);
      END_STATE();
    case 258:
      if (lookahead == 's') ADVANCE(250);
      END_STATE();
    case 259:
      if (lookahead == 's') ADVANCE(113);
      END_STATE();
    case 260:
      if (lookahead == 's') ADVANCE(269);
      END_STATE();
    case 261:
      if (lookahead == 's') ADVANCE(76);
      END_STATE();
    case 262:
      if (lookahead == 's') ADVANCE(153);
      END_STATE();
    case 263:
      if (lookahead == 's') ADVANCE(152);
      END_STATE();
    case 264:
      if (lookahead == 's') ADVANCE(103);
      END_STATE();
    case 265:
      if (lookahead == 't') ADVANCE(341);
      END_STATE();
    case 266:
      if (lookahead == 't') ADVANCE(342);
      END_STATE();
    case 267:
      if (lookahead == 't') ADVANCE(371);
      END_STATE();
    case 268:
      if (lookahead == 't') ADVANCE(367);
      END_STATE();
    case 269:
      if (lookahead == 't') ADVANCE(398);
      END_STATE();
    case 270:
      if (lookahead == 't') ADVANCE(409);
      END_STATE();
    case 271:
      if (lookahead == 't') ADVANCE(366);
      END_STATE();
    case 272:
      if (lookahead == 't') ADVANCE(390);
      END_STATE();
    case 273:
      if (lookahead == 't') ADVANCE(407);
      END_STATE();
    case 274:
      if (lookahead == 't') ADVANCE(139);
      END_STATE();
    case 275:
      if (lookahead == 't') ADVANCE(210);
      END_STATE();
    case 276:
      if (lookahead == 't') ADVANCE(293);
      END_STATE();
    case 277:
      if (lookahead == 't') ADVANCE(305);
      END_STATE();
    case 278:
      if (lookahead == 't') ADVANCE(155);
      END_STATE();
    case 279:
      if (lookahead == 't') ADVANCE(150);
      END_STATE();
    case 280:
      if (lookahead == 't') ADVANCE(238);
      END_STATE();
    case 281:
      if (lookahead == 't') ADVANCE(45);
      END_STATE();
    case 282:
      if (lookahead == 't') ADVANCE(215);
      END_STATE();
    case 283:
      if (lookahead == 't') ADVANCE(36);
      END_STATE();
    case 284:
      if (lookahead == 't') ADVANCE(47);
      END_STATE();
    case 285:
      if (lookahead == 't') ADVANCE(77);
      END_STATE();
    case 286:
      if (lookahead == 't') ADVANCE(82);
      END_STATE();
    case 287:
      if (lookahead == 't') ADVANCE(106);
      END_STATE();
    case 288:
      if (lookahead == 't') ADVANCE(108);
      END_STATE();
    case 289:
      if (lookahead == 't') ADVANCE(111);
      END_STATE();
    case 290:
      if (lookahead == 't') ADVANCE(156);
      END_STATE();
    case 291:
      if (lookahead == 't') ADVANCE(157);
      END_STATE();
    case 292:
      if (lookahead == 'u') ADVANCE(221);
      END_STATE();
    case 293:
      if (lookahead == 'u') ADVANCE(249);
      END_STATE();
    case 294:
      if (lookahead == 'u') ADVANCE(159);
      END_STATE();
    case 295:
      if (lookahead == 'u') ADVANCE(83);
      END_STATE();
    case 296:
      if (lookahead == 'u') ADVANCE(160);
      END_STATE();
    case 297:
      if (lookahead == 'u') ADVANCE(161);
      END_STATE();
    case 298:
      if (lookahead == 'v') ADVANCE(116);
      END_STATE();
    case 299:
      if (lookahead == 'v') ADVANCE(80);
      END_STATE();
    case 300:
      if (lookahead == 'v') ADVANCE(100);
      END_STATE();
    case 301:
      if (lookahead == 'x') ADVANCE(149);
      END_STATE();
    case 302:
      if (lookahead == 'x') ADVANCE(268);
      END_STATE();
    case 303:
      if (lookahead == 'y') ADVANCE(348);
      END_STATE();
    case 304:
      if (lookahead == 'y') ADVANCE(369);
      END_STATE();
    case 305:
      if (lookahead == 'y') ADVANCE(378);
      END_STATE();
    case 306:
      if (lookahead == 'y') ADVANCE(365);
      END_STATE();
    case 307:
      if (lookahead == 'y') ADVANCE(399);
      END_STATE();
    case 308:
      if (lookahead == 'y') ADVANCE(102);
      END_STATE();
    case 309:
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(10);
      if (lookahead == '"') ADVANCE(12);
      if (lookahead == '\\') ADVANCE(309);
      END_STATE();
    case 310:
      if (lookahead != 0 &&
          lookahead != '\'' &&
          lookahead != '\\') ADVANCE(20);
      if (lookahead == '\'') ADVANCE(24);
      if (lookahead == '\\') ADVANCE(310);
      END_STATE();
    case 311:
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(5);
      END_STATE();
    case 312:
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(19);
      END_STATE();
    case 313:
      if (eof) ADVANCE(314);
      if (lookahead == '"') ADVANCE(4);
      if (lookahead == '#') ADVANCE(316);
      if (lookahead == '\'') ADVANCE(18);
      if (lookahead == ',') ADVANCE(345);
      if (lookahead == '-') ADVANCE(26);
      if (lookahead == 'b') ADVANCE(29);
      if (lookahead == 'c') ADVANCE(137);
      if (lookahead == 'd') ADVANCE(118);
      if (lookahead == 'g') ADVANCE(218);
      if (lookahead == 'h') ADVANCE(41);
      if (lookahead == 'i') ADVANCE(201);
      if (lookahead == 'l') ADVANCE(89);
      if (lookahead == 'n') ADVANCE(205);
      if (lookahead == 'o') ADVANCE(197);
      if (lookahead == 'r') ADVANCE(15);
      if (lookahead == 's') ADVANCE(95);
      if (lookahead == 't') ADVANCE(88);
      if (lookahead == 'u') ADVANCE(192);
      if (lookahead == 'w') ADVANCE(92);
      if (lookahead == '{') ADVANCE(353);
      if (lookahead == '}') ADVANCE(354);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(313)
      END_STATE();
    case 314:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 315:
      ACCEPT_TOKEN(sym_comment);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(315);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(316);
      END_STATE();
    case 316:
      ACCEPT_TOKEN(sym_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(316);
      END_STATE();
    case 317:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '"') ADVANCE(7);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 318:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '#') ADVANCE(317);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 319:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(324);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 320:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'f') ADVANCE(356);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 321:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'h') ADVANCE(319);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 322:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(326);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 323:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(322);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 324:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(352);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 325:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(358);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 326:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'y') ADVANCE(349);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 327:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(327);
      if (lookahead == '#' ||
          lookahead == '-' ||
          lookahead == ':' ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 328:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 329:
      ACCEPT_TOKEN(sym_number);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(329);
      END_STATE();
    case 330:
      ACCEPT_TOKEN(aux_sym_string_token1);
      END_STATE();
    case 331:
      ACCEPT_TOKEN(aux_sym_string_token1);
      if (lookahead == '"') ADVANCE(10);
      END_STATE();
    case 332:
      ACCEPT_TOKEN(aux_sym_string_token2);
      END_STATE();
    case 333:
      ACCEPT_TOKEN(aux_sym_string_token2);
      if (lookahead == '\'') ADVANCE(20);
      END_STATE();
    case 334:
      ACCEPT_TOKEN(aux_sym_string_token3);
      END_STATE();
    case 335:
      ACCEPT_TOKEN(aux_sym_string_token3);
      if (lookahead == '"') ADVANCE(334);
      if (lookahead != 0) ADVANCE(10);
      END_STATE();
    case 336:
      ACCEPT_TOKEN(aux_sym_string_token4);
      END_STATE();
    case 337:
      ACCEPT_TOKEN(aux_sym_string_token4);
      if (lookahead == '\'') ADVANCE(336);
      if (lookahead != 0) ADVANCE(20);
      END_STATE();
    case 338:
      ACCEPT_TOKEN(aux_sym_string_token5);
      END_STATE();
    case 339:
      ACCEPT_TOKEN(anon_sym_true);
      END_STATE();
    case 340:
      ACCEPT_TOKEN(anon_sym_false);
      END_STATE();
    case 341:
      ACCEPT_TOKEN(anon_sym_let);
      END_STATE();
    case 342:
      ACCEPT_TOKEN(anon_sym_set);
      END_STATE();
    case 343:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 344:
      ACCEPT_TOKEN(anon_sym_LPAREN);
      END_STATE();
    case 345:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 346:
      ACCEPT_TOKEN(anon_sym_RPAREN);
      END_STATE();
    case 347:
      ACCEPT_TOKEN(anon_sym_trigger);
      END_STATE();
    case 348:
      ACCEPT_TOKEN(anon_sym_only);
      END_STATE();
    case 349:
      ACCEPT_TOKEN(anon_sym_only);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 350:
      ACCEPT_TOKEN(anon_sym_once);
      END_STATE();
    case 351:
      ACCEPT_TOKEN(anon_sym_when);
      END_STATE();
    case 352:
      ACCEPT_TOKEN(anon_sym_when);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 353:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 354:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 355:
      ACCEPT_TOKEN(anon_sym_if);
      END_STATE();
    case 356:
      ACCEPT_TOKEN(anon_sym_if);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 357:
      ACCEPT_TOKEN(anon_sym_do);
      END_STATE();
    case 358:
      ACCEPT_TOKEN(anon_sym_do);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 359:
      ACCEPT_TOKEN(anon_sym_room);
      END_STATE();
    case 360:
      ACCEPT_TOKEN(anon_sym_name);
      END_STATE();
    case 361:
      ACCEPT_TOKEN(anon_sym_desc);
      END_STATE();
    case 362:
      ACCEPT_TOKEN(anon_sym_desc);
      if (lookahead == 'r') ADVANCE(146);
      END_STATE();
    case 363:
      ACCEPT_TOKEN(anon_sym_description);
      END_STATE();
    case 364:
      ACCEPT_TOKEN(anon_sym_visited);
      END_STATE();
    case 365:
      ACCEPT_TOKEN(anon_sym_overlay);
      END_STATE();
    case 366:
      ACCEPT_TOKEN(anon_sym_unset);
      END_STATE();
    case 367:
      ACCEPT_TOKEN(anon_sym_text);
      END_STATE();
    case 368:
      ACCEPT_TOKEN(anon_sym_normal);
      END_STATE();
    case 369:
      ACCEPT_TOKEN(anon_sym_happy);
      END_STATE();
    case 370:
      ACCEPT_TOKEN(anon_sym_bored);
      END_STATE();
    case 371:
      ACCEPT_TOKEN(anon_sym_exit);
      END_STATE();
    case 372:
      ACCEPT_TOKEN(anon_sym_DASH_GT);
      END_STATE();
    case 373:
      ACCEPT_TOKEN(anon_sym_required_flags);
      END_STATE();
    case 374:
      ACCEPT_TOKEN(anon_sym_required_items);
      END_STATE();
    case 375:
      ACCEPT_TOKEN(anon_sym_barred);
      END_STATE();
    case 376:
      ACCEPT_TOKEN(anon_sym_item);
      END_STATE();
    case 377:
      ACCEPT_TOKEN(anon_sym_portable);
      END_STATE();
    case 378:
      ACCEPT_TOKEN(anon_sym_ability);
      END_STATE();
    case 379:
      ACCEPT_TOKEN(anon_sym_container);
      END_STATE();
    case 380:
      ACCEPT_TOKEN(anon_sym_state);
      END_STATE();
    case 381:
      ACCEPT_TOKEN(anon_sym_open);
      END_STATE();
    case 382:
      ACCEPT_TOKEN(anon_sym_closed);
      END_STATE();
    case 383:
      ACCEPT_TOKEN(anon_sym_restricted);
      END_STATE();
    case 384:
      ACCEPT_TOKEN(anon_sym_spinner);
      END_STATE();
    case 385:
      ACCEPT_TOKEN(anon_sym_wedge);
      END_STATE();
    case 386:
      ACCEPT_TOKEN(anon_sym_width);
      END_STATE();
    case 387:
      ACCEPT_TOKEN(anon_sym_npc);
      END_STATE();
    case 388:
      ACCEPT_TOKEN(anon_sym_mad);
      END_STATE();
    case 389:
      ACCEPT_TOKEN(anon_sym_custom);
      END_STATE();
    case 390:
      ACCEPT_TOKEN(anon_sym_movement);
      END_STATE();
    case 391:
      ACCEPT_TOKEN(anon_sym_random);
      END_STATE();
    case 392:
      ACCEPT_TOKEN(anon_sym_route);
      END_STATE();
    case 393:
      ACCEPT_TOKEN(anon_sym_rooms);
      END_STATE();
    case 394:
      ACCEPT_TOKEN(anon_sym_timing);
      END_STATE();
    case 395:
      ACCEPT_TOKEN(anon_sym_active);
      END_STATE();
    case 396:
      ACCEPT_TOKEN(anon_sym_dialogue);
      END_STATE();
    case 397:
      ACCEPT_TOKEN(anon_sym_location);
      END_STATE();
    case 398:
      ACCEPT_TOKEN(anon_sym_chest);
      END_STATE();
    case 399:
      ACCEPT_TOKEN(anon_sym_inventory);
      END_STATE();
    case 400:
      ACCEPT_TOKEN(anon_sym_player);
      END_STATE();
    case 401:
      ACCEPT_TOKEN(anon_sym_nowhere);
      END_STATE();
    case 402:
      ACCEPT_TOKEN(anon_sym_goal);
      END_STATE();
    case 403:
      ACCEPT_TOKEN(anon_sym_group);
      END_STATE();
    case 404:
      ACCEPT_TOKEN(anon_sym_required);
      END_STATE();
    case 405:
      ACCEPT_TOKEN(anon_sym_required);
      if (lookahead == '_') ADVANCE(127);
      END_STATE();
    case 406:
      ACCEPT_TOKEN(anon_sym_optional);
      END_STATE();
    case 407:
      ACCEPT_TOKEN(anon_sym_status_DASHeffect);
      END_STATE();
    case 408:
      ACCEPT_TOKEN(anon_sym_done);
      END_STATE();
    case 409:
      ACCEPT_TOKEN(anon_sym_start);
      END_STATE();
    case 410:
      ACCEPT_TOKEN(anon_sym_has);
      END_STATE();
    case 411:
      ACCEPT_TOKEN(anon_sym_flag);
      END_STATE();
    case 412:
      ACCEPT_TOKEN(anon_sym_missing);
      END_STATE();
    case 413:
      ACCEPT_TOKEN(anon_sym_reached);
      END_STATE();
    case 414:
      ACCEPT_TOKEN(anon_sym_complete);
      END_STATE();
    case 415:
      ACCEPT_TOKEN(anon_sym_in);
      END_STATE();
    case 416:
      ACCEPT_TOKEN(anon_sym_in);
      if (lookahead == 'v') ADVANCE(100);
      END_STATE();
    case 417:
      ACCEPT_TOKEN(anon_sym_progress);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 0},
  [2] = {.lex_state = 313},
  [3] = {.lex_state = 0},
  [4] = {.lex_state = 0},
  [5] = {.lex_state = 0},
  [6] = {.lex_state = 313},
  [7] = {.lex_state = 313},
  [8] = {.lex_state = 313},
  [9] = {.lex_state = 0},
  [10] = {.lex_state = 0},
  [11] = {.lex_state = 0},
  [12] = {.lex_state = 0},
  [13] = {.lex_state = 0},
  [14] = {.lex_state = 0},
  [15] = {.lex_state = 0},
  [16] = {.lex_state = 0},
  [17] = {.lex_state = 0},
  [18] = {.lex_state = 2},
  [19] = {.lex_state = 0},
  [20] = {.lex_state = 0},
  [21] = {.lex_state = 1},
  [22] = {.lex_state = 1},
  [23] = {.lex_state = 1},
  [24] = {.lex_state = 2},
  [25] = {.lex_state = 1},
  [26] = {.lex_state = 3},
  [27] = {.lex_state = 313},
  [28] = {.lex_state = 313},
  [29] = {.lex_state = 313},
  [30] = {.lex_state = 313},
  [31] = {.lex_state = 313},
  [32] = {.lex_state = 313},
  [33] = {.lex_state = 313},
  [34] = {.lex_state = 3},
  [35] = {.lex_state = 2},
  [36] = {.lex_state = 1},
  [37] = {.lex_state = 0},
  [38] = {.lex_state = 0},
  [39] = {.lex_state = 0},
  [40] = {.lex_state = 0},
  [41] = {.lex_state = 0},
  [42] = {.lex_state = 0},
  [43] = {.lex_state = 0},
  [44] = {.lex_state = 1},
  [45] = {.lex_state = 0},
  [46] = {.lex_state = 3},
  [47] = {.lex_state = 0},
  [48] = {.lex_state = 0},
  [49] = {.lex_state = 0},
  [50] = {.lex_state = 313},
  [51] = {.lex_state = 0},
  [52] = {.lex_state = 0},
  [53] = {.lex_state = 0},
  [54] = {.lex_state = 313},
  [55] = {.lex_state = 313},
  [56] = {.lex_state = 0},
  [57] = {.lex_state = 0},
  [58] = {.lex_state = 1},
  [59] = {.lex_state = 0},
  [60] = {.lex_state = 0},
  [61] = {.lex_state = 0},
  [62] = {.lex_state = 0},
  [63] = {.lex_state = 0},
  [64] = {.lex_state = 0},
  [65] = {.lex_state = 0},
  [66] = {.lex_state = 0},
  [67] = {.lex_state = 0},
  [68] = {.lex_state = 0},
  [69] = {.lex_state = 0},
  [70] = {.lex_state = 0},
  [71] = {.lex_state = 0},
  [72] = {.lex_state = 0},
  [73] = {.lex_state = 0},
  [74] = {.lex_state = 0},
  [75] = {.lex_state = 0},
  [76] = {.lex_state = 0},
  [77] = {.lex_state = 0},
  [78] = {.lex_state = 0},
  [79] = {.lex_state = 0},
  [80] = {.lex_state = 0},
  [81] = {.lex_state = 0},
  [82] = {.lex_state = 0},
  [83] = {.lex_state = 0},
  [84] = {.lex_state = 0},
  [85] = {.lex_state = 0},
  [86] = {.lex_state = 0},
  [87] = {.lex_state = 0},
  [88] = {.lex_state = 0},
  [89] = {.lex_state = 0},
  [90] = {.lex_state = 0},
  [91] = {.lex_state = 0},
  [92] = {.lex_state = 0},
  [93] = {.lex_state = 0},
  [94] = {.lex_state = 0},
  [95] = {.lex_state = 0},
  [96] = {.lex_state = 0},
  [97] = {.lex_state = 0},
  [98] = {.lex_state = 0},
  [99] = {.lex_state = 0},
  [100] = {.lex_state = 0},
  [101] = {.lex_state = 0},
  [102] = {.lex_state = 0},
  [103] = {.lex_state = 0},
  [104] = {.lex_state = 0},
  [105] = {.lex_state = 0},
  [106] = {.lex_state = 0},
  [107] = {.lex_state = 1},
  [108] = {.lex_state = 0},
  [109] = {.lex_state = 0},
  [110] = {.lex_state = 0},
  [111] = {.lex_state = 0},
  [112] = {.lex_state = 0},
  [113] = {.lex_state = 1},
  [114] = {.lex_state = 0},
  [115] = {.lex_state = 0},
  [116] = {.lex_state = 0},
  [117] = {.lex_state = 0},
  [118] = {.lex_state = 0},
  [119] = {.lex_state = 0},
  [120] = {.lex_state = 0},
  [121] = {.lex_state = 0},
  [122] = {.lex_state = 0},
  [123] = {.lex_state = 0},
  [124] = {.lex_state = 0},
  [125] = {.lex_state = 0},
  [126] = {.lex_state = 0},
  [127] = {.lex_state = 0},
  [128] = {.lex_state = 0},
  [129] = {.lex_state = 0},
  [130] = {.lex_state = 0},
  [131] = {.lex_state = 313},
  [132] = {.lex_state = 313},
  [133] = {.lex_state = 313},
  [134] = {.lex_state = 0},
  [135] = {.lex_state = 313},
  [136] = {.lex_state = 313},
  [137] = {.lex_state = 313},
  [138] = {.lex_state = 0},
  [139] = {.lex_state = 0},
  [140] = {.lex_state = 313},
  [141] = {.lex_state = 0},
  [142] = {.lex_state = 0},
  [143] = {.lex_state = 0},
  [144] = {.lex_state = 313},
  [145] = {.lex_state = 0},
  [146] = {.lex_state = 0},
  [147] = {.lex_state = 13},
  [148] = {.lex_state = 0},
  [149] = {.lex_state = 0},
  [150] = {.lex_state = 0},
  [151] = {.lex_state = 0},
  [152] = {.lex_state = 0},
  [153] = {.lex_state = 0},
  [154] = {.lex_state = 0},
  [155] = {.lex_state = 0},
  [156] = {.lex_state = 0},
  [157] = {.lex_state = 0},
  [158] = {.lex_state = 0},
  [159] = {.lex_state = 0},
  [160] = {.lex_state = 0},
  [161] = {.lex_state = 0},
  [162] = {.lex_state = 0},
  [163] = {.lex_state = 0},
  [164] = {.lex_state = 0},
  [165] = {.lex_state = 0},
  [166] = {.lex_state = 0},
  [167] = {.lex_state = 0},
  [168] = {.lex_state = 0},
  [169] = {.lex_state = 0},
  [170] = {.lex_state = 0},
  [171] = {.lex_state = 0},
  [172] = {.lex_state = 0},
  [173] = {.lex_state = 0},
  [174] = {.lex_state = 0},
  [175] = {.lex_state = 0},
  [176] = {.lex_state = 0},
  [177] = {.lex_state = 0},
  [178] = {.lex_state = 0},
  [179] = {.lex_state = 13},
  [180] = {.lex_state = 0},
  [181] = {.lex_state = 0},
  [182] = {.lex_state = 0},
  [183] = {.lex_state = 0},
  [184] = {.lex_state = 0},
  [185] = {.lex_state = 17},
  [186] = {.lex_state = 17},
  [187] = {.lex_state = 0},
  [188] = {.lex_state = 17},
  [189] = {.lex_state = 0},
  [190] = {.lex_state = 0},
  [191] = {.lex_state = 17},
  [192] = {.lex_state = 17},
  [193] = {.lex_state = 17},
  [194] = {.lex_state = 17},
  [195] = {.lex_state = 0},
  [196] = {.lex_state = 17},
  [197] = {.lex_state = 17},
  [198] = {.lex_state = 0},
  [199] = {.lex_state = 0},
  [200] = {.lex_state = 0},
  [201] = {.lex_state = 0},
  [202] = {.lex_state = 0},
  [203] = {.lex_state = 0},
  [204] = {.lex_state = 17},
  [205] = {.lex_state = 0},
  [206] = {.lex_state = 0},
  [207] = {.lex_state = 17},
  [208] = {.lex_state = 17},
  [209] = {.lex_state = 0},
  [210] = {.lex_state = 17},
  [211] = {.lex_state = 0},
  [212] = {.lex_state = 0},
  [213] = {.lex_state = 17},
  [214] = {.lex_state = 0},
  [215] = {.lex_state = 17},
  [216] = {.lex_state = 13},
  [217] = {.lex_state = 0},
  [218] = {.lex_state = 17},
  [219] = {.lex_state = 17},
  [220] = {.lex_state = 17},
  [221] = {.lex_state = 17},
  [222] = {.lex_state = 17},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [sym_comment] = ACTIONS(3),
    [sym_number] = ACTIONS(1),
    [aux_sym_string_token1] = ACTIONS(1),
    [aux_sym_string_token2] = ACTIONS(1),
    [aux_sym_string_token3] = ACTIONS(1),
    [aux_sym_string_token4] = ACTIONS(1),
    [aux_sym_string_token5] = ACTIONS(1),
    [anon_sym_true] = ACTIONS(1),
    [anon_sym_false] = ACTIONS(1),
    [anon_sym_let] = ACTIONS(1),
    [anon_sym_set] = ACTIONS(1),
    [anon_sym_EQ] = ACTIONS(1),
    [anon_sym_LPAREN] = ACTIONS(1),
    [anon_sym_COMMA] = ACTIONS(1),
    [anon_sym_RPAREN] = ACTIONS(1),
    [anon_sym_trigger] = ACTIONS(1),
    [anon_sym_only] = ACTIONS(1),
    [anon_sym_once] = ACTIONS(1),
    [anon_sym_when] = ACTIONS(1),
    [anon_sym_LBRACE] = ACTIONS(1),
    [anon_sym_RBRACE] = ACTIONS(1),
    [anon_sym_if] = ACTIONS(1),
    [anon_sym_do] = ACTIONS(1),
    [anon_sym_room] = ACTIONS(1),
    [anon_sym_name] = ACTIONS(1),
    [anon_sym_desc] = ACTIONS(1),
    [anon_sym_description] = ACTIONS(1),
    [anon_sym_visited] = ACTIONS(1),
    [anon_sym_overlay] = ACTIONS(1),
    [anon_sym_unset] = ACTIONS(1),
    [anon_sym_text] = ACTIONS(1),
    [anon_sym_normal] = ACTIONS(1),
    [anon_sym_happy] = ACTIONS(1),
    [anon_sym_bored] = ACTIONS(1),
    [anon_sym_exit] = ACTIONS(1),
    [anon_sym_DASH_GT] = ACTIONS(1),
    [anon_sym_required_flags] = ACTIONS(1),
    [anon_sym_required_items] = ACTIONS(1),
    [anon_sym_barred] = ACTIONS(1),
    [anon_sym_item] = ACTIONS(1),
    [anon_sym_portable] = ACTIONS(1),
    [anon_sym_ability] = ACTIONS(1),
    [anon_sym_container] = ACTIONS(1),
    [anon_sym_state] = ACTIONS(1),
    [anon_sym_open] = ACTIONS(1),
    [anon_sym_closed] = ACTIONS(1),
    [anon_sym_restricted] = ACTIONS(1),
    [anon_sym_spinner] = ACTIONS(1),
    [anon_sym_wedge] = ACTIONS(1),
    [anon_sym_width] = ACTIONS(1),
    [anon_sym_npc] = ACTIONS(1),
    [anon_sym_mad] = ACTIONS(1),
    [anon_sym_custom] = ACTIONS(1),
    [anon_sym_movement] = ACTIONS(1),
    [anon_sym_random] = ACTIONS(1),
    [anon_sym_route] = ACTIONS(1),
    [anon_sym_timing] = ACTIONS(1),
    [anon_sym_active] = ACTIONS(1),
    [anon_sym_dialogue] = ACTIONS(1),
    [anon_sym_location] = ACTIONS(1),
    [anon_sym_chest] = ACTIONS(1),
    [anon_sym_inventory] = ACTIONS(1),
    [anon_sym_player] = ACTIONS(1),
    [anon_sym_nowhere] = ACTIONS(1),
    [anon_sym_goal] = ACTIONS(1),
    [anon_sym_group] = ACTIONS(1),
    [anon_sym_required] = ACTIONS(1),
    [anon_sym_optional] = ACTIONS(1),
    [anon_sym_status_DASHeffect] = ACTIONS(1),
    [anon_sym_start] = ACTIONS(1),
    [anon_sym_has] = ACTIONS(1),
    [anon_sym_flag] = ACTIONS(1),
    [anon_sym_missing] = ACTIONS(1),
    [anon_sym_reached] = ACTIONS(1),
    [anon_sym_complete] = ACTIONS(1),
    [anon_sym_in] = ACTIONS(1),
    [anon_sym_progress] = ACTIONS(1),
  },
  [1] = {
    [sym_program] = STATE(217),
    [sym_set_decl] = STATE(10),
    [sym_trigger] = STATE(10),
    [sym_room_def] = STATE(10),
    [sym_item_def] = STATE(10),
    [sym_spinner_def] = STATE(10),
    [sym_npc_def] = STATE(10),
    [sym_goal_def] = STATE(10),
    [aux_sym_program_repeat1] = STATE(10),
    [ts_builtin_sym_end] = ACTIONS(5),
    [sym_comment] = ACTIONS(3),
    [anon_sym_let] = ACTIONS(7),
    [anon_sym_trigger] = ACTIONS(9),
    [anon_sym_room] = ACTIONS(11),
    [anon_sym_item] = ACTIONS(13),
    [anon_sym_spinner] = ACTIONS(15),
    [anon_sym_npc] = ACTIONS(17),
    [anon_sym_goal] = ACTIONS(19),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(23), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(21), 32,
      ts_builtin_sym_end,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
      anon_sym_let,
      anon_sym_set,
      anon_sym_COMMA,
      anon_sym_trigger,
      anon_sym_only,
      anon_sym_when,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_room,
      anon_sym_desc,
      anon_sym_unset,
      anon_sym_text,
      anon_sym_normal,
      anon_sym_happy,
      anon_sym_bored,
      anon_sym_DASH_GT,
      anon_sym_required_flags,
      anon_sym_required_items,
      anon_sym_barred,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_wedge,
      anon_sym_width,
      anon_sym_npc,
      anon_sym_goal,
      anon_sym_group,
      anon_sym_done,
      anon_sym_start,
  [42] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(25), 1,
      anon_sym_RBRACE,
    ACTIONS(27), 1,
      anon_sym_name,
    ACTIONS(29), 1,
      anon_sym_desc,
    ACTIONS(31), 1,
      anon_sym_description,
    ACTIONS(33), 1,
      anon_sym_text,
    ACTIONS(35), 1,
      anon_sym_portable,
    ACTIONS(37), 1,
      anon_sym_ability,
    ACTIONS(39), 1,
      anon_sym_container,
    ACTIONS(41), 1,
      anon_sym_restricted,
    ACTIONS(43), 1,
      anon_sym_location,
    STATE(48), 1,
      sym_location,
    STATE(5), 2,
      sym_item_stmt,
      aux_sym_item_block_repeat1,
    STATE(49), 8,
      sym_item_name,
      sym_item_desc,
      sym_item_portable,
      sym_item_text,
      sym_item_location,
      sym_item_ability,
      sym_item_container_state,
      sym_item_restricted,
  [93] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(45), 1,
      anon_sym_RBRACE,
    ACTIONS(47), 1,
      anon_sym_name,
    ACTIONS(50), 1,
      anon_sym_desc,
    ACTIONS(53), 1,
      anon_sym_description,
    ACTIONS(56), 1,
      anon_sym_text,
    ACTIONS(59), 1,
      anon_sym_portable,
    ACTIONS(62), 1,
      anon_sym_ability,
    ACTIONS(65), 1,
      anon_sym_container,
    ACTIONS(68), 1,
      anon_sym_restricted,
    ACTIONS(71), 1,
      anon_sym_location,
    STATE(48), 1,
      sym_location,
    STATE(4), 2,
      sym_item_stmt,
      aux_sym_item_block_repeat1,
    STATE(49), 8,
      sym_item_name,
      sym_item_desc,
      sym_item_portable,
      sym_item_text,
      sym_item_location,
      sym_item_ability,
      sym_item_container_state,
      sym_item_restricted,
  [144] = 14,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(27), 1,
      anon_sym_name,
    ACTIONS(29), 1,
      anon_sym_desc,
    ACTIONS(31), 1,
      anon_sym_description,
    ACTIONS(33), 1,
      anon_sym_text,
    ACTIONS(35), 1,
      anon_sym_portable,
    ACTIONS(37), 1,
      anon_sym_ability,
    ACTIONS(39), 1,
      anon_sym_container,
    ACTIONS(41), 1,
      anon_sym_restricted,
    ACTIONS(43), 1,
      anon_sym_location,
    ACTIONS(74), 1,
      anon_sym_RBRACE,
    STATE(48), 1,
      sym_location,
    STATE(4), 2,
      sym_item_stmt,
      aux_sym_item_block_repeat1,
    STATE(49), 8,
      sym_item_name,
      sym_item_desc,
      sym_item_portable,
      sym_item_text,
      sym_item_location,
      sym_item_ability,
      sym_item_container_state,
      sym_item_restricted,
  [195] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(78), 1,
      anon_sym_desc,
    ACTIONS(80), 1,
      anon_sym_group,
    ACTIONS(82), 1,
      anon_sym_done,
    ACTIONS(84), 1,
      anon_sym_start,
    STATE(8), 2,
      sym_goal_stmt,
      aux_sym_goal_def_repeat1,
    STATE(33), 4,
      sym_goal_desc,
      sym_goal_group,
      sym_goal_done,
      sym_goal_start,
    ACTIONS(76), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [231] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(78), 1,
      anon_sym_desc,
    ACTIONS(80), 1,
      anon_sym_group,
    ACTIONS(82), 1,
      anon_sym_done,
    ACTIONS(84), 1,
      anon_sym_start,
    STATE(6), 2,
      sym_goal_stmt,
      aux_sym_goal_def_repeat1,
    STATE(33), 4,
      sym_goal_desc,
      sym_goal_group,
      sym_goal_done,
      sym_goal_start,
    ACTIONS(86), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [267] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(90), 1,
      anon_sym_desc,
    ACTIONS(93), 1,
      anon_sym_group,
    ACTIONS(96), 1,
      anon_sym_done,
    ACTIONS(99), 1,
      anon_sym_start,
    STATE(8), 2,
      sym_goal_stmt,
      aux_sym_goal_def_repeat1,
    STATE(33), 4,
      sym_goal_desc,
      sym_goal_group,
      sym_goal_done,
      sym_goal_start,
    ACTIONS(88), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [303] = 11,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(43), 1,
      anon_sym_location,
    ACTIONS(102), 1,
      anon_sym_RBRACE,
    ACTIONS(104), 1,
      anon_sym_name,
    ACTIONS(106), 1,
      anon_sym_desc,
    ACTIONS(108), 1,
      anon_sym_description,
    ACTIONS(110), 1,
      anon_sym_state,
    ACTIONS(112), 1,
      anon_sym_movement,
    ACTIONS(114), 1,
      anon_sym_dialogue,
    STATE(13), 2,
      sym_npc_stmt,
      aux_sym_npc_block_repeat1,
    STATE(74), 6,
      sym_npc_name,
      sym_npc_desc,
      sym_npc_state,
      sym_movement_stmt,
      sym_dialogue_stmt,
      sym_location,
  [343] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      anon_sym_let,
    ACTIONS(9), 1,
      anon_sym_trigger,
    ACTIONS(11), 1,
      anon_sym_room,
    ACTIONS(13), 1,
      anon_sym_item,
    ACTIONS(15), 1,
      anon_sym_spinner,
    ACTIONS(17), 1,
      anon_sym_npc,
    ACTIONS(19), 1,
      anon_sym_goal,
    ACTIONS(116), 1,
      ts_builtin_sym_end,
    STATE(14), 8,
      sym_set_decl,
      sym_trigger,
      sym_room_def,
      sym_item_def,
      sym_spinner_def,
      sym_npc_def,
      sym_goal_def,
      aux_sym_program_repeat1,
  [381] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(23), 1,
      anon_sym_desc,
    ACTIONS(21), 15,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_overlay,
      anon_sym_text,
      anon_sym_exit,
      anon_sym_portable,
      anon_sym_ability,
      anon_sym_container,
      anon_sym_state,
      anon_sym_restricted,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [405] = 11,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(118), 1,
      anon_sym_RBRACE,
    ACTIONS(120), 1,
      anon_sym_name,
    ACTIONS(123), 1,
      anon_sym_desc,
    ACTIONS(126), 1,
      anon_sym_description,
    ACTIONS(129), 1,
      anon_sym_state,
    ACTIONS(132), 1,
      anon_sym_movement,
    ACTIONS(135), 1,
      anon_sym_dialogue,
    ACTIONS(138), 1,
      anon_sym_location,
    STATE(12), 2,
      sym_npc_stmt,
      aux_sym_npc_block_repeat1,
    STATE(74), 6,
      sym_npc_name,
      sym_npc_desc,
      sym_npc_state,
      sym_movement_stmt,
      sym_dialogue_stmt,
      sym_location,
  [445] = 11,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(43), 1,
      anon_sym_location,
    ACTIONS(104), 1,
      anon_sym_name,
    ACTIONS(106), 1,
      anon_sym_desc,
    ACTIONS(108), 1,
      anon_sym_description,
    ACTIONS(110), 1,
      anon_sym_state,
    ACTIONS(112), 1,
      anon_sym_movement,
    ACTIONS(114), 1,
      anon_sym_dialogue,
    ACTIONS(141), 1,
      anon_sym_RBRACE,
    STATE(12), 2,
      sym_npc_stmt,
      aux_sym_npc_block_repeat1,
    STATE(74), 6,
      sym_npc_name,
      sym_npc_desc,
      sym_npc_state,
      sym_movement_stmt,
      sym_dialogue_stmt,
      sym_location,
  [485] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(143), 1,
      ts_builtin_sym_end,
    ACTIONS(145), 1,
      anon_sym_let,
    ACTIONS(148), 1,
      anon_sym_trigger,
    ACTIONS(151), 1,
      anon_sym_room,
    ACTIONS(154), 1,
      anon_sym_item,
    ACTIONS(157), 1,
      anon_sym_spinner,
    ACTIONS(160), 1,
      anon_sym_npc,
    ACTIONS(163), 1,
      anon_sym_goal,
    STATE(14), 8,
      sym_set_decl,
      sym_trigger,
      sym_room_def,
      sym_item_def,
      sym_spinner_def,
      sym_npc_def,
      sym_goal_def,
      aux_sym_program_repeat1,
  [523] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(168), 1,
      anon_sym_desc,
    ACTIONS(166), 15,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_overlay,
      anon_sym_text,
      anon_sym_exit,
      anon_sym_portable,
      anon_sym_ability,
      anon_sym_container,
      anon_sym_state,
      anon_sym_restricted,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [547] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(170), 1,
      anon_sym_RBRACE,
    ACTIONS(172), 1,
      anon_sym_name,
    ACTIONS(174), 1,
      anon_sym_desc,
    ACTIONS(176), 1,
      anon_sym_description,
    ACTIONS(178), 1,
      anon_sym_visited,
    ACTIONS(180), 1,
      anon_sym_overlay,
    ACTIONS(182), 1,
      anon_sym_exit,
    STATE(19), 2,
      sym_room_stmt,
      aux_sym_room_block_repeat1,
    STATE(105), 5,
      sym_room_name,
      sym_room_desc,
      sym_room_visited,
      sym_overlay_stmt,
      sym_exit_stmt,
  [583] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(172), 1,
      anon_sym_name,
    ACTIONS(174), 1,
      anon_sym_desc,
    ACTIONS(176), 1,
      anon_sym_description,
    ACTIONS(178), 1,
      anon_sym_visited,
    ACTIONS(180), 1,
      anon_sym_overlay,
    ACTIONS(182), 1,
      anon_sym_exit,
    ACTIONS(184), 1,
      anon_sym_RBRACE,
    STATE(16), 2,
      sym_room_stmt,
      aux_sym_room_block_repeat1,
    STATE(105), 5,
      sym_room_name,
      sym_room_desc,
      sym_room_visited,
      sym_overlay_stmt,
      sym_exit_stmt,
  [619] = 9,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(192), 1,
      anon_sym_LBRACE,
    ACTIONS(194), 1,
      anon_sym_RBRACE,
    STATE(145), 1,
      sym_braced_block,
    ACTIONS(186), 2,
      sym_identifier,
      sym_number,
    ACTIONS(188), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(196), 2,
      anon_sym_if,
      anon_sym_do,
    STATE(24), 2,
      sym_string,
      aux_sym_do_stmt_repeat1,
    ACTIONS(190), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [653] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(198), 1,
      anon_sym_RBRACE,
    ACTIONS(200), 1,
      anon_sym_name,
    ACTIONS(203), 1,
      anon_sym_desc,
    ACTIONS(206), 1,
      anon_sym_description,
    ACTIONS(209), 1,
      anon_sym_visited,
    ACTIONS(212), 1,
      anon_sym_overlay,
    ACTIONS(215), 1,
      anon_sym_exit,
    STATE(19), 2,
      sym_room_stmt,
      aux_sym_room_block_repeat1,
    STATE(105), 5,
      sym_room_name,
      sym_room_desc,
      sym_room_visited,
      sym_overlay_stmt,
      sym_exit_stmt,
  [689] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(220), 1,
      anon_sym_desc,
    ACTIONS(218), 12,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_text,
      anon_sym_portable,
      anon_sym_ability,
      anon_sym_container,
      anon_sym_state,
      anon_sym_restricted,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [710] = 7,
    ACTIONS(3), 1,
      sym_comment,
    STATE(171), 1,
      sym_cond_line_ext,
    ACTIONS(222), 2,
      sym_identifier,
      sym_number,
    ACTIONS(224), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    STATE(23), 2,
      sym_string,
      aux_sym_cond_line_ext_repeat1,
    ACTIONS(226), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
    ACTIONS(228), 3,
      anon_sym_LPAREN,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [739] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(242), 1,
      anon_sym_LBRACE,
    ACTIONS(230), 2,
      sym_identifier,
      sym_number,
    ACTIONS(233), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    STATE(22), 2,
      sym_string,
      aux_sym_cond_line_ext_repeat1,
    ACTIONS(236), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
    ACTIONS(239), 3,
      anon_sym_LPAREN,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [768] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(248), 1,
      anon_sym_LBRACE,
    ACTIONS(224), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(244), 2,
      sym_identifier,
      sym_number,
    STATE(22), 2,
      sym_string,
      aux_sym_cond_line_ext_repeat1,
    ACTIONS(226), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
    ACTIONS(246), 3,
      anon_sym_LPAREN,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [797] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(250), 2,
      sym_identifier,
      sym_number,
    ACTIONS(253), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(259), 2,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
    ACTIONS(261), 2,
      anon_sym_if,
      anon_sym_do,
    STATE(24), 2,
      sym_string,
      aux_sym_do_stmt_repeat1,
    ACTIONS(256), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [826] = 7,
    ACTIONS(3), 1,
      sym_comment,
    STATE(169), 1,
      sym_cond_line_ext,
    ACTIONS(222), 2,
      sym_identifier,
      sym_number,
    ACTIONS(224), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    STATE(23), 2,
      sym_string,
      aux_sym_cond_line_ext_repeat1,
    ACTIONS(226), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
    ACTIONS(228), 3,
      anon_sym_LPAREN,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [855] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(271), 1,
      anon_sym_LBRACE,
    ACTIONS(263), 2,
      sym_identifier,
      sym_number,
    ACTIONS(265), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(269), 2,
      anon_sym_only,
      anon_sym_when,
    STATE(34), 2,
      sym_string,
      aux_sym_do_stmt_repeat1,
    ACTIONS(267), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [883] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(273), 12,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_desc,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
      anon_sym_group,
      anon_sym_done,
      anon_sym_start,
  [901] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(275), 12,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_desc,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
      anon_sym_group,
      anon_sym_done,
      anon_sym_start,
  [919] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(277), 12,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_desc,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
      anon_sym_group,
      anon_sym_done,
      anon_sym_start,
  [937] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(279), 12,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_desc,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
      anon_sym_group,
      anon_sym_done,
      anon_sym_start,
  [955] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(281), 12,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_desc,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
      anon_sym_group,
      anon_sym_done,
      anon_sym_start,
  [973] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(283), 12,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_desc,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
      anon_sym_group,
      anon_sym_done,
      anon_sym_start,
  [991] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(285), 12,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_desc,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
      anon_sym_group,
      anon_sym_done,
      anon_sym_start,
  [1009] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(259), 1,
      anon_sym_LBRACE,
    ACTIONS(261), 2,
      anon_sym_only,
      anon_sym_when,
    ACTIONS(287), 2,
      sym_identifier,
      sym_number,
    ACTIONS(290), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    STATE(34), 2,
      sym_string,
      aux_sym_do_stmt_repeat1,
    ACTIONS(293), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [1037] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(21), 5,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
    ACTIONS(23), 6,
      sym_identifier,
      sym_number,
      aux_sym_string_token1,
      aux_sym_string_token2,
      anon_sym_if,
      anon_sym_do,
  [1056] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(23), 4,
      sym_identifier,
      sym_number,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(21), 7,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
      anon_sym_LPAREN,
      anon_sym_COMMA,
      anon_sym_RPAREN,
      anon_sym_LBRACE,
  [1075] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(298), 1,
      anon_sym_desc,
    ACTIONS(296), 9,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_text,
      anon_sym_portable,
      anon_sym_ability,
      anon_sym_container,
      anon_sym_restricted,
      anon_sym_location,
  [1093] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(302), 1,
      anon_sym_desc,
    ACTIONS(300), 9,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_text,
      anon_sym_portable,
      anon_sym_ability,
      anon_sym_container,
      anon_sym_restricted,
      anon_sym_location,
  [1111] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(306), 1,
      anon_sym_desc,
    ACTIONS(308), 1,
      anon_sym_timing,
    ACTIONS(310), 1,
      anon_sym_active,
    ACTIONS(304), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [1133] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(314), 1,
      anon_sym_desc,
    ACTIONS(312), 9,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_text,
      anon_sym_portable,
      anon_sym_ability,
      anon_sym_container,
      anon_sym_restricted,
      anon_sym_location,
  [1151] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(318), 1,
      anon_sym_desc,
    ACTIONS(316), 9,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_text,
      anon_sym_portable,
      anon_sym_ability,
      anon_sym_container,
      anon_sym_restricted,
      anon_sym_location,
  [1169] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(322), 1,
      anon_sym_desc,
    ACTIONS(320), 9,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_text,
      anon_sym_portable,
      anon_sym_ability,
      anon_sym_container,
      anon_sym_restricted,
      anon_sym_location,
  [1187] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(326), 1,
      anon_sym_desc,
    ACTIONS(324), 9,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_text,
      anon_sym_portable,
      anon_sym_ability,
      anon_sym_container,
      anon_sym_restricted,
      anon_sym_location,
  [1205] = 6,
    ACTIONS(3), 1,
      sym_comment,
    STATE(152), 1,
      sym_cond_line,
    ACTIONS(265), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(328), 2,
      sym_identifier,
      sym_number,
    STATE(26), 2,
      sym_string,
      aux_sym_do_stmt_repeat1,
    ACTIONS(267), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [1229] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(332), 1,
      anon_sym_desc,
    ACTIONS(330), 9,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_text,
      anon_sym_portable,
      anon_sym_ability,
      anon_sym_container,
      anon_sym_restricted,
      anon_sym_location,
  [1247] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(21), 4,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
      anon_sym_LBRACE,
    ACTIONS(23), 6,
      sym_identifier,
      sym_number,
      aux_sym_string_token1,
      aux_sym_string_token2,
      anon_sym_only,
      anon_sym_when,
  [1265] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(336), 1,
      anon_sym_desc,
    ACTIONS(338), 1,
      anon_sym_timing,
    ACTIONS(340), 1,
      anon_sym_active,
    ACTIONS(334), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [1287] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(344), 1,
      anon_sym_desc,
    ACTIONS(342), 9,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_text,
      anon_sym_portable,
      anon_sym_ability,
      anon_sym_container,
      anon_sym_restricted,
      anon_sym_location,
  [1305] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(348), 1,
      anon_sym_desc,
    ACTIONS(346), 9,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_text,
      anon_sym_portable,
      anon_sym_ability,
      anon_sym_container,
      anon_sym_restricted,
      anon_sym_location,
  [1323] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(350), 1,
      anon_sym_RBRACE,
    ACTIONS(352), 1,
      anon_sym_required_flags,
    ACTIONS(354), 1,
      anon_sym_required_items,
    ACTIONS(356), 1,
      anon_sym_barred,
    STATE(55), 2,
      sym_exit_attr,
      aux_sym_exit_block_repeat1,
    STATE(140), 3,
      sym_exit_required_flags,
      sym_exit_required_items,
      sym_exit_barred,
  [1348] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(360), 1,
      anon_sym_RBRACE,
    STATE(57), 2,
      sym_overlay_entry,
      aux_sym_overlay_block_repeat1,
    ACTIONS(358), 6,
      anon_sym_set,
      anon_sym_unset,
      anon_sym_text,
      anon_sym_normal,
      anon_sym_happy,
      anon_sym_bored,
  [1367] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(364), 1,
      anon_sym_desc,
    ACTIONS(366), 1,
      anon_sym_active,
    ACTIONS(362), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [1386] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(370), 1,
      anon_sym_desc,
    ACTIONS(372), 1,
      anon_sym_active,
    ACTIONS(368), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [1405] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(374), 1,
      anon_sym_RBRACE,
    ACTIONS(376), 1,
      anon_sym_required_flags,
    ACTIONS(379), 1,
      anon_sym_required_items,
    ACTIONS(382), 1,
      anon_sym_barred,
    STATE(54), 2,
      sym_exit_attr,
      aux_sym_exit_block_repeat1,
    STATE(140), 3,
      sym_exit_required_flags,
      sym_exit_required_items,
      sym_exit_barred,
  [1430] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(352), 1,
      anon_sym_required_flags,
    ACTIONS(354), 1,
      anon_sym_required_items,
    ACTIONS(356), 1,
      anon_sym_barred,
    ACTIONS(385), 1,
      anon_sym_RBRACE,
    STATE(54), 2,
      sym_exit_attr,
      aux_sym_exit_block_repeat1,
    STATE(140), 3,
      sym_exit_required_flags,
      sym_exit_required_items,
      sym_exit_barred,
  [1455] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(390), 1,
      anon_sym_RBRACE,
    STATE(56), 2,
      sym_overlay_entry,
      aux_sym_overlay_block_repeat1,
    ACTIONS(387), 6,
      anon_sym_set,
      anon_sym_unset,
      anon_sym_text,
      anon_sym_normal,
      anon_sym_happy,
      anon_sym_bored,
  [1474] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(392), 1,
      anon_sym_RBRACE,
    STATE(56), 2,
      sym_overlay_entry,
      aux_sym_overlay_block_repeat1,
    ACTIONS(358), 6,
      anon_sym_set,
      anon_sym_unset,
      anon_sym_text,
      anon_sym_normal,
      anon_sym_happy,
      anon_sym_bored,
  [1493] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(188), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(394), 2,
      sym_identifier,
      sym_number,
    STATE(18), 2,
      sym_string,
      aux_sym_do_stmt_repeat1,
    ACTIONS(190), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [1514] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(396), 1,
      anon_sym_LBRACE,
    ACTIONS(400), 1,
      anon_sym_desc,
    STATE(102), 1,
      sym_exit_block,
    ACTIONS(398), 6,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_overlay,
      anon_sym_exit,
  [1535] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(402), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [1549] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(404), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [1563] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(410), 1,
      anon_sym_RBRACE,
    ACTIONS(406), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    STATE(66), 2,
      sym_string,
      aux_sym_dialogue_stmt_repeat1,
    ACTIONS(408), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [1583] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(410), 1,
      anon_sym_RBRACE,
    ACTIONS(406), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    STATE(65), 2,
      sym_string,
      aux_sym_dialogue_stmt_repeat1,
    ACTIONS(408), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [1603] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(414), 1,
      anon_sym_desc,
    ACTIONS(412), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [1619] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(422), 1,
      anon_sym_RBRACE,
    ACTIONS(416), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    STATE(65), 2,
      sym_string,
      aux_sym_dialogue_stmt_repeat1,
    ACTIONS(419), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [1639] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(424), 1,
      anon_sym_RBRACE,
    ACTIONS(406), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    STATE(65), 2,
      sym_string,
      aux_sym_dialogue_stmt_repeat1,
    ACTIONS(408), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [1659] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(426), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [1673] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(430), 1,
      anon_sym_desc,
    ACTIONS(428), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [1689] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(432), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [1703] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(436), 1,
      anon_sym_desc,
    ACTIONS(434), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [1719] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(438), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [1733] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(440), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [1747] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(442), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [1761] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(446), 1,
      anon_sym_desc,
    ACTIONS(444), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [1777] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(448), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [1791] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(450), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [1805] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(452), 1,
      anon_sym_RBRACE,
    ACTIONS(406), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    STATE(63), 2,
      sym_string,
      aux_sym_dialogue_stmt_repeat1,
    ACTIONS(408), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [1825] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(456), 1,
      anon_sym_desc,
    ACTIONS(454), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [1841] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(460), 1,
      anon_sym_desc,
    ACTIONS(458), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [1857] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(462), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [1871] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(466), 1,
      anon_sym_desc,
    ACTIONS(464), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [1887] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(470), 1,
      anon_sym_desc,
    ACTIONS(468), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [1903] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(472), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [1917] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(364), 1,
      anon_sym_desc,
    ACTIONS(362), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [1933] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(370), 1,
      anon_sym_desc,
    ACTIONS(368), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [1949] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(474), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [1963] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(476), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [1977] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(478), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [1991] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(480), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [2005] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(484), 1,
      anon_sym_desc,
    ACTIONS(482), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [2021] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(486), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [2035] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(490), 1,
      anon_sym_desc,
    ACTIONS(488), 7,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_state,
      anon_sym_movement,
      anon_sym_dialogue,
      anon_sym_location,
  [2051] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(492), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [2065] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(494), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [2079] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(496), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [2093] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(498), 1,
      anon_sym_RBRACE,
    ACTIONS(500), 1,
      anon_sym_if,
    ACTIONS(502), 1,
      anon_sym_do,
    STATE(111), 2,
      sym_trigger_stmt,
      aux_sym_trigger_block_repeat1,
    STATE(153), 2,
      sym_if_block,
      sym_do_stmt,
  [2114] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(506), 1,
      anon_sym_desc,
    ACTIONS(504), 6,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_overlay,
      anon_sym_exit,
  [2129] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(510), 1,
      anon_sym_desc,
    ACTIONS(508), 6,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_overlay,
      anon_sym_exit,
  [2144] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(500), 1,
      anon_sym_if,
    ACTIONS(502), 1,
      anon_sym_do,
    ACTIONS(512), 1,
      anon_sym_RBRACE,
    STATE(109), 2,
      sym_trigger_stmt,
      aux_sym_trigger_block_repeat1,
    STATE(153), 2,
      sym_if_block,
      sym_do_stmt,
  [2165] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(516), 1,
      anon_sym_desc,
    ACTIONS(514), 6,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_overlay,
      anon_sym_exit,
  [2180] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(520), 1,
      anon_sym_desc,
    ACTIONS(518), 6,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_overlay,
      anon_sym_exit,
  [2195] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(524), 1,
      anon_sym_desc,
    ACTIONS(522), 6,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_overlay,
      anon_sym_exit,
  [2210] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(526), 1,
      anon_sym_RBRACE,
    ACTIONS(528), 1,
      anon_sym_if,
    ACTIONS(531), 1,
      anon_sym_do,
    STATE(103), 2,
      sym_trigger_stmt,
      aux_sym_trigger_block_repeat1,
    STATE(153), 2,
      sym_if_block,
      sym_do_stmt,
  [2231] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(536), 1,
      anon_sym_desc,
    ACTIONS(534), 6,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_overlay,
      anon_sym_exit,
  [2246] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(540), 1,
      anon_sym_desc,
    ACTIONS(538), 6,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_overlay,
      anon_sym_exit,
  [2261] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(544), 1,
      anon_sym_desc,
    ACTIONS(542), 6,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_overlay,
      anon_sym_exit,
  [2276] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(546), 1,
      sym_identifier,
    STATE(209), 1,
      sym_string,
    ACTIONS(406), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(408), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [2295] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(550), 1,
      anon_sym_desc,
    ACTIONS(548), 6,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_overlay,
      anon_sym_exit,
  [2310] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(500), 1,
      anon_sym_if,
    ACTIONS(502), 1,
      anon_sym_do,
    ACTIONS(552), 1,
      anon_sym_RBRACE,
    STATE(103), 2,
      sym_trigger_stmt,
      aux_sym_trigger_block_repeat1,
    STATE(153), 2,
      sym_if_block,
      sym_do_stmt,
  [2331] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(554), 7,
      anon_sym_set,
      anon_sym_RBRACE,
      anon_sym_unset,
      anon_sym_text,
      anon_sym_normal,
      anon_sym_happy,
      anon_sym_bored,
  [2344] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(500), 1,
      anon_sym_if,
    ACTIONS(502), 1,
      anon_sym_do,
    ACTIONS(556), 1,
      anon_sym_RBRACE,
    STATE(103), 2,
      sym_trigger_stmt,
      aux_sym_trigger_block_repeat1,
    STATE(153), 2,
      sym_if_block,
      sym_do_stmt,
  [2365] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(560), 1,
      anon_sym_desc,
    ACTIONS(558), 6,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_overlay,
      anon_sym_exit,
  [2380] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(562), 1,
      sym_identifier,
    STATE(119), 1,
      sym_string,
    ACTIONS(406), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(408), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [2399] = 4,
    ACTIONS(3), 1,
      sym_comment,
    STATE(43), 1,
      sym_string,
    ACTIONS(564), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(566), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [2415] = 4,
    ACTIONS(3), 1,
      sym_comment,
    STATE(149), 1,
      sym_string,
    ACTIONS(406), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(408), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [2431] = 4,
    ACTIONS(3), 1,
      sym_comment,
    STATE(30), 1,
      sym_string,
    ACTIONS(406), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(408), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [2447] = 4,
    ACTIONS(3), 1,
      sym_comment,
    STATE(64), 1,
      sym_string,
    ACTIONS(564), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(566), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [2463] = 4,
    ACTIONS(3), 1,
      sym_comment,
    STATE(70), 1,
      sym_string,
    ACTIONS(564), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(566), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [2479] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(568), 1,
      anon_sym_only,
    ACTIONS(570), 1,
      anon_sym_when,
    ACTIONS(572), 1,
      anon_sym_LBRACE,
    STATE(91), 1,
      sym_trigger_block,
    STATE(129), 2,
      sym_trigger_mod,
      aux_sym_trigger_repeat1,
  [2499] = 4,
    ACTIONS(3), 1,
      sym_comment,
    STATE(135), 1,
      sym_string,
    ACTIONS(406), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(408), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [2515] = 4,
    ACTIONS(3), 1,
      sym_comment,
    STATE(7), 1,
      sym_string,
    ACTIONS(406), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(408), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [2531] = 4,
    ACTIONS(3), 1,
      sym_comment,
    STATE(20), 1,
      sym_string,
    ACTIONS(564), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(566), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [2547] = 4,
    ACTIONS(3), 1,
      sym_comment,
    STATE(100), 1,
      sym_string,
    ACTIONS(564), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(566), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [2563] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(574), 1,
      anon_sym_goal,
    ACTIONS(576), 1,
      anon_sym_has,
    ACTIONS(578), 1,
      anon_sym_flag,
    ACTIONS(580), 1,
      anon_sym_missing,
    ACTIONS(582), 1,
      anon_sym_reached,
    STATE(27), 1,
      sym_goal_cond,
  [2585] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(574), 1,
      anon_sym_goal,
    ACTIONS(576), 1,
      anon_sym_has,
    ACTIONS(578), 1,
      anon_sym_flag,
    ACTIONS(580), 1,
      anon_sym_missing,
    ACTIONS(582), 1,
      anon_sym_reached,
    STATE(29), 1,
      sym_goal_cond,
  [2607] = 4,
    ACTIONS(3), 1,
      sym_comment,
    STATE(98), 1,
      sym_string,
    ACTIONS(564), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(566), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [2623] = 4,
    ACTIONS(3), 1,
      sym_comment,
    STATE(45), 1,
      sym_string,
    ACTIONS(564), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(566), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [2639] = 4,
    ACTIONS(3), 1,
      sym_comment,
    STATE(42), 1,
      sym_string,
    ACTIONS(564), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(566), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [2655] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(568), 1,
      anon_sym_only,
    ACTIONS(570), 1,
      anon_sym_when,
    ACTIONS(572), 1,
      anon_sym_LBRACE,
    STATE(76), 1,
      sym_trigger_block,
    STATE(134), 2,
      sym_trigger_mod,
      aux_sym_trigger_repeat1,
  [2675] = 4,
    ACTIONS(3), 1,
      sym_comment,
    STATE(110), 1,
      sym_string,
    ACTIONS(406), 2,
      aux_sym_string_token1,
      aux_sym_string_token2,
    ACTIONS(408), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [2691] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(584), 5,
      anon_sym_COMMA,
      anon_sym_RBRACE,
      anon_sym_required_flags,
      anon_sym_required_items,
      anon_sym_barred,
  [2702] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(588), 1,
      anon_sym_inventory,
    ACTIONS(590), 1,
      anon_sym_nowhere,
    ACTIONS(586), 3,
      anon_sym_room,
      anon_sym_npc,
      anon_sym_chest,
  [2717] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(592), 5,
      anon_sym_COMMA,
      anon_sym_RBRACE,
      anon_sym_required_flags,
      anon_sym_required_items,
      anon_sym_barred,
  [2728] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(594), 1,
      anon_sym_only,
    ACTIONS(597), 1,
      anon_sym_when,
    ACTIONS(600), 1,
      anon_sym_LBRACE,
    STATE(134), 2,
      sym_trigger_mod,
      aux_sym_trigger_repeat1,
  [2745] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(602), 5,
      anon_sym_COMMA,
      anon_sym_RBRACE,
      anon_sym_required_flags,
      anon_sym_required_items,
      anon_sym_barred,
  [2756] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(604), 5,
      anon_sym_COMMA,
      anon_sym_RBRACE,
      anon_sym_required_flags,
      anon_sym_required_items,
      anon_sym_barred,
  [2767] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(606), 5,
      anon_sym_COMMA,
      anon_sym_RBRACE,
      anon_sym_required_flags,
      anon_sym_required_items,
      anon_sym_barred,
  [2778] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(610), 1,
      anon_sym_custom,
    ACTIONS(608), 4,
      anon_sym_normal,
      anon_sym_happy,
      anon_sym_bored,
      anon_sym_mad,
  [2791] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(614), 1,
      anon_sym_custom,
    ACTIONS(612), 4,
      anon_sym_normal,
      anon_sym_happy,
      anon_sym_bored,
      anon_sym_mad,
  [2804] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(616), 1,
      anon_sym_COMMA,
    ACTIONS(618), 4,
      anon_sym_RBRACE,
      anon_sym_required_flags,
      anon_sym_required_items,
      anon_sym_barred,
  [2817] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(620), 1,
      anon_sym_RBRACE,
    ACTIONS(622), 1,
      anon_sym_wedge,
    STATE(143), 2,
      sym_wedge_stmt,
      aux_sym_spinner_block_repeat1,
  [2831] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(624), 1,
      anon_sym_RBRACE,
    ACTIONS(626), 1,
      anon_sym_wedge,
    STATE(142), 2,
      sym_wedge_stmt,
      aux_sym_spinner_block_repeat1,
  [2845] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(622), 1,
      anon_sym_wedge,
    ACTIONS(629), 1,
      anon_sym_RBRACE,
    STATE(142), 2,
      sym_wedge_stmt,
      aux_sym_spinner_block_repeat1,
  [2859] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(631), 4,
      anon_sym_RBRACE,
      anon_sym_required_flags,
      anon_sym_required_items,
      anon_sym_barred,
  [2869] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(633), 3,
      anon_sym_RBRACE,
      anon_sym_if,
      anon_sym_do,
  [2878] = 3,
    ACTIONS(3), 1,
      sym_comment,
    STATE(41), 1,
      sym_boolean,
    ACTIONS(635), 2,
      anon_sym_true,
      anon_sym_false,
  [2889] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(637), 3,
      anon_sym_required,
      anon_sym_optional,
      anon_sym_status_DASHeffect,
  [2898] = 3,
    ACTIONS(3), 1,
      sym_comment,
    STATE(79), 1,
      sym_boolean,
    ACTIONS(635), 2,
      anon_sym_true,
      anon_sym_false,
  [2909] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(641), 1,
      anon_sym_width,
    ACTIONS(639), 2,
      anon_sym_RBRACE,
      anon_sym_wedge,
  [2920] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(643), 1,
      anon_sym_COMMA,
    ACTIONS(645), 1,
      anon_sym_RPAREN,
    STATE(165), 1,
      aux_sym_set_list_repeat1,
  [2933] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(647), 3,
      anon_sym_RBRACE,
      anon_sym_if,
      anon_sym_do,
  [2942] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(649), 3,
      anon_sym_only,
      anon_sym_when,
      anon_sym_LBRACE,
  [2951] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(651), 3,
      anon_sym_RBRACE,
      anon_sym_if,
      anon_sym_do,
  [2960] = 3,
    ACTIONS(3), 1,
      sym_comment,
    STATE(81), 1,
      sym_boolean,
    ACTIONS(635), 2,
      anon_sym_true,
      anon_sym_false,
  [2971] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(643), 1,
      anon_sym_COMMA,
    ACTIONS(653), 1,
      anon_sym_RPAREN,
    STATE(167), 1,
      aux_sym_set_list_repeat1,
  [2984] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(643), 1,
      anon_sym_COMMA,
    ACTIONS(655), 1,
      anon_sym_RPAREN,
    STATE(165), 1,
      aux_sym_set_list_repeat1,
  [2997] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(643), 1,
      anon_sym_COMMA,
    ACTIONS(657), 1,
      anon_sym_RPAREN,
    STATE(156), 1,
      aux_sym_set_list_repeat1,
  [3010] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(643), 1,
      anon_sym_COMMA,
    ACTIONS(659), 1,
      anon_sym_RPAREN,
    STATE(150), 1,
      aux_sym_set_list_repeat1,
  [3023] = 3,
    ACTIONS(3), 1,
      sym_comment,
    STATE(97), 1,
      sym_boolean,
    ACTIONS(635), 2,
      anon_sym_true,
      anon_sym_false,
  [3034] = 3,
    ACTIONS(3), 1,
      sym_comment,
    STATE(84), 1,
      sym_boolean,
    ACTIONS(635), 2,
      anon_sym_true,
      anon_sym_false,
  [3045] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(643), 1,
      anon_sym_COMMA,
    ACTIONS(661), 1,
      anon_sym_RPAREN,
    STATE(162), 1,
      aux_sym_set_list_repeat1,
  [3058] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(643), 1,
      anon_sym_COMMA,
    ACTIONS(663), 1,
      anon_sym_RPAREN,
    STATE(165), 1,
      aux_sym_set_list_repeat1,
  [3071] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(665), 3,
      anon_sym_RBRACE,
      anon_sym_if,
      anon_sym_do,
  [3080] = 3,
    ACTIONS(3), 1,
      sym_comment,
    STATE(85), 1,
      sym_boolean,
    ACTIONS(635), 2,
      anon_sym_true,
      anon_sym_false,
  [3091] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(667), 1,
      anon_sym_COMMA,
    ACTIONS(670), 1,
      anon_sym_RPAREN,
    STATE(165), 1,
      aux_sym_set_list_repeat1,
  [3104] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(672), 3,
      anon_sym_RBRACE,
      anon_sym_if,
      anon_sym_do,
  [3113] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(643), 1,
      anon_sym_COMMA,
    ACTIONS(674), 1,
      anon_sym_RPAREN,
    STATE(165), 1,
      aux_sym_set_list_repeat1,
  [3126] = 3,
    ACTIONS(3), 1,
      sym_comment,
    STATE(37), 1,
      sym_boolean,
    ACTIONS(635), 2,
      anon_sym_true,
      anon_sym_false,
  [3137] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(676), 1,
      anon_sym_LBRACE,
    STATE(112), 1,
      sym_overlay_block,
  [3147] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(670), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [3155] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(192), 1,
      anon_sym_LBRACE,
    STATE(163), 1,
      sym_braced_block,
  [3165] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(678), 2,
      anon_sym_open,
      anon_sym_closed,
  [3173] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(680), 1,
      anon_sym_LBRACE,
    STATE(94), 1,
      sym_item_block,
  [3183] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(682), 1,
      anon_sym_LBRACE,
    STATE(61), 1,
      sym_spinner_block,
  [3193] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(684), 2,
      anon_sym_RBRACE,
      anon_sym_wedge,
  [3201] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(686), 1,
      anon_sym_LBRACE,
    STATE(86), 1,
      sym_npc_block,
  [3211] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(688), 2,
      anon_sym_item,
      anon_sym_flag,
  [3219] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(690), 2,
      anon_sym_random,
      anon_sym_route,
  [3227] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(688), 1,
      anon_sym_complete,
    ACTIONS(692), 1,
      anon_sym_in,
  [3237] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(694), 1,
      anon_sym_LBRACE,
    STATE(93), 1,
      sym_room_block,
  [3247] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(696), 1,
      anon_sym_LPAREN,
    STATE(83), 1,
      sym_set_list,
  [3257] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(698), 1,
      anon_sym_state,
  [3264] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(700), 1,
      anon_sym_LPAREN,
  [3271] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(702), 1,
      anon_sym_if,
  [3278] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(704), 1,
      sym_identifier,
  [3285] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(706), 1,
      sym_identifier,
  [3292] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(708), 1,
      anon_sym_LPAREN,
  [3299] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(710), 1,
      sym_identifier,
  [3306] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(712), 1,
      anon_sym_progress,
  [3313] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(714), 1,
      anon_sym_once,
  [3320] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(716), 1,
      sym_identifier,
  [3327] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(718), 1,
      sym_identifier,
  [3334] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(720), 1,
      sym_identifier,
  [3341] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(722), 1,
      sym_identifier,
  [3348] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(724), 1,
      anon_sym_EQ,
  [3355] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(726), 1,
      sym_identifier,
  [3362] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(728), 1,
      sym_identifier,
  [3369] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(688), 1,
      anon_sym_room,
  [3376] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(688), 1,
      anon_sym_flag,
  [3383] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(688), 1,
      anon_sym_complete,
  [3390] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(730), 1,
      anon_sym_LBRACE,
  [3397] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(732), 1,
      anon_sym_when,
  [3404] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(734), 1,
      anon_sym_LPAREN,
  [3411] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(736), 1,
      sym_identifier,
  [3418] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(738), 1,
      anon_sym_when,
  [3425] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(740), 1,
      sym_number,
  [3432] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(742), 1,
      sym_identifier,
  [3439] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(744), 1,
      sym_identifier,
  [3446] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(746), 1,
      anon_sym_DASH_GT,
  [3453] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(748), 1,
      sym_identifier,
  [3460] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(750), 1,
      anon_sym_player,
  [3467] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(752), 1,
      anon_sym_set,
  [3474] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(754), 1,
      sym_identifier,
  [3481] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(756), 1,
      anon_sym_LBRACE,
  [3488] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(758), 1,
      sym_identifier,
  [3495] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(760), 1,
      anon_sym_rooms,
  [3502] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(762), 1,
      ts_builtin_sym_end,
  [3509] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(764), 1,
      sym_identifier,
  [3516] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(766), 1,
      sym_identifier,
  [3523] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(768), 1,
      sym_identifier,
  [3530] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(770), 1,
      sym_identifier,
  [3537] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(772), 1,
      sym_identifier,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 42,
  [SMALL_STATE(4)] = 93,
  [SMALL_STATE(5)] = 144,
  [SMALL_STATE(6)] = 195,
  [SMALL_STATE(7)] = 231,
  [SMALL_STATE(8)] = 267,
  [SMALL_STATE(9)] = 303,
  [SMALL_STATE(10)] = 343,
  [SMALL_STATE(11)] = 381,
  [SMALL_STATE(12)] = 405,
  [SMALL_STATE(13)] = 445,
  [SMALL_STATE(14)] = 485,
  [SMALL_STATE(15)] = 523,
  [SMALL_STATE(16)] = 547,
  [SMALL_STATE(17)] = 583,
  [SMALL_STATE(18)] = 619,
  [SMALL_STATE(19)] = 653,
  [SMALL_STATE(20)] = 689,
  [SMALL_STATE(21)] = 710,
  [SMALL_STATE(22)] = 739,
  [SMALL_STATE(23)] = 768,
  [SMALL_STATE(24)] = 797,
  [SMALL_STATE(25)] = 826,
  [SMALL_STATE(26)] = 855,
  [SMALL_STATE(27)] = 883,
  [SMALL_STATE(28)] = 901,
  [SMALL_STATE(29)] = 919,
  [SMALL_STATE(30)] = 937,
  [SMALL_STATE(31)] = 955,
  [SMALL_STATE(32)] = 973,
  [SMALL_STATE(33)] = 991,
  [SMALL_STATE(34)] = 1009,
  [SMALL_STATE(35)] = 1037,
  [SMALL_STATE(36)] = 1056,
  [SMALL_STATE(37)] = 1075,
  [SMALL_STATE(38)] = 1093,
  [SMALL_STATE(39)] = 1111,
  [SMALL_STATE(40)] = 1133,
  [SMALL_STATE(41)] = 1151,
  [SMALL_STATE(42)] = 1169,
  [SMALL_STATE(43)] = 1187,
  [SMALL_STATE(44)] = 1205,
  [SMALL_STATE(45)] = 1229,
  [SMALL_STATE(46)] = 1247,
  [SMALL_STATE(47)] = 1265,
  [SMALL_STATE(48)] = 1287,
  [SMALL_STATE(49)] = 1305,
  [SMALL_STATE(50)] = 1323,
  [SMALL_STATE(51)] = 1348,
  [SMALL_STATE(52)] = 1367,
  [SMALL_STATE(53)] = 1386,
  [SMALL_STATE(54)] = 1405,
  [SMALL_STATE(55)] = 1430,
  [SMALL_STATE(56)] = 1455,
  [SMALL_STATE(57)] = 1474,
  [SMALL_STATE(58)] = 1493,
  [SMALL_STATE(59)] = 1514,
  [SMALL_STATE(60)] = 1535,
  [SMALL_STATE(61)] = 1549,
  [SMALL_STATE(62)] = 1563,
  [SMALL_STATE(63)] = 1583,
  [SMALL_STATE(64)] = 1603,
  [SMALL_STATE(65)] = 1619,
  [SMALL_STATE(66)] = 1639,
  [SMALL_STATE(67)] = 1659,
  [SMALL_STATE(68)] = 1673,
  [SMALL_STATE(69)] = 1689,
  [SMALL_STATE(70)] = 1703,
  [SMALL_STATE(71)] = 1719,
  [SMALL_STATE(72)] = 1733,
  [SMALL_STATE(73)] = 1747,
  [SMALL_STATE(74)] = 1761,
  [SMALL_STATE(75)] = 1777,
  [SMALL_STATE(76)] = 1791,
  [SMALL_STATE(77)] = 1805,
  [SMALL_STATE(78)] = 1825,
  [SMALL_STATE(79)] = 1841,
  [SMALL_STATE(80)] = 1857,
  [SMALL_STATE(81)] = 1871,
  [SMALL_STATE(82)] = 1887,
  [SMALL_STATE(83)] = 1903,
  [SMALL_STATE(84)] = 1917,
  [SMALL_STATE(85)] = 1933,
  [SMALL_STATE(86)] = 1949,
  [SMALL_STATE(87)] = 1963,
  [SMALL_STATE(88)] = 1977,
  [SMALL_STATE(89)] = 1991,
  [SMALL_STATE(90)] = 2005,
  [SMALL_STATE(91)] = 2021,
  [SMALL_STATE(92)] = 2035,
  [SMALL_STATE(93)] = 2051,
  [SMALL_STATE(94)] = 2065,
  [SMALL_STATE(95)] = 2079,
  [SMALL_STATE(96)] = 2093,
  [SMALL_STATE(97)] = 2114,
  [SMALL_STATE(98)] = 2129,
  [SMALL_STATE(99)] = 2144,
  [SMALL_STATE(100)] = 2165,
  [SMALL_STATE(101)] = 2180,
  [SMALL_STATE(102)] = 2195,
  [SMALL_STATE(103)] = 2210,
  [SMALL_STATE(104)] = 2231,
  [SMALL_STATE(105)] = 2246,
  [SMALL_STATE(106)] = 2261,
  [SMALL_STATE(107)] = 2276,
  [SMALL_STATE(108)] = 2295,
  [SMALL_STATE(109)] = 2310,
  [SMALL_STATE(110)] = 2331,
  [SMALL_STATE(111)] = 2344,
  [SMALL_STATE(112)] = 2365,
  [SMALL_STATE(113)] = 2380,
  [SMALL_STATE(114)] = 2399,
  [SMALL_STATE(115)] = 2415,
  [SMALL_STATE(116)] = 2431,
  [SMALL_STATE(117)] = 2447,
  [SMALL_STATE(118)] = 2463,
  [SMALL_STATE(119)] = 2479,
  [SMALL_STATE(120)] = 2499,
  [SMALL_STATE(121)] = 2515,
  [SMALL_STATE(122)] = 2531,
  [SMALL_STATE(123)] = 2547,
  [SMALL_STATE(124)] = 2563,
  [SMALL_STATE(125)] = 2585,
  [SMALL_STATE(126)] = 2607,
  [SMALL_STATE(127)] = 2623,
  [SMALL_STATE(128)] = 2639,
  [SMALL_STATE(129)] = 2655,
  [SMALL_STATE(130)] = 2675,
  [SMALL_STATE(131)] = 2691,
  [SMALL_STATE(132)] = 2702,
  [SMALL_STATE(133)] = 2717,
  [SMALL_STATE(134)] = 2728,
  [SMALL_STATE(135)] = 2745,
  [SMALL_STATE(136)] = 2756,
  [SMALL_STATE(137)] = 2767,
  [SMALL_STATE(138)] = 2778,
  [SMALL_STATE(139)] = 2791,
  [SMALL_STATE(140)] = 2804,
  [SMALL_STATE(141)] = 2817,
  [SMALL_STATE(142)] = 2831,
  [SMALL_STATE(143)] = 2845,
  [SMALL_STATE(144)] = 2859,
  [SMALL_STATE(145)] = 2869,
  [SMALL_STATE(146)] = 2878,
  [SMALL_STATE(147)] = 2889,
  [SMALL_STATE(148)] = 2898,
  [SMALL_STATE(149)] = 2909,
  [SMALL_STATE(150)] = 2920,
  [SMALL_STATE(151)] = 2933,
  [SMALL_STATE(152)] = 2942,
  [SMALL_STATE(153)] = 2951,
  [SMALL_STATE(154)] = 2960,
  [SMALL_STATE(155)] = 2971,
  [SMALL_STATE(156)] = 2984,
  [SMALL_STATE(157)] = 2997,
  [SMALL_STATE(158)] = 3010,
  [SMALL_STATE(159)] = 3023,
  [SMALL_STATE(160)] = 3034,
  [SMALL_STATE(161)] = 3045,
  [SMALL_STATE(162)] = 3058,
  [SMALL_STATE(163)] = 3071,
  [SMALL_STATE(164)] = 3080,
  [SMALL_STATE(165)] = 3091,
  [SMALL_STATE(166)] = 3104,
  [SMALL_STATE(167)] = 3113,
  [SMALL_STATE(168)] = 3126,
  [SMALL_STATE(169)] = 3137,
  [SMALL_STATE(170)] = 3147,
  [SMALL_STATE(171)] = 3155,
  [SMALL_STATE(172)] = 3165,
  [SMALL_STATE(173)] = 3173,
  [SMALL_STATE(174)] = 3183,
  [SMALL_STATE(175)] = 3193,
  [SMALL_STATE(176)] = 3201,
  [SMALL_STATE(177)] = 3211,
  [SMALL_STATE(178)] = 3219,
  [SMALL_STATE(179)] = 3227,
  [SMALL_STATE(180)] = 3237,
  [SMALL_STATE(181)] = 3247,
  [SMALL_STATE(182)] = 3257,
  [SMALL_STATE(183)] = 3264,
  [SMALL_STATE(184)] = 3271,
  [SMALL_STATE(185)] = 3278,
  [SMALL_STATE(186)] = 3285,
  [SMALL_STATE(187)] = 3292,
  [SMALL_STATE(188)] = 3299,
  [SMALL_STATE(189)] = 3306,
  [SMALL_STATE(190)] = 3313,
  [SMALL_STATE(191)] = 3320,
  [SMALL_STATE(192)] = 3327,
  [SMALL_STATE(193)] = 3334,
  [SMALL_STATE(194)] = 3341,
  [SMALL_STATE(195)] = 3348,
  [SMALL_STATE(196)] = 3355,
  [SMALL_STATE(197)] = 3362,
  [SMALL_STATE(198)] = 3369,
  [SMALL_STATE(199)] = 3376,
  [SMALL_STATE(200)] = 3383,
  [SMALL_STATE(201)] = 3390,
  [SMALL_STATE(202)] = 3397,
  [SMALL_STATE(203)] = 3404,
  [SMALL_STATE(204)] = 3411,
  [SMALL_STATE(205)] = 3418,
  [SMALL_STATE(206)] = 3425,
  [SMALL_STATE(207)] = 3432,
  [SMALL_STATE(208)] = 3439,
  [SMALL_STATE(209)] = 3446,
  [SMALL_STATE(210)] = 3453,
  [SMALL_STATE(211)] = 3460,
  [SMALL_STATE(212)] = 3467,
  [SMALL_STATE(213)] = 3474,
  [SMALL_STATE(214)] = 3481,
  [SMALL_STATE(215)] = 3488,
  [SMALL_STATE(216)] = 3495,
  [SMALL_STATE(217)] = 3502,
  [SMALL_STATE(218)] = 3509,
  [SMALL_STATE(219)] = 3516,
  [SMALL_STATE(220)] = 3523,
  [SMALL_STATE(221)] = 3530,
  [SMALL_STATE(222)] = 3537,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_program, 0),
  [7] = {.entry = {.count = 1, .reusable = true}}, SHIFT(212),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(113),
  [11] = {.entry = {.count = 1, .reusable = true}}, SHIFT(222),
  [13] = {.entry = {.count = 1, .reusable = true}}, SHIFT(221),
  [15] = {.entry = {.count = 1, .reusable = true}}, SHIFT(220),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(219),
  [19] = {.entry = {.count = 1, .reusable = true}}, SHIFT(218),
  [21] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string, 1),
  [23] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string, 1),
  [25] = {.entry = {.count = 1, .reusable = true}}, SHIFT(89),
  [27] = {.entry = {.count = 1, .reusable = true}}, SHIFT(127),
  [29] = {.entry = {.count = 1, .reusable = false}}, SHIFT(114),
  [31] = {.entry = {.count = 1, .reusable = true}}, SHIFT(114),
  [33] = {.entry = {.count = 1, .reusable = true}}, SHIFT(128),
  [35] = {.entry = {.count = 1, .reusable = true}}, SHIFT(168),
  [37] = {.entry = {.count = 1, .reusable = true}}, SHIFT(186),
  [39] = {.entry = {.count = 1, .reusable = true}}, SHIFT(182),
  [41] = {.entry = {.count = 1, .reusable = true}}, SHIFT(146),
  [43] = {.entry = {.count = 1, .reusable = true}}, SHIFT(132),
  [45] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_item_block_repeat1, 2),
  [47] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_item_block_repeat1, 2), SHIFT_REPEAT(127),
  [50] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_item_block_repeat1, 2), SHIFT_REPEAT(114),
  [53] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_item_block_repeat1, 2), SHIFT_REPEAT(114),
  [56] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_item_block_repeat1, 2), SHIFT_REPEAT(128),
  [59] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_item_block_repeat1, 2), SHIFT_REPEAT(168),
  [62] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_item_block_repeat1, 2), SHIFT_REPEAT(186),
  [65] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_item_block_repeat1, 2), SHIFT_REPEAT(182),
  [68] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_item_block_repeat1, 2), SHIFT_REPEAT(146),
  [71] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_item_block_repeat1, 2), SHIFT_REPEAT(132),
  [74] = {.entry = {.count = 1, .reusable = true}}, SHIFT(80),
  [76] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_def, 4),
  [78] = {.entry = {.count = 1, .reusable = true}}, SHIFT(116),
  [80] = {.entry = {.count = 1, .reusable = true}}, SHIFT(147),
  [82] = {.entry = {.count = 1, .reusable = true}}, SHIFT(202),
  [84] = {.entry = {.count = 1, .reusable = true}}, SHIFT(205),
  [86] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_def, 3),
  [88] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_goal_def_repeat1, 2),
  [90] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_goal_def_repeat1, 2), SHIFT_REPEAT(116),
  [93] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_goal_def_repeat1, 2), SHIFT_REPEAT(147),
  [96] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_goal_def_repeat1, 2), SHIFT_REPEAT(202),
  [99] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_goal_def_repeat1, 2), SHIFT_REPEAT(205),
  [102] = {.entry = {.count = 1, .reusable = true}}, SHIFT(69),
  [104] = {.entry = {.count = 1, .reusable = true}}, SHIFT(118),
  [106] = {.entry = {.count = 1, .reusable = false}}, SHIFT(117),
  [108] = {.entry = {.count = 1, .reusable = true}}, SHIFT(117),
  [110] = {.entry = {.count = 1, .reusable = true}}, SHIFT(138),
  [112] = {.entry = {.count = 1, .reusable = true}}, SHIFT(178),
  [114] = {.entry = {.count = 1, .reusable = true}}, SHIFT(139),
  [116] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_program, 1),
  [118] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_npc_block_repeat1, 2),
  [120] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_npc_block_repeat1, 2), SHIFT_REPEAT(118),
  [123] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_npc_block_repeat1, 2), SHIFT_REPEAT(117),
  [126] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_npc_block_repeat1, 2), SHIFT_REPEAT(117),
  [129] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_npc_block_repeat1, 2), SHIFT_REPEAT(138),
  [132] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_npc_block_repeat1, 2), SHIFT_REPEAT(178),
  [135] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_npc_block_repeat1, 2), SHIFT_REPEAT(139),
  [138] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_npc_block_repeat1, 2), SHIFT_REPEAT(132),
  [141] = {.entry = {.count = 1, .reusable = true}}, SHIFT(75),
  [143] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2),
  [145] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2), SHIFT_REPEAT(212),
  [148] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2), SHIFT_REPEAT(113),
  [151] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2), SHIFT_REPEAT(222),
  [154] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2), SHIFT_REPEAT(221),
  [157] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2), SHIFT_REPEAT(220),
  [160] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2), SHIFT_REPEAT(219),
  [163] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2), SHIFT_REPEAT(218),
  [166] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_boolean, 1),
  [168] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_boolean, 1),
  [170] = {.entry = {.count = 1, .reusable = true}}, SHIFT(95),
  [172] = {.entry = {.count = 1, .reusable = true}}, SHIFT(123),
  [174] = {.entry = {.count = 1, .reusable = false}}, SHIFT(126),
  [176] = {.entry = {.count = 1, .reusable = true}}, SHIFT(126),
  [178] = {.entry = {.count = 1, .reusable = true}}, SHIFT(159),
  [180] = {.entry = {.count = 1, .reusable = true}}, SHIFT(184),
  [182] = {.entry = {.count = 1, .reusable = true}}, SHIFT(107),
  [184] = {.entry = {.count = 1, .reusable = true}}, SHIFT(73),
  [186] = {.entry = {.count = 1, .reusable = false}}, SHIFT(24),
  [188] = {.entry = {.count = 1, .reusable = false}}, SHIFT(35),
  [190] = {.entry = {.count = 1, .reusable = true}}, SHIFT(35),
  [192] = {.entry = {.count = 1, .reusable = true}}, SHIFT(96),
  [194] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_do_stmt, 2),
  [196] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_do_stmt, 2),
  [198] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_room_block_repeat1, 2),
  [200] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_room_block_repeat1, 2), SHIFT_REPEAT(123),
  [203] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_room_block_repeat1, 2), SHIFT_REPEAT(126),
  [206] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_room_block_repeat1, 2), SHIFT_REPEAT(126),
  [209] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_room_block_repeat1, 2), SHIFT_REPEAT(159),
  [212] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_room_block_repeat1, 2), SHIFT_REPEAT(184),
  [215] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_room_block_repeat1, 2), SHIFT_REPEAT(107),
  [218] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_location, 3),
  [220] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_location, 3),
  [222] = {.entry = {.count = 1, .reusable = false}}, SHIFT(23),
  [224] = {.entry = {.count = 1, .reusable = false}}, SHIFT(36),
  [226] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [228] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [230] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_cond_line_ext_repeat1, 2), SHIFT_REPEAT(22),
  [233] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_cond_line_ext_repeat1, 2), SHIFT_REPEAT(36),
  [236] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_cond_line_ext_repeat1, 2), SHIFT_REPEAT(36),
  [239] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_cond_line_ext_repeat1, 2), SHIFT_REPEAT(22),
  [242] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_cond_line_ext_repeat1, 2),
  [244] = {.entry = {.count = 1, .reusable = false}}, SHIFT(22),
  [246] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [248] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_cond_line_ext, 1),
  [250] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_do_stmt_repeat1, 2), SHIFT_REPEAT(24),
  [253] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_do_stmt_repeat1, 2), SHIFT_REPEAT(35),
  [256] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_do_stmt_repeat1, 2), SHIFT_REPEAT(35),
  [259] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_do_stmt_repeat1, 2),
  [261] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_do_stmt_repeat1, 2),
  [263] = {.entry = {.count = 1, .reusable = false}}, SHIFT(34),
  [265] = {.entry = {.count = 1, .reusable = false}}, SHIFT(46),
  [267] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [269] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_cond_line, 1),
  [271] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_cond_line, 1),
  [273] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_done, 3),
  [275] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_cond, 3),
  [277] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_start, 3),
  [279] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_desc, 2),
  [281] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_group, 2),
  [283] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_cond, 4),
  [285] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_stmt, 1),
  [287] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_do_stmt_repeat1, 2), SHIFT_REPEAT(34),
  [290] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_do_stmt_repeat1, 2), SHIFT_REPEAT(46),
  [293] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_do_stmt_repeat1, 2), SHIFT_REPEAT(46),
  [296] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_portable, 2),
  [298] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_item_portable, 2),
  [300] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_container_state, 3),
  [302] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_item_container_state, 3),
  [304] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_movement_stmt, 7),
  [306] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_movement_stmt, 7),
  [308] = {.entry = {.count = 1, .reusable = true}}, SHIFT(204),
  [310] = {.entry = {.count = 1, .reusable = true}}, SHIFT(160),
  [312] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_ability, 2),
  [314] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_item_ability, 2),
  [316] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_restricted, 2),
  [318] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_item_restricted, 2),
  [320] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_text, 2),
  [322] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_item_text, 2),
  [324] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_desc, 2),
  [326] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_item_desc, 2),
  [328] = {.entry = {.count = 1, .reusable = false}}, SHIFT(26),
  [330] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_name, 2),
  [332] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_item_name, 2),
  [334] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_movement_stmt, 6),
  [336] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_movement_stmt, 6),
  [338] = {.entry = {.count = 1, .reusable = true}}, SHIFT(197),
  [340] = {.entry = {.count = 1, .reusable = true}}, SHIFT(164),
  [342] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_location, 1),
  [344] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_item_location, 1),
  [346] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_stmt, 1),
  [348] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_item_stmt, 1),
  [350] = {.entry = {.count = 1, .reusable = true}}, SHIFT(104),
  [352] = {.entry = {.count = 1, .reusable = true}}, SHIFT(187),
  [354] = {.entry = {.count = 1, .reusable = true}}, SHIFT(183),
  [356] = {.entry = {.count = 1, .reusable = true}}, SHIFT(120),
  [358] = {.entry = {.count = 1, .reusable = true}}, SHIFT(130),
  [360] = {.entry = {.count = 1, .reusable = true}}, SHIFT(106),
  [362] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_movement_stmt, 9),
  [364] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_movement_stmt, 9),
  [366] = {.entry = {.count = 1, .reusable = true}}, SHIFT(148),
  [368] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_movement_stmt, 8),
  [370] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_movement_stmt, 8),
  [372] = {.entry = {.count = 1, .reusable = true}}, SHIFT(154),
  [374] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_exit_block_repeat1, 2),
  [376] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_exit_block_repeat1, 2), SHIFT_REPEAT(187),
  [379] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_exit_block_repeat1, 2), SHIFT_REPEAT(183),
  [382] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_exit_block_repeat1, 2), SHIFT_REPEAT(120),
  [385] = {.entry = {.count = 1, .reusable = true}}, SHIFT(101),
  [387] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_overlay_block_repeat1, 2), SHIFT_REPEAT(130),
  [390] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_overlay_block_repeat1, 2),
  [392] = {.entry = {.count = 1, .reusable = true}}, SHIFT(108),
  [394] = {.entry = {.count = 1, .reusable = false}}, SHIFT(18),
  [396] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [398] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_exit_stmt, 4),
  [400] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_exit_stmt, 4),
  [402] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_trigger_block, 2),
  [404] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_spinner_def, 3),
  [406] = {.entry = {.count = 1, .reusable = false}}, SHIFT(2),
  [408] = {.entry = {.count = 1, .reusable = true}}, SHIFT(2),
  [410] = {.entry = {.count = 1, .reusable = true}}, SHIFT(82),
  [412] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_npc_desc, 2),
  [414] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_npc_desc, 2),
  [416] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_dialogue_stmt_repeat1, 2), SHIFT_REPEAT(2),
  [419] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_dialogue_stmt_repeat1, 2), SHIFT_REPEAT(2),
  [422] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_dialogue_stmt_repeat1, 2),
  [424] = {.entry = {.count = 1, .reusable = true}}, SHIFT(90),
  [426] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_spinner_block, 2),
  [428] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_dialogue_stmt, 4),
  [430] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_dialogue_stmt, 4),
  [432] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_npc_block, 2),
  [434] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_npc_name, 2),
  [436] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_npc_name, 2),
  [438] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_spinner_block, 3),
  [440] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_list, 3),
  [442] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_room_block, 2),
  [444] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_npc_stmt, 1),
  [446] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_npc_stmt, 1),
  [448] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_npc_block, 3),
  [450] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_trigger, 4),
  [452] = {.entry = {.count = 1, .reusable = true}}, SHIFT(68),
  [454] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_npc_state, 3),
  [456] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_npc_state, 3),
  [458] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_movement_stmt, 11),
  [460] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_movement_stmt, 11),
  [462] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_block, 3),
  [464] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_movement_stmt, 10),
  [466] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_movement_stmt, 10),
  [468] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_dialogue_stmt, 5),
  [470] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_dialogue_stmt, 5),
  [472] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_decl, 5),
  [474] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_npc_def, 3),
  [476] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_list, 4),
  [478] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_trigger_block, 3),
  [480] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_block, 2),
  [482] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_dialogue_stmt, 6),
  [484] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_dialogue_stmt, 6),
  [486] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_trigger, 3),
  [488] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_npc_state, 2),
  [490] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_npc_state, 2),
  [492] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_room_def, 3),
  [494] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_def, 3),
  [496] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_room_block, 3),
  [498] = {.entry = {.count = 1, .reusable = true}}, SHIFT(151),
  [500] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [502] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [504] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_room_visited, 2),
  [506] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_room_visited, 2),
  [508] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_room_desc, 2),
  [510] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_room_desc, 2),
  [512] = {.entry = {.count = 1, .reusable = true}}, SHIFT(60),
  [514] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_room_name, 2),
  [516] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_room_name, 2),
  [518] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_exit_block, 3),
  [520] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_exit_block, 3),
  [522] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_exit_stmt, 5),
  [524] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_exit_stmt, 5),
  [526] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_trigger_block_repeat1, 2),
  [528] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_trigger_block_repeat1, 2), SHIFT_REPEAT(21),
  [531] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_trigger_block_repeat1, 2), SHIFT_REPEAT(58),
  [534] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_exit_block, 2),
  [536] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_exit_block, 2),
  [538] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_room_stmt, 1),
  [540] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_room_stmt, 1),
  [542] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_overlay_block, 2),
  [544] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_overlay_block, 2),
  [546] = {.entry = {.count = 1, .reusable = false}}, SHIFT(209),
  [548] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_overlay_block, 3),
  [550] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_overlay_block, 3),
  [552] = {.entry = {.count = 1, .reusable = true}}, SHIFT(88),
  [554] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_overlay_entry, 2),
  [556] = {.entry = {.count = 1, .reusable = true}}, SHIFT(166),
  [558] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_overlay_stmt, 4),
  [560] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_overlay_stmt, 4),
  [562] = {.entry = {.count = 1, .reusable = false}}, SHIFT(119),
  [564] = {.entry = {.count = 1, .reusable = false}}, SHIFT(11),
  [566] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [568] = {.entry = {.count = 1, .reusable = true}}, SHIFT(190),
  [570] = {.entry = {.count = 1, .reusable = true}}, SHIFT(44),
  [572] = {.entry = {.count = 1, .reusable = true}}, SHIFT(99),
  [574] = {.entry = {.count = 1, .reusable = true}}, SHIFT(200),
  [576] = {.entry = {.count = 1, .reusable = true}}, SHIFT(177),
  [578] = {.entry = {.count = 1, .reusable = true}}, SHIFT(179),
  [580] = {.entry = {.count = 1, .reusable = true}}, SHIFT(199),
  [582] = {.entry = {.count = 1, .reusable = true}}, SHIFT(198),
  [584] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_exit_required_items, 4),
  [586] = {.entry = {.count = 1, .reusable = true}}, SHIFT(210),
  [588] = {.entry = {.count = 1, .reusable = true}}, SHIFT(211),
  [590] = {.entry = {.count = 1, .reusable = true}}, SHIFT(122),
  [592] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_exit_required_flags, 4),
  [594] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_trigger_repeat1, 2), SHIFT_REPEAT(190),
  [597] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_trigger_repeat1, 2), SHIFT_REPEAT(44),
  [600] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_trigger_repeat1, 2),
  [602] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_exit_barred, 2),
  [604] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_exit_required_flags, 5),
  [606] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_exit_required_items, 5),
  [608] = {.entry = {.count = 1, .reusable = true}}, SHIFT(92),
  [610] = {.entry = {.count = 1, .reusable = true}}, SHIFT(188),
  [612] = {.entry = {.count = 1, .reusable = true}}, SHIFT(214),
  [614] = {.entry = {.count = 1, .reusable = true}}, SHIFT(213),
  [616] = {.entry = {.count = 1, .reusable = true}}, SHIFT(144),
  [618] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_exit_attr, 1),
  [620] = {.entry = {.count = 1, .reusable = true}}, SHIFT(67),
  [622] = {.entry = {.count = 1, .reusable = true}}, SHIFT(115),
  [624] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_spinner_block_repeat1, 2),
  [626] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_spinner_block_repeat1, 2), SHIFT_REPEAT(115),
  [629] = {.entry = {.count = 1, .reusable = true}}, SHIFT(71),
  [631] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_exit_attr, 2),
  [633] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_do_stmt, 3),
  [635] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [637] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [639] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_wedge_stmt, 2),
  [641] = {.entry = {.count = 1, .reusable = true}}, SHIFT(206),
  [643] = {.entry = {.count = 1, .reusable = true}}, SHIFT(196),
  [645] = {.entry = {.count = 1, .reusable = true}}, SHIFT(87),
  [647] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_braced_block, 2),
  [649] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_trigger_mod, 2),
  [651] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_trigger_stmt, 1),
  [653] = {.entry = {.count = 1, .reusable = true}}, SHIFT(133),
  [655] = {.entry = {.count = 1, .reusable = true}}, SHIFT(137),
  [657] = {.entry = {.count = 1, .reusable = true}}, SHIFT(131),
  [659] = {.entry = {.count = 1, .reusable = true}}, SHIFT(72),
  [661] = {.entry = {.count = 1, .reusable = true}}, SHIFT(47),
  [663] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [665] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_block, 3),
  [667] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_list_repeat1, 2), SHIFT_REPEAT(196),
  [670] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_set_list_repeat1, 2),
  [672] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_braced_block, 3),
  [674] = {.entry = {.count = 1, .reusable = true}}, SHIFT(136),
  [676] = {.entry = {.count = 1, .reusable = true}}, SHIFT(51),
  [678] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [680] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [682] = {.entry = {.count = 1, .reusable = true}}, SHIFT(141),
  [684] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_wedge_stmt, 4),
  [686] = {.entry = {.count = 1, .reusable = true}}, SHIFT(9),
  [688] = {.entry = {.count = 1, .reusable = true}}, SHIFT(193),
  [690] = {.entry = {.count = 1, .reusable = true}}, SHIFT(216),
  [692] = {.entry = {.count = 1, .reusable = true}}, SHIFT(189),
  [694] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [696] = {.entry = {.count = 1, .reusable = true}}, SHIFT(207),
  [698] = {.entry = {.count = 1, .reusable = true}}, SHIFT(172),
  [700] = {.entry = {.count = 1, .reusable = true}}, SHIFT(192),
  [702] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [704] = {.entry = {.count = 1, .reusable = false}}, SHIFT(32),
  [706] = {.entry = {.count = 1, .reusable = false}}, SHIFT(40),
  [708] = {.entry = {.count = 1, .reusable = true}}, SHIFT(191),
  [710] = {.entry = {.count = 1, .reusable = false}}, SHIFT(78),
  [712] = {.entry = {.count = 1, .reusable = true}}, SHIFT(185),
  [714] = {.entry = {.count = 1, .reusable = true}}, SHIFT(152),
  [716] = {.entry = {.count = 1, .reusable = false}}, SHIFT(155),
  [718] = {.entry = {.count = 1, .reusable = false}}, SHIFT(157),
  [720] = {.entry = {.count = 1, .reusable = false}}, SHIFT(28),
  [722] = {.entry = {.count = 1, .reusable = false}}, SHIFT(161),
  [724] = {.entry = {.count = 1, .reusable = true}}, SHIFT(181),
  [726] = {.entry = {.count = 1, .reusable = false}}, SHIFT(170),
  [728] = {.entry = {.count = 1, .reusable = false}}, SHIFT(53),
  [730] = {.entry = {.count = 1, .reusable = true}}, SHIFT(62),
  [732] = {.entry = {.count = 1, .reusable = true}}, SHIFT(124),
  [734] = {.entry = {.count = 1, .reusable = true}}, SHIFT(194),
  [736] = {.entry = {.count = 1, .reusable = false}}, SHIFT(52),
  [738] = {.entry = {.count = 1, .reusable = true}}, SHIFT(125),
  [740] = {.entry = {.count = 1, .reusable = true}}, SHIFT(175),
  [742] = {.entry = {.count = 1, .reusable = false}}, SHIFT(158),
  [744] = {.entry = {.count = 1, .reusable = false}}, SHIFT(59),
  [746] = {.entry = {.count = 1, .reusable = true}}, SHIFT(208),
  [748] = {.entry = {.count = 1, .reusable = false}}, SHIFT(20),
  [750] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [752] = {.entry = {.count = 1, .reusable = true}}, SHIFT(215),
  [754] = {.entry = {.count = 1, .reusable = false}}, SHIFT(201),
  [756] = {.entry = {.count = 1, .reusable = true}}, SHIFT(77),
  [758] = {.entry = {.count = 1, .reusable = false}}, SHIFT(195),
  [760] = {.entry = {.count = 1, .reusable = true}}, SHIFT(203),
  [762] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [764] = {.entry = {.count = 1, .reusable = false}}, SHIFT(121),
  [766] = {.entry = {.count = 1, .reusable = false}}, SHIFT(176),
  [768] = {.entry = {.count = 1, .reusable = false}}, SHIFT(174),
  [770] = {.entry = {.count = 1, .reusable = false}}, SHIFT(173),
  [772] = {.entry = {.count = 1, .reusable = false}}, SHIFT(180),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef _WIN32
#define extern __declspec(dllexport)
#endif

extern const TSLanguage *tree_sitter_amble_dsl(void) {
  static const TSLanguage language = {
    .version = LANGUAGE_VERSION,
    .symbol_count = SYMBOL_COUNT,
    .alias_count = ALIAS_COUNT,
    .token_count = TOKEN_COUNT,
    .external_token_count = EXTERNAL_TOKEN_COUNT,
    .state_count = STATE_COUNT,
    .large_state_count = LARGE_STATE_COUNT,
    .production_id_count = PRODUCTION_ID_COUNT,
    .field_count = FIELD_COUNT,
    .max_alias_sequence_length = MAX_ALIAS_SEQUENCE_LENGTH,
    .parse_table = &ts_parse_table[0][0],
    .small_parse_table = ts_small_parse_table,
    .small_parse_table_map = ts_small_parse_table_map,
    .parse_actions = ts_parse_actions,
    .symbol_names = ts_symbol_names,
    .symbol_metadata = ts_symbol_metadata,
    .public_symbol_map = ts_symbol_map,
    .alias_map = ts_non_terminal_alias_map,
    .alias_sequences = &ts_alias_sequences[0][0],
    .lex_modes = ts_lex_modes,
    .lex_fn = ts_lex,
    .primary_state_ids = ts_primary_state_ids,
  };
  return &language;
}
#ifdef __cplusplus
}
#endif
