#include <tree_sitter/parser.h>

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 125
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 94
#define ALIAS_COUNT 0
#define TOKEN_COUNT 50
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 0
#define MAX_ALIAS_SEQUENCE_LENGTH 5
#define PRODUCTION_ID_COUNT 1

enum {
  sym_comment = 1,
  sym_identifier = 2,
  sym_number = 3,
  aux_sym_string_token1 = 4,
  aux_sym_string_token2 = 5,
  anon_sym_DQUOTE_DQUOTE_DQUOTE = 6,
  aux_sym_string_token3 = 7,
  aux_sym_string_token4 = 8,
  aux_sym_string_token5 = 9,
  anon_sym_true = 10,
  anon_sym_false = 11,
  anon_sym_let = 12,
  anon_sym_set = 13,
  anon_sym_EQ = 14,
  anon_sym_LPAREN = 15,
  anon_sym_COMMA = 16,
  anon_sym_RPAREN = 17,
  anon_sym_trigger = 18,
  anon_sym_LBRACE = 19,
  anon_sym_RBRACE = 20,
  anon_sym_room = 21,
  anon_sym_name = 22,
  anon_sym_desc = 23,
  anon_sym_description = 24,
  anon_sym_visited = 25,
  anon_sym_exit = 26,
  anon_sym_DASH_GT = 27,
  anon_sym_item = 28,
  anon_sym_portable = 29,
  anon_sym_spinner = 30,
  anon_sym_wedge = 31,
  anon_sym_width = 32,
  anon_sym_npc = 33,
  anon_sym_location = 34,
  anon_sym_nowhere = 35,
  anon_sym_goal = 36,
  anon_sym_group = 37,
  anon_sym_required = 38,
  anon_sym_optional = 39,
  anon_sym_status_DASHeffect = 40,
  anon_sym_done = 41,
  anon_sym_when = 42,
  anon_sym_has = 43,
  anon_sym_flag = 44,
  anon_sym_missing = 45,
  anon_sym_reached = 46,
  anon_sym_complete = 47,
  anon_sym_in = 48,
  anon_sym_progress = 49,
  sym_program = 50,
  sym_string = 51,
  sym_boolean = 52,
  sym_set_decl = 53,
  sym_set_list = 54,
  sym_trigger = 55,
  sym__trigger_stmt = 56,
  sym_room_def = 57,
  sym_room_block = 58,
  sym_room_stmt = 59,
  sym_room_name = 60,
  sym_room_desc = 61,
  sym_room_visited = 62,
  sym_exit_stmt = 63,
  sym_item_def = 64,
  sym_item_block = 65,
  sym_item_stmt = 66,
  sym_item_name = 67,
  sym_item_desc = 68,
  sym_item_portable = 69,
  sym_spinner_def = 70,
  sym_spinner_block = 71,
  sym_wedge_stmt = 72,
  sym_npc_def = 73,
  sym_npc_block = 74,
  sym_npc_stmt = 75,
  sym_npc_name = 76,
  sym_npc_desc = 77,
  sym_npc_location = 78,
  sym_goal_def = 79,
  sym_goal_stmt = 80,
  sym_goal_desc = 81,
  sym_goal_group = 82,
  sym_goal_done = 83,
  sym_goal_cond = 84,
  aux_sym_program_repeat1 = 85,
  aux_sym_string_repeat1 = 86,
  aux_sym_set_list_repeat1 = 87,
  aux_sym_trigger_repeat1 = 88,
  aux_sym_room_block_repeat1 = 89,
  aux_sym_item_block_repeat1 = 90,
  aux_sym_spinner_block_repeat1 = 91,
  aux_sym_npc_block_repeat1 = 92,
  aux_sym_goal_def_repeat1 = 93,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [sym_comment] = "comment",
  [sym_identifier] = "identifier",
  [sym_number] = "number",
  [aux_sym_string_token1] = "string_token1",
  [aux_sym_string_token2] = "string_token2",
  [anon_sym_DQUOTE_DQUOTE_DQUOTE] = "\"\"\"",
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
  [anon_sym_LBRACE] = "{",
  [anon_sym_RBRACE] = "}",
  [anon_sym_room] = "room",
  [anon_sym_name] = "name",
  [anon_sym_desc] = "desc",
  [anon_sym_description] = "description",
  [anon_sym_visited] = "visited",
  [anon_sym_exit] = "exit",
  [anon_sym_DASH_GT] = "->",
  [anon_sym_item] = "item",
  [anon_sym_portable] = "portable",
  [anon_sym_spinner] = "spinner",
  [anon_sym_wedge] = "wedge",
  [anon_sym_width] = "width",
  [anon_sym_npc] = "npc",
  [anon_sym_location] = "location",
  [anon_sym_nowhere] = "nowhere",
  [anon_sym_goal] = "goal",
  [anon_sym_group] = "group",
  [anon_sym_required] = "required",
  [anon_sym_optional] = "optional",
  [anon_sym_status_DASHeffect] = "status-effect",
  [anon_sym_done] = "done",
  [anon_sym_when] = "when",
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
  [sym__trigger_stmt] = "_trigger_stmt",
  [sym_room_def] = "room_def",
  [sym_room_block] = "room_block",
  [sym_room_stmt] = "room_stmt",
  [sym_room_name] = "room_name",
  [sym_room_desc] = "room_desc",
  [sym_room_visited] = "room_visited",
  [sym_exit_stmt] = "exit_stmt",
  [sym_item_def] = "item_def",
  [sym_item_block] = "item_block",
  [sym_item_stmt] = "item_stmt",
  [sym_item_name] = "item_name",
  [sym_item_desc] = "item_desc",
  [sym_item_portable] = "item_portable",
  [sym_spinner_def] = "spinner_def",
  [sym_spinner_block] = "spinner_block",
  [sym_wedge_stmt] = "wedge_stmt",
  [sym_npc_def] = "npc_def",
  [sym_npc_block] = "npc_block",
  [sym_npc_stmt] = "npc_stmt",
  [sym_npc_name] = "npc_name",
  [sym_npc_desc] = "npc_desc",
  [sym_npc_location] = "npc_location",
  [sym_goal_def] = "goal_def",
  [sym_goal_stmt] = "goal_stmt",
  [sym_goal_desc] = "goal_desc",
  [sym_goal_group] = "goal_group",
  [sym_goal_done] = "goal_done",
  [sym_goal_cond] = "goal_cond",
  [aux_sym_program_repeat1] = "program_repeat1",
  [aux_sym_string_repeat1] = "string_repeat1",
  [aux_sym_set_list_repeat1] = "set_list_repeat1",
  [aux_sym_trigger_repeat1] = "trigger_repeat1",
  [aux_sym_room_block_repeat1] = "room_block_repeat1",
  [aux_sym_item_block_repeat1] = "item_block_repeat1",
  [aux_sym_spinner_block_repeat1] = "spinner_block_repeat1",
  [aux_sym_npc_block_repeat1] = "npc_block_repeat1",
  [aux_sym_goal_def_repeat1] = "goal_def_repeat1",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [sym_comment] = sym_comment,
  [sym_identifier] = sym_identifier,
  [sym_number] = sym_number,
  [aux_sym_string_token1] = aux_sym_string_token1,
  [aux_sym_string_token2] = aux_sym_string_token2,
  [anon_sym_DQUOTE_DQUOTE_DQUOTE] = anon_sym_DQUOTE_DQUOTE_DQUOTE,
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
  [anon_sym_LBRACE] = anon_sym_LBRACE,
  [anon_sym_RBRACE] = anon_sym_RBRACE,
  [anon_sym_room] = anon_sym_room,
  [anon_sym_name] = anon_sym_name,
  [anon_sym_desc] = anon_sym_desc,
  [anon_sym_description] = anon_sym_description,
  [anon_sym_visited] = anon_sym_visited,
  [anon_sym_exit] = anon_sym_exit,
  [anon_sym_DASH_GT] = anon_sym_DASH_GT,
  [anon_sym_item] = anon_sym_item,
  [anon_sym_portable] = anon_sym_portable,
  [anon_sym_spinner] = anon_sym_spinner,
  [anon_sym_wedge] = anon_sym_wedge,
  [anon_sym_width] = anon_sym_width,
  [anon_sym_npc] = anon_sym_npc,
  [anon_sym_location] = anon_sym_location,
  [anon_sym_nowhere] = anon_sym_nowhere,
  [anon_sym_goal] = anon_sym_goal,
  [anon_sym_group] = anon_sym_group,
  [anon_sym_required] = anon_sym_required,
  [anon_sym_optional] = anon_sym_optional,
  [anon_sym_status_DASHeffect] = anon_sym_status_DASHeffect,
  [anon_sym_done] = anon_sym_done,
  [anon_sym_when] = anon_sym_when,
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
  [sym__trigger_stmt] = sym__trigger_stmt,
  [sym_room_def] = sym_room_def,
  [sym_room_block] = sym_room_block,
  [sym_room_stmt] = sym_room_stmt,
  [sym_room_name] = sym_room_name,
  [sym_room_desc] = sym_room_desc,
  [sym_room_visited] = sym_room_visited,
  [sym_exit_stmt] = sym_exit_stmt,
  [sym_item_def] = sym_item_def,
  [sym_item_block] = sym_item_block,
  [sym_item_stmt] = sym_item_stmt,
  [sym_item_name] = sym_item_name,
  [sym_item_desc] = sym_item_desc,
  [sym_item_portable] = sym_item_portable,
  [sym_spinner_def] = sym_spinner_def,
  [sym_spinner_block] = sym_spinner_block,
  [sym_wedge_stmt] = sym_wedge_stmt,
  [sym_npc_def] = sym_npc_def,
  [sym_npc_block] = sym_npc_block,
  [sym_npc_stmt] = sym_npc_stmt,
  [sym_npc_name] = sym_npc_name,
  [sym_npc_desc] = sym_npc_desc,
  [sym_npc_location] = sym_npc_location,
  [sym_goal_def] = sym_goal_def,
  [sym_goal_stmt] = sym_goal_stmt,
  [sym_goal_desc] = sym_goal_desc,
  [sym_goal_group] = sym_goal_group,
  [sym_goal_done] = sym_goal_done,
  [sym_goal_cond] = sym_goal_cond,
  [aux_sym_program_repeat1] = aux_sym_program_repeat1,
  [aux_sym_string_repeat1] = aux_sym_string_repeat1,
  [aux_sym_set_list_repeat1] = aux_sym_set_list_repeat1,
  [aux_sym_trigger_repeat1] = aux_sym_trigger_repeat1,
  [aux_sym_room_block_repeat1] = aux_sym_room_block_repeat1,
  [aux_sym_item_block_repeat1] = aux_sym_item_block_repeat1,
  [aux_sym_spinner_block_repeat1] = aux_sym_spinner_block_repeat1,
  [aux_sym_npc_block_repeat1] = aux_sym_npc_block_repeat1,
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
  [anon_sym_DQUOTE_DQUOTE_DQUOTE] = {
    .visible = true,
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
  [anon_sym_LBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACE] = {
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
  [anon_sym_exit] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DASH_GT] = {
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
  [anon_sym_location] = {
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
  [anon_sym_when] = {
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
  [sym__trigger_stmt] = {
    .visible = false,
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
  [sym_exit_stmt] = {
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
  [sym_npc_location] = {
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
  [sym_goal_cond] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_program_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_repeat1] = {
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
  [aux_sym_room_block_repeat1] = {
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
  [11] = 11,
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
  [34] = 34,
  [35] = 35,
  [36] = 36,
  [37] = 37,
  [38] = 38,
  [39] = 39,
  [40] = 40,
  [41] = 41,
  [42] = 42,
  [43] = 43,
  [44] = 44,
  [45] = 45,
  [46] = 46,
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
  [57] = 49,
  [58] = 58,
  [59] = 51,
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
  [93] = 2,
  [94] = 94,
  [95] = 4,
  [96] = 3,
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
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(145);
      if (lookahead == '"') ADVANCE(2);
      if (lookahead == '#') ADVANCE(147);
      if (lookahead == '\'') ADVANCE(6);
      if (lookahead == '(') ADVANCE(164);
      if (lookahead == ')') ADVANCE(166);
      if (lookahead == ',') ADVANCE(165);
      if (lookahead == '-') ADVANCE(8);
      if (lookahead == '=') ADVANCE(163);
      if (lookahead == '\\') ADVANCE(143);
      if (lookahead == 'c') ADVANCE(96);
      if (lookahead == 'd') ADVANCE(30);
      if (lookahead == 'e') ADVANCE(141);
      if (lookahead == 'f') ADVANCE(9);
      if (lookahead == 'g') ADVANCE(100);
      if (lookahead == 'h') ADVANCE(13);
      if (lookahead == 'i') ADVANCE(87);
      if (lookahead == 'l') ADVANCE(31);
      if (lookahead == 'm') ADVANCE(67);
      if (lookahead == 'n') ADVANCE(12);
      if (lookahead == 'o') ADVANCE(106);
      if (lookahead == 'p') ADVANCE(98);
      if (lookahead == 'r') ADVANCE(32);
      if (lookahead == 's') ADVANCE(45);
      if (lookahead == 't') ADVANCE(109);
      if (lookahead == 'v') ADVANCE(70);
      if (lookahead == 'w') ADVANCE(33);
      if (lookahead == '{') ADVANCE(168);
      if (lookahead == '}') ADVANCE(169);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(0)
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(149);
      END_STATE();
    case 1:
      if (lookahead == '"') ADVANCE(2);
      if (lookahead == '#') ADVANCE(146);
      if (lookahead == '\'') ADVANCE(6);
      if (lookahead == '}') ADVANCE(169);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(1)
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(148);
      END_STATE();
    case 2:
      if (lookahead == '"') ADVANCE(151);
      if (lookahead == '\\') ADVANCE(144);
      if (lookahead != 0) ADVANCE(3);
      END_STATE();
    case 3:
      if (lookahead == '"') ADVANCE(150);
      if (lookahead == '\\') ADVANCE(144);
      if (lookahead != 0) ADVANCE(3);
      END_STATE();
    case 4:
      if (lookahead == '"') ADVANCE(5);
      if (lookahead == '#') ADVANCE(147);
      if (lookahead == '\\') ADVANCE(156);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(155);
      if (lookahead != 0) ADVANCE(154);
      END_STATE();
    case 5:
      if (lookahead == '"') ADVANCE(157);
      END_STATE();
    case 6:
      if (lookahead == '\'') ADVANCE(152);
      if (lookahead == '\\') ADVANCE(142);
      if (lookahead != 0) ADVANCE(6);
      END_STATE();
    case 7:
      if (lookahead == '-') ADVANCE(42);
      END_STATE();
    case 8:
      if (lookahead == '>') ADVANCE(176);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(149);
      END_STATE();
    case 9:
      if (lookahead == 'a') ADVANCE(80);
      if (lookahead == 'l') ADVANCE(10);
      END_STATE();
    case 10:
      if (lookahead == 'a') ADVANCE(57);
      END_STATE();
    case 11:
      if (lookahead == 'a') ADVANCE(19);
      END_STATE();
    case 12:
      if (lookahead == 'a') ADVANCE(86);
      if (lookahead == 'o') ADVANCE(140);
      if (lookahead == 'p') ADVANCE(20);
      END_STATE();
    case 13:
      if (lookahead == 'a') ADVANCE(116);
      END_STATE();
    case 14:
      if (lookahead == 'a') ADVANCE(78);
      END_STATE();
    case 15:
      if (lookahead == 'a') ADVANCE(79);
      END_STATE();
    case 16:
      if (lookahead == 'a') ADVANCE(24);
      if (lookahead == 'q') ADVANCE(138);
      END_STATE();
    case 17:
      if (lookahead == 'a') ADVANCE(130);
      END_STATE();
    case 18:
      if (lookahead == 'a') ADVANCE(135);
      END_STATE();
    case 19:
      if (lookahead == 'b') ADVANCE(82);
      END_STATE();
    case 20:
      if (lookahead == 'c') ADVANCE(182);
      END_STATE();
    case 21:
      if (lookahead == 'c') ADVANCE(172);
      END_STATE();
    case 22:
      if (lookahead == 'c') ADVANCE(18);
      END_STATE();
    case 23:
      if (lookahead == 'c') ADVANCE(128);
      END_STATE();
    case 24:
      if (lookahead == 'c') ADVANCE(65);
      END_STATE();
    case 25:
      if (lookahead == 'd') ADVANCE(195);
      END_STATE();
    case 26:
      if (lookahead == 'd') ADVANCE(174);
      END_STATE();
    case 27:
      if (lookahead == 'd') ADVANCE(187);
      END_STATE();
    case 28:
      if (lookahead == 'd') ADVANCE(60);
      END_STATE();
    case 29:
      if (lookahead == 'd') ADVANCE(129);
      END_STATE();
    case 30:
      if (lookahead == 'e') ADVANCE(119);
      if (lookahead == 'o') ADVANCE(91);
      END_STATE();
    case 31:
      if (lookahead == 'e') ADVANCE(125);
      if (lookahead == 'o') ADVANCE(22);
      END_STATE();
    case 32:
      if (lookahead == 'e') ADVANCE(16);
      if (lookahead == 'o') ADVANCE(101);
      END_STATE();
    case 33:
      if (lookahead == 'e') ADVANCE(28);
      if (lookahead == 'h') ADVANCE(44);
      if (lookahead == 'i') ADVANCE(29);
      END_STATE();
    case 34:
      if (lookahead == 'e') ADVANCE(190);
      END_STATE();
    case 35:
      if (lookahead == 'e') ADVANCE(171);
      END_STATE();
    case 36:
      if (lookahead == 'e') ADVANCE(159);
      END_STATE();
    case 37:
      if (lookahead == 'e') ADVANCE(160);
      END_STATE();
    case 38:
      if (lookahead == 'e') ADVANCE(180);
      END_STATE();
    case 39:
      if (lookahead == 'e') ADVANCE(184);
      END_STATE();
    case 40:
      if (lookahead == 'e') ADVANCE(196);
      END_STATE();
    case 41:
      if (lookahead == 'e') ADVANCE(178);
      END_STATE();
    case 42:
      if (lookahead == 'e') ADVANCE(55);
      END_STATE();
    case 43:
      if (lookahead == 'e') ADVANCE(83);
      END_STATE();
    case 44:
      if (lookahead == 'e') ADVANCE(88);
      END_STATE();
    case 45:
      if (lookahead == 'e') ADVANCE(126);
      if (lookahead == 'p') ADVANCE(66);
      if (lookahead == 't') ADVANCE(17);
      END_STATE();
    case 46:
      if (lookahead == 'e') ADVANCE(25);
      END_STATE();
    case 47:
      if (lookahead == 'e') ADVANCE(110);
      END_STATE();
    case 48:
      if (lookahead == 'e') ADVANCE(26);
      END_STATE();
    case 49:
      if (lookahead == 'e') ADVANCE(111);
      END_STATE();
    case 50:
      if (lookahead == 'e') ADVANCE(23);
      END_STATE();
    case 51:
      if (lookahead == 'e') ADVANCE(27);
      END_STATE();
    case 52:
      if (lookahead == 'e') ADVANCE(114);
      END_STATE();
    case 53:
      if (lookahead == 'e') ADVANCE(123);
      END_STATE();
    case 54:
      if (lookahead == 'e') ADVANCE(134);
      END_STATE();
    case 55:
      if (lookahead == 'f') ADVANCE(56);
      END_STATE();
    case 56:
      if (lookahead == 'f') ADVANCE(50);
      END_STATE();
    case 57:
      if (lookahead == 'g') ADVANCE(193);
      END_STATE();
    case 58:
      if (lookahead == 'g') ADVANCE(194);
      END_STATE();
    case 59:
      if (lookahead == 'g') ADVANCE(113);
      END_STATE();
    case 60:
      if (lookahead == 'g') ADVANCE(38);
      END_STATE();
    case 61:
      if (lookahead == 'g') ADVANCE(49);
      END_STATE();
    case 62:
      if (lookahead == 'g') ADVANCE(61);
      END_STATE();
    case 63:
      if (lookahead == 'h') ADVANCE(181);
      END_STATE();
    case 64:
      if (lookahead == 'h') ADVANCE(52);
      END_STATE();
    case 65:
      if (lookahead == 'h') ADVANCE(46);
      END_STATE();
    case 66:
      if (lookahead == 'i') ADVANCE(95);
      END_STATE();
    case 67:
      if (lookahead == 'i') ADVANCE(121);
      END_STATE();
    case 68:
      if (lookahead == 'i') ADVANCE(62);
      if (lookahead == 'u') ADVANCE(36);
      END_STATE();
    case 69:
      if (lookahead == 'i') ADVANCE(102);
      END_STATE();
    case 70:
      if (lookahead == 'i') ADVANCE(124);
      END_STATE();
    case 71:
      if (lookahead == 'i') ADVANCE(92);
      END_STATE();
    case 72:
      if (lookahead == 'i') ADVANCE(127);
      END_STATE();
    case 73:
      if (lookahead == 'i') ADVANCE(133);
      END_STATE();
    case 74:
      if (lookahead == 'i') ADVANCE(103);
      END_STATE();
    case 75:
      if (lookahead == 'i') ADVANCE(115);
      END_STATE();
    case 76:
      if (lookahead == 'i') ADVANCE(104);
      END_STATE();
    case 77:
      if (lookahead == 'i') ADVANCE(108);
      END_STATE();
    case 78:
      if (lookahead == 'l') ADVANCE(185);
      END_STATE();
    case 79:
      if (lookahead == 'l') ADVANCE(188);
      END_STATE();
    case 80:
      if (lookahead == 'l') ADVANCE(122);
      END_STATE();
    case 81:
      if (lookahead == 'l') ADVANCE(54);
      END_STATE();
    case 82:
      if (lookahead == 'l') ADVANCE(41);
      END_STATE();
    case 83:
      if (lookahead == 'm') ADVANCE(177);
      END_STATE();
    case 84:
      if (lookahead == 'm') ADVANCE(170);
      END_STATE();
    case 85:
      if (lookahead == 'm') ADVANCE(107);
      END_STATE();
    case 86:
      if (lookahead == 'm') ADVANCE(35);
      END_STATE();
    case 87:
      if (lookahead == 'n') ADVANCE(197);
      if (lookahead == 't') ADVANCE(43);
      END_STATE();
    case 88:
      if (lookahead == 'n') ADVANCE(191);
      END_STATE();
    case 89:
      if (lookahead == 'n') ADVANCE(183);
      END_STATE();
    case 90:
      if (lookahead == 'n') ADVANCE(173);
      END_STATE();
    case 91:
      if (lookahead == 'n') ADVANCE(34);
      END_STATE();
    case 92:
      if (lookahead == 'n') ADVANCE(58);
      END_STATE();
    case 93:
      if (lookahead == 'n') ADVANCE(15);
      END_STATE();
    case 94:
      if (lookahead == 'n') ADVANCE(47);
      END_STATE();
    case 95:
      if (lookahead == 'n') ADVANCE(94);
      END_STATE();
    case 96:
      if (lookahead == 'o') ADVANCE(85);
      END_STATE();
    case 97:
      if (lookahead == 'o') ADVANCE(137);
      END_STATE();
    case 98:
      if (lookahead == 'o') ADVANCE(112);
      if (lookahead == 'r') ADVANCE(99);
      END_STATE();
    case 99:
      if (lookahead == 'o') ADVANCE(59);
      END_STATE();
    case 100:
      if (lookahead == 'o') ADVANCE(14);
      if (lookahead == 'r') ADVANCE(97);
      END_STATE();
    case 101:
      if (lookahead == 'o') ADVANCE(84);
      END_STATE();
    case 102:
      if (lookahead == 'o') ADVANCE(93);
      END_STATE();
    case 103:
      if (lookahead == 'o') ADVANCE(89);
      END_STATE();
    case 104:
      if (lookahead == 'o') ADVANCE(90);
      END_STATE();
    case 105:
      if (lookahead == 'p') ADVANCE(186);
      END_STATE();
    case 106:
      if (lookahead == 'p') ADVANCE(131);
      END_STATE();
    case 107:
      if (lookahead == 'p') ADVANCE(81);
      END_STATE();
    case 108:
      if (lookahead == 'p') ADVANCE(136);
      END_STATE();
    case 109:
      if (lookahead == 'r') ADVANCE(68);
      END_STATE();
    case 110:
      if (lookahead == 'r') ADVANCE(179);
      END_STATE();
    case 111:
      if (lookahead == 'r') ADVANCE(167);
      END_STATE();
    case 112:
      if (lookahead == 'r') ADVANCE(132);
      END_STATE();
    case 113:
      if (lookahead == 'r') ADVANCE(53);
      END_STATE();
    case 114:
      if (lookahead == 'r') ADVANCE(39);
      END_STATE();
    case 115:
      if (lookahead == 'r') ADVANCE(51);
      END_STATE();
    case 116:
      if (lookahead == 's') ADVANCE(192);
      END_STATE();
    case 117:
      if (lookahead == 's') ADVANCE(7);
      END_STATE();
    case 118:
      if (lookahead == 's') ADVANCE(198);
      END_STATE();
    case 119:
      if (lookahead == 's') ADVANCE(21);
      END_STATE();
    case 120:
      if (lookahead == 's') ADVANCE(71);
      END_STATE();
    case 121:
      if (lookahead == 's') ADVANCE(120);
      END_STATE();
    case 122:
      if (lookahead == 's') ADVANCE(37);
      END_STATE();
    case 123:
      if (lookahead == 's') ADVANCE(118);
      END_STATE();
    case 124:
      if (lookahead == 's') ADVANCE(73);
      END_STATE();
    case 125:
      if (lookahead == 't') ADVANCE(161);
      END_STATE();
    case 126:
      if (lookahead == 't') ADVANCE(162);
      END_STATE();
    case 127:
      if (lookahead == 't') ADVANCE(175);
      END_STATE();
    case 128:
      if (lookahead == 't') ADVANCE(189);
      END_STATE();
    case 129:
      if (lookahead == 't') ADVANCE(63);
      END_STATE();
    case 130:
      if (lookahead == 't') ADVANCE(139);
      END_STATE();
    case 131:
      if (lookahead == 't') ADVANCE(69);
      END_STATE();
    case 132:
      if (lookahead == 't') ADVANCE(11);
      END_STATE();
    case 133:
      if (lookahead == 't') ADVANCE(48);
      END_STATE();
    case 134:
      if (lookahead == 't') ADVANCE(40);
      END_STATE();
    case 135:
      if (lookahead == 't') ADVANCE(74);
      END_STATE();
    case 136:
      if (lookahead == 't') ADVANCE(76);
      END_STATE();
    case 137:
      if (lookahead == 'u') ADVANCE(105);
      END_STATE();
    case 138:
      if (lookahead == 'u') ADVANCE(75);
      END_STATE();
    case 139:
      if (lookahead == 'u') ADVANCE(117);
      END_STATE();
    case 140:
      if (lookahead == 'w') ADVANCE(64);
      END_STATE();
    case 141:
      if (lookahead == 'x') ADVANCE(72);
      END_STATE();
    case 142:
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(6);
      END_STATE();
    case 143:
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(158);
      END_STATE();
    case 144:
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(3);
      END_STATE();
    case 145:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 146:
      ACCEPT_TOKEN(sym_comment);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(146);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(147);
      END_STATE();
    case 147:
      ACCEPT_TOKEN(sym_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(147);
      END_STATE();
    case 148:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '#' ||
          lookahead == '-' ||
          ('0' <= lookahead && lookahead <= ':') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(148);
      END_STATE();
    case 149:
      ACCEPT_TOKEN(sym_number);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(149);
      END_STATE();
    case 150:
      ACCEPT_TOKEN(aux_sym_string_token1);
      END_STATE();
    case 151:
      ACCEPT_TOKEN(aux_sym_string_token1);
      if (lookahead == '"') ADVANCE(153);
      END_STATE();
    case 152:
      ACCEPT_TOKEN(aux_sym_string_token2);
      END_STATE();
    case 153:
      ACCEPT_TOKEN(anon_sym_DQUOTE_DQUOTE_DQUOTE);
      END_STATE();
    case 154:
      ACCEPT_TOKEN(aux_sym_string_token3);
      END_STATE();
    case 155:
      ACCEPT_TOKEN(aux_sym_string_token3);
      if (lookahead == '#') ADVANCE(147);
      if (lookahead == '\\') ADVANCE(156);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(155);
      if (lookahead != 0 &&
          lookahead != '"') ADVANCE(154);
      END_STATE();
    case 156:
      ACCEPT_TOKEN(aux_sym_string_token3);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(158);
      END_STATE();
    case 157:
      ACCEPT_TOKEN(aux_sym_string_token4);
      if (lookahead == '"') ADVANCE(153);
      END_STATE();
    case 158:
      ACCEPT_TOKEN(aux_sym_string_token5);
      END_STATE();
    case 159:
      ACCEPT_TOKEN(anon_sym_true);
      END_STATE();
    case 160:
      ACCEPT_TOKEN(anon_sym_false);
      END_STATE();
    case 161:
      ACCEPT_TOKEN(anon_sym_let);
      END_STATE();
    case 162:
      ACCEPT_TOKEN(anon_sym_set);
      END_STATE();
    case 163:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 164:
      ACCEPT_TOKEN(anon_sym_LPAREN);
      END_STATE();
    case 165:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 166:
      ACCEPT_TOKEN(anon_sym_RPAREN);
      END_STATE();
    case 167:
      ACCEPT_TOKEN(anon_sym_trigger);
      END_STATE();
    case 168:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 169:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 170:
      ACCEPT_TOKEN(anon_sym_room);
      END_STATE();
    case 171:
      ACCEPT_TOKEN(anon_sym_name);
      END_STATE();
    case 172:
      ACCEPT_TOKEN(anon_sym_desc);
      if (lookahead == 'r') ADVANCE(77);
      END_STATE();
    case 173:
      ACCEPT_TOKEN(anon_sym_description);
      END_STATE();
    case 174:
      ACCEPT_TOKEN(anon_sym_visited);
      END_STATE();
    case 175:
      ACCEPT_TOKEN(anon_sym_exit);
      END_STATE();
    case 176:
      ACCEPT_TOKEN(anon_sym_DASH_GT);
      END_STATE();
    case 177:
      ACCEPT_TOKEN(anon_sym_item);
      END_STATE();
    case 178:
      ACCEPT_TOKEN(anon_sym_portable);
      END_STATE();
    case 179:
      ACCEPT_TOKEN(anon_sym_spinner);
      END_STATE();
    case 180:
      ACCEPT_TOKEN(anon_sym_wedge);
      END_STATE();
    case 181:
      ACCEPT_TOKEN(anon_sym_width);
      END_STATE();
    case 182:
      ACCEPT_TOKEN(anon_sym_npc);
      END_STATE();
    case 183:
      ACCEPT_TOKEN(anon_sym_location);
      END_STATE();
    case 184:
      ACCEPT_TOKEN(anon_sym_nowhere);
      END_STATE();
    case 185:
      ACCEPT_TOKEN(anon_sym_goal);
      END_STATE();
    case 186:
      ACCEPT_TOKEN(anon_sym_group);
      END_STATE();
    case 187:
      ACCEPT_TOKEN(anon_sym_required);
      END_STATE();
    case 188:
      ACCEPT_TOKEN(anon_sym_optional);
      END_STATE();
    case 189:
      ACCEPT_TOKEN(anon_sym_status_DASHeffect);
      END_STATE();
    case 190:
      ACCEPT_TOKEN(anon_sym_done);
      END_STATE();
    case 191:
      ACCEPT_TOKEN(anon_sym_when);
      END_STATE();
    case 192:
      ACCEPT_TOKEN(anon_sym_has);
      END_STATE();
    case 193:
      ACCEPT_TOKEN(anon_sym_flag);
      END_STATE();
    case 194:
      ACCEPT_TOKEN(anon_sym_missing);
      END_STATE();
    case 195:
      ACCEPT_TOKEN(anon_sym_reached);
      END_STATE();
    case 196:
      ACCEPT_TOKEN(anon_sym_complete);
      END_STATE();
    case 197:
      ACCEPT_TOKEN(anon_sym_in);
      END_STATE();
    case 198:
      ACCEPT_TOKEN(anon_sym_progress);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 0},
  [2] = {.lex_state = 0},
  [3] = {.lex_state = 0},
  [4] = {.lex_state = 0},
  [5] = {.lex_state = 0},
  [6] = {.lex_state = 0},
  [7] = {.lex_state = 0},
  [8] = {.lex_state = 0},
  [9] = {.lex_state = 0},
  [10] = {.lex_state = 0},
  [11] = {.lex_state = 0},
  [12] = {.lex_state = 0},
  [13] = {.lex_state = 0},
  [14] = {.lex_state = 0},
  [15] = {.lex_state = 0},
  [16] = {.lex_state = 0},
  [17] = {.lex_state = 0},
  [18] = {.lex_state = 0},
  [19] = {.lex_state = 0},
  [20] = {.lex_state = 0},
  [21] = {.lex_state = 0},
  [22] = {.lex_state = 0},
  [23] = {.lex_state = 0},
  [24] = {.lex_state = 0},
  [25] = {.lex_state = 0},
  [26] = {.lex_state = 0},
  [27] = {.lex_state = 0},
  [28] = {.lex_state = 0},
  [29] = {.lex_state = 0},
  [30] = {.lex_state = 0},
  [31] = {.lex_state = 0},
  [32] = {.lex_state = 0},
  [33] = {.lex_state = 0},
  [34] = {.lex_state = 0},
  [35] = {.lex_state = 0},
  [36] = {.lex_state = 0},
  [37] = {.lex_state = 0},
  [38] = {.lex_state = 0},
  [39] = {.lex_state = 0},
  [40] = {.lex_state = 0},
  [41] = {.lex_state = 0},
  [42] = {.lex_state = 0},
  [43] = {.lex_state = 0},
  [44] = {.lex_state = 0},
  [45] = {.lex_state = 0},
  [46] = {.lex_state = 0},
  [47] = {.lex_state = 0},
  [48] = {.lex_state = 0},
  [49] = {.lex_state = 4},
  [50] = {.lex_state = 0},
  [51] = {.lex_state = 4},
  [52] = {.lex_state = 0},
  [53] = {.lex_state = 0},
  [54] = {.lex_state = 0},
  [55] = {.lex_state = 0},
  [56] = {.lex_state = 0},
  [57] = {.lex_state = 4},
  [58] = {.lex_state = 0},
  [59] = {.lex_state = 4},
  [60] = {.lex_state = 1},
  [61] = {.lex_state = 0},
  [62] = {.lex_state = 4},
  [63] = {.lex_state = 1},
  [64] = {.lex_state = 1},
  [65] = {.lex_state = 1},
  [66] = {.lex_state = 1},
  [67] = {.lex_state = 1},
  [68] = {.lex_state = 1},
  [69] = {.lex_state = 1},
  [70] = {.lex_state = 1},
  [71] = {.lex_state = 1},
  [72] = {.lex_state = 1},
  [73] = {.lex_state = 1},
  [74] = {.lex_state = 1},
  [75] = {.lex_state = 0},
  [76] = {.lex_state = 0},
  [77] = {.lex_state = 1},
  [78] = {.lex_state = 1},
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
  [93] = {.lex_state = 1},
  [94] = {.lex_state = 0},
  [95] = {.lex_state = 1},
  [96] = {.lex_state = 1},
  [97] = {.lex_state = 0},
  [98] = {.lex_state = 0},
  [99] = {.lex_state = 1},
  [100] = {.lex_state = 0},
  [101] = {.lex_state = 1},
  [102] = {.lex_state = 1},
  [103] = {.lex_state = 0},
  [104] = {.lex_state = 0},
  [105] = {.lex_state = 0},
  [106] = {.lex_state = 0},
  [107] = {.lex_state = 1},
  [108] = {.lex_state = 1},
  [109] = {.lex_state = 1},
  [110] = {.lex_state = 1},
  [111] = {.lex_state = 0},
  [112] = {.lex_state = 0},
  [113] = {.lex_state = 0},
  [114] = {.lex_state = 1},
  [115] = {.lex_state = 1},
  [116] = {.lex_state = 1},
  [117] = {.lex_state = 1},
  [118] = {.lex_state = 1},
  [119] = {.lex_state = 1},
  [120] = {.lex_state = 0},
  [121] = {.lex_state = 1},
  [122] = {.lex_state = 0},
  [123] = {.lex_state = 0},
  [124] = {.lex_state = 0},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [sym_comment] = ACTIONS(3),
    [sym_number] = ACTIONS(1),
    [aux_sym_string_token1] = ACTIONS(1),
    [aux_sym_string_token2] = ACTIONS(1),
    [anon_sym_DQUOTE_DQUOTE_DQUOTE] = ACTIONS(1),
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
    [anon_sym_LBRACE] = ACTIONS(1),
    [anon_sym_RBRACE] = ACTIONS(1),
    [anon_sym_room] = ACTIONS(1),
    [anon_sym_name] = ACTIONS(1),
    [anon_sym_desc] = ACTIONS(1),
    [anon_sym_description] = ACTIONS(1),
    [anon_sym_visited] = ACTIONS(1),
    [anon_sym_exit] = ACTIONS(1),
    [anon_sym_DASH_GT] = ACTIONS(1),
    [anon_sym_item] = ACTIONS(1),
    [anon_sym_portable] = ACTIONS(1),
    [anon_sym_spinner] = ACTIONS(1),
    [anon_sym_wedge] = ACTIONS(1),
    [anon_sym_width] = ACTIONS(1),
    [anon_sym_npc] = ACTIONS(1),
    [anon_sym_location] = ACTIONS(1),
    [anon_sym_nowhere] = ACTIONS(1),
    [anon_sym_goal] = ACTIONS(1),
    [anon_sym_group] = ACTIONS(1),
    [anon_sym_required] = ACTIONS(1),
    [anon_sym_optional] = ACTIONS(1),
    [anon_sym_status_DASHeffect] = ACTIONS(1),
    [anon_sym_done] = ACTIONS(1),
    [anon_sym_when] = ACTIONS(1),
    [anon_sym_has] = ACTIONS(1),
    [anon_sym_flag] = ACTIONS(1),
    [anon_sym_missing] = ACTIONS(1),
    [anon_sym_reached] = ACTIONS(1),
    [anon_sym_complete] = ACTIONS(1),
    [anon_sym_in] = ACTIONS(1),
    [anon_sym_progress] = ACTIONS(1),
  },
  [1] = {
    [sym_program] = STATE(120),
    [sym_set_decl] = STATE(8),
    [sym_trigger] = STATE(8),
    [sym_room_def] = STATE(8),
    [sym_item_def] = STATE(8),
    [sym_spinner_def] = STATE(8),
    [sym_npc_def] = STATE(8),
    [sym_goal_def] = STATE(8),
    [aux_sym_program_repeat1] = STATE(8),
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
    ACTIONS(23), 1,
      anon_sym_desc,
    ACTIONS(21), 20,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_RBRACE,
      anon_sym_room,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_exit,
      anon_sym_DASH_GT,
      anon_sym_item,
      anon_sym_portable,
      anon_sym_spinner,
      anon_sym_wedge,
      anon_sym_width,
      anon_sym_npc,
      anon_sym_location,
      anon_sym_goal,
      anon_sym_group,
      anon_sym_done,
  [29] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(27), 1,
      anon_sym_desc,
    ACTIONS(25), 20,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_RBRACE,
      anon_sym_room,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_exit,
      anon_sym_DASH_GT,
      anon_sym_item,
      anon_sym_portable,
      anon_sym_spinner,
      anon_sym_wedge,
      anon_sym_width,
      anon_sym_npc,
      anon_sym_location,
      anon_sym_goal,
      anon_sym_group,
      anon_sym_done,
  [58] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(31), 1,
      anon_sym_desc,
    ACTIONS(29), 20,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_RBRACE,
      anon_sym_room,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_exit,
      anon_sym_DASH_GT,
      anon_sym_item,
      anon_sym_portable,
      anon_sym_spinner,
      anon_sym_wedge,
      anon_sym_width,
      anon_sym_npc,
      anon_sym_location,
      anon_sym_goal,
      anon_sym_group,
      anon_sym_done,
  [87] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(35), 1,
      anon_sym_desc,
    ACTIONS(37), 1,
      anon_sym_group,
    ACTIONS(39), 1,
      anon_sym_done,
    STATE(9), 2,
      sym_goal_stmt,
      aux_sym_goal_def_repeat1,
    STATE(16), 3,
      sym_goal_desc,
      sym_goal_group,
      sym_goal_done,
    ACTIONS(33), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [119] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(43), 1,
      anon_sym_desc,
    ACTIONS(46), 1,
      anon_sym_group,
    ACTIONS(49), 1,
      anon_sym_done,
    STATE(6), 2,
      sym_goal_stmt,
      aux_sym_goal_def_repeat1,
    STATE(16), 3,
      sym_goal_desc,
      sym_goal_group,
      sym_goal_done,
    ACTIONS(41), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [151] = 10,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(52), 1,
      ts_builtin_sym_end,
    ACTIONS(54), 1,
      anon_sym_let,
    ACTIONS(57), 1,
      anon_sym_trigger,
    ACTIONS(60), 1,
      anon_sym_room,
    ACTIONS(63), 1,
      anon_sym_item,
    ACTIONS(66), 1,
      anon_sym_spinner,
    ACTIONS(69), 1,
      anon_sym_npc,
    ACTIONS(72), 1,
      anon_sym_goal,
    STATE(7), 8,
      sym_set_decl,
      sym_trigger,
      sym_room_def,
      sym_item_def,
      sym_spinner_def,
      sym_npc_def,
      sym_goal_def,
      aux_sym_program_repeat1,
  [189] = 10,
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
    ACTIONS(75), 1,
      ts_builtin_sym_end,
    STATE(7), 8,
      sym_set_decl,
      sym_trigger,
      sym_room_def,
      sym_item_def,
      sym_spinner_def,
      sym_npc_def,
      sym_goal_def,
      aux_sym_program_repeat1,
  [227] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(35), 1,
      anon_sym_desc,
    ACTIONS(37), 1,
      anon_sym_group,
    ACTIONS(39), 1,
      anon_sym_done,
    STATE(6), 2,
      sym_goal_stmt,
      aux_sym_goal_def_repeat1,
    STATE(16), 3,
      sym_goal_desc,
      sym_goal_group,
      sym_goal_done,
    ACTIONS(77), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [259] = 9,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(79), 1,
      anon_sym_RBRACE,
    ACTIONS(81), 1,
      anon_sym_name,
    ACTIONS(83), 1,
      anon_sym_desc,
    ACTIONS(85), 1,
      anon_sym_description,
    ACTIONS(87), 1,
      anon_sym_visited,
    ACTIONS(89), 1,
      anon_sym_exit,
    STATE(12), 2,
      sym_room_stmt,
      aux_sym_room_block_repeat1,
    STATE(43), 4,
      sym_room_name,
      sym_room_desc,
      sym_room_visited,
      sym_exit_stmt,
  [291] = 9,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(91), 1,
      anon_sym_RBRACE,
    ACTIONS(93), 1,
      anon_sym_name,
    ACTIONS(96), 1,
      anon_sym_desc,
    ACTIONS(99), 1,
      anon_sym_description,
    ACTIONS(102), 1,
      anon_sym_visited,
    ACTIONS(105), 1,
      anon_sym_exit,
    STATE(11), 2,
      sym_room_stmt,
      aux_sym_room_block_repeat1,
    STATE(43), 4,
      sym_room_name,
      sym_room_desc,
      sym_room_visited,
      sym_exit_stmt,
  [323] = 9,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(81), 1,
      anon_sym_name,
    ACTIONS(83), 1,
      anon_sym_desc,
    ACTIONS(85), 1,
      anon_sym_description,
    ACTIONS(87), 1,
      anon_sym_visited,
    ACTIONS(89), 1,
      anon_sym_exit,
    ACTIONS(108), 1,
      anon_sym_RBRACE,
    STATE(11), 2,
      sym_room_stmt,
      aux_sym_room_block_repeat1,
    STATE(43), 4,
      sym_room_name,
      sym_room_desc,
      sym_room_visited,
      sym_exit_stmt,
  [355] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(110), 11,
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
  [372] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(112), 11,
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
  [389] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(114), 11,
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
  [406] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(116), 11,
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
  [423] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(118), 11,
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
  [440] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(120), 11,
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
  [457] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(122), 1,
      anon_sym_RBRACE,
    ACTIONS(124), 1,
      anon_sym_name,
    ACTIONS(127), 1,
      anon_sym_desc,
    ACTIONS(130), 1,
      anon_sym_description,
    ACTIONS(133), 1,
      anon_sym_location,
    STATE(19), 2,
      sym_npc_stmt,
      aux_sym_npc_block_repeat1,
    STATE(58), 3,
      sym_npc_name,
      sym_npc_desc,
      sym_npc_location,
  [485] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(136), 1,
      anon_sym_RBRACE,
    ACTIONS(138), 1,
      anon_sym_name,
    ACTIONS(140), 1,
      anon_sym_desc,
    ACTIONS(142), 1,
      anon_sym_description,
    ACTIONS(144), 1,
      anon_sym_location,
    STATE(22), 2,
      sym_npc_stmt,
      aux_sym_npc_block_repeat1,
    STATE(58), 3,
      sym_npc_name,
      sym_npc_desc,
      sym_npc_location,
  [513] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(146), 1,
      anon_sym_RBRACE,
    ACTIONS(148), 1,
      anon_sym_name,
    ACTIONS(151), 1,
      anon_sym_desc,
    ACTIONS(154), 1,
      anon_sym_description,
    ACTIONS(157), 1,
      anon_sym_portable,
    STATE(21), 2,
      sym_item_stmt,
      aux_sym_item_block_repeat1,
    STATE(52), 3,
      sym_item_name,
      sym_item_desc,
      sym_item_portable,
  [541] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(138), 1,
      anon_sym_name,
    ACTIONS(140), 1,
      anon_sym_desc,
    ACTIONS(142), 1,
      anon_sym_description,
    ACTIONS(144), 1,
      anon_sym_location,
    ACTIONS(160), 1,
      anon_sym_RBRACE,
    STATE(19), 2,
      sym_npc_stmt,
      aux_sym_npc_block_repeat1,
    STATE(58), 3,
      sym_npc_name,
      sym_npc_desc,
      sym_npc_location,
  [569] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(162), 1,
      anon_sym_RBRACE,
    ACTIONS(164), 1,
      anon_sym_name,
    ACTIONS(166), 1,
      anon_sym_desc,
    ACTIONS(168), 1,
      anon_sym_description,
    ACTIONS(170), 1,
      anon_sym_portable,
    STATE(21), 2,
      sym_item_stmt,
      aux_sym_item_block_repeat1,
    STATE(52), 3,
      sym_item_name,
      sym_item_desc,
      sym_item_portable,
  [597] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(164), 1,
      anon_sym_name,
    ACTIONS(166), 1,
      anon_sym_desc,
    ACTIONS(168), 1,
      anon_sym_description,
    ACTIONS(170), 1,
      anon_sym_portable,
    ACTIONS(172), 1,
      anon_sym_RBRACE,
    STATE(23), 2,
      sym_item_stmt,
      aux_sym_item_block_repeat1,
    STATE(52), 3,
      sym_item_name,
      sym_item_desc,
      sym_item_portable,
  [625] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(174), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [639] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(176), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [653] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(178), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [667] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(180), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [681] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(182), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [695] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(184), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [709] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(186), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [723] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(188), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [737] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(190), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [751] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(192), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [765] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(194), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [779] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(196), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [793] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(198), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [807] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(200), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [821] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(202), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [835] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(204), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [849] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(206), 8,
      ts_builtin_sym_end,
      anon_sym_let,
      anon_sym_trigger,
      anon_sym_room,
      anon_sym_item,
      anon_sym_spinner,
      anon_sym_npc,
      anon_sym_goal,
  [863] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(210), 1,
      anon_sym_desc,
    ACTIONS(208), 6,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_exit,
      anon_sym_portable,
  [878] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(214), 1,
      anon_sym_desc,
    ACTIONS(212), 5,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_exit,
  [892] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(218), 1,
      anon_sym_desc,
    ACTIONS(216), 5,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_exit,
  [906] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(222), 1,
      anon_sym_desc,
    ACTIONS(220), 5,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_exit,
  [920] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(224), 1,
      anon_sym_goal,
    ACTIONS(226), 1,
      anon_sym_has,
    ACTIONS(228), 1,
      anon_sym_flag,
    ACTIONS(230), 1,
      anon_sym_missing,
    ACTIONS(232), 1,
      anon_sym_reached,
    STATE(17), 1,
      sym_goal_cond,
  [942] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(236), 1,
      anon_sym_desc,
    ACTIONS(234), 5,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_exit,
  [956] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(240), 1,
      anon_sym_desc,
    ACTIONS(238), 5,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_visited,
      anon_sym_exit,
  [970] = 4,
    ACTIONS(242), 1,
      sym_comment,
    ACTIONS(244), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(59), 1,
      aux_sym_string_repeat1,
    ACTIONS(246), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [985] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(250), 1,
      anon_sym_desc,
    ACTIONS(248), 4,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_location,
  [998] = 4,
    ACTIONS(242), 1,
      sym_comment,
    ACTIONS(252), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(62), 1,
      aux_sym_string_repeat1,
    ACTIONS(254), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [1013] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(258), 1,
      anon_sym_desc,
    ACTIONS(256), 4,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_portable,
  [1026] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(262), 1,
      anon_sym_desc,
    ACTIONS(260), 4,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_portable,
  [1039] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(266), 1,
      anon_sym_desc,
    ACTIONS(264), 4,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_portable,
  [1052] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(270), 1,
      anon_sym_desc,
    ACTIONS(268), 4,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_portable,
  [1065] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(274), 1,
      anon_sym_desc,
    ACTIONS(272), 4,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_location,
  [1078] = 4,
    ACTIONS(242), 1,
      sym_comment,
    ACTIONS(276), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(51), 1,
      aux_sym_string_repeat1,
    ACTIONS(278), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [1093] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(282), 1,
      anon_sym_desc,
    ACTIONS(280), 4,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_location,
  [1106] = 4,
    ACTIONS(242), 1,
      sym_comment,
    ACTIONS(284), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(62), 1,
      aux_sym_string_repeat1,
    ACTIONS(254), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [1121] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(286), 1,
      sym_identifier,
    ACTIONS(288), 1,
      aux_sym_string_token1,
    ACTIONS(290), 1,
      aux_sym_string_token2,
    ACTIONS(292), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(123), 1,
      sym_string,
  [1140] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(296), 1,
      anon_sym_desc,
    ACTIONS(294), 4,
      anon_sym_RBRACE,
      anon_sym_name,
      anon_sym_description,
      anon_sym_location,
  [1153] = 4,
    ACTIONS(242), 1,
      sym_comment,
    ACTIONS(298), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(62), 1,
      aux_sym_string_repeat1,
    ACTIONS(300), 3,
      aux_sym_string_token3,
      aux_sym_string_token4,
      aux_sym_string_token5,
  [1168] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(288), 1,
      aux_sym_string_token1,
    ACTIONS(290), 1,
      aux_sym_string_token2,
    ACTIONS(292), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(55), 1,
      sym_string,
  [1184] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(288), 1,
      aux_sym_string_token1,
    ACTIONS(290), 1,
      aux_sym_string_token2,
    ACTIONS(292), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(61), 1,
      sym_string,
  [1200] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(303), 1,
      sym_identifier,
    ACTIONS(305), 1,
      anon_sym_RBRACE,
    STATE(72), 2,
      sym__trigger_stmt,
      aux_sym_trigger_repeat1,
  [1214] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(288), 1,
      aux_sym_string_token1,
    ACTIONS(290), 1,
      aux_sym_string_token2,
    ACTIONS(292), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(86), 1,
      sym_string,
  [1230] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(288), 1,
      aux_sym_string_token1,
    ACTIONS(290), 1,
      aux_sym_string_token2,
    ACTIONS(292), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(13), 1,
      sym_string,
  [1246] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(288), 1,
      aux_sym_string_token1,
    ACTIONS(290), 1,
      aux_sym_string_token2,
    ACTIONS(292), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(54), 1,
      sym_string,
  [1262] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(307), 1,
      sym_identifier,
    ACTIONS(310), 1,
      anon_sym_RBRACE,
    STATE(69), 2,
      sym__trigger_stmt,
      aux_sym_trigger_repeat1,
  [1276] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(288), 1,
      aux_sym_string_token1,
    ACTIONS(290), 1,
      aux_sym_string_token2,
    ACTIONS(292), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(45), 1,
      sym_string,
  [1292] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(288), 1,
      aux_sym_string_token1,
    ACTIONS(290), 1,
      aux_sym_string_token2,
    ACTIONS(292), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(44), 1,
      sym_string,
  [1308] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(303), 1,
      sym_identifier,
    ACTIONS(312), 1,
      anon_sym_RBRACE,
    STATE(69), 2,
      sym__trigger_stmt,
      aux_sym_trigger_repeat1,
  [1322] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(314), 1,
      aux_sym_string_token1,
    ACTIONS(316), 1,
      aux_sym_string_token2,
    ACTIONS(318), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(99), 1,
      sym_string,
  [1338] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(288), 1,
      aux_sym_string_token1,
    ACTIONS(290), 1,
      aux_sym_string_token2,
    ACTIONS(292), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(5), 1,
      sym_string,
  [1354] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(320), 1,
      anon_sym_RBRACE,
    ACTIONS(322), 1,
      anon_sym_wedge,
    STATE(75), 2,
      sym_wedge_stmt,
      aux_sym_spinner_block_repeat1,
  [1368] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(325), 1,
      anon_sym_RBRACE,
    ACTIONS(327), 1,
      anon_sym_wedge,
    STATE(79), 2,
      sym_wedge_stmt,
      aux_sym_spinner_block_repeat1,
  [1382] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(288), 1,
      aux_sym_string_token1,
    ACTIONS(290), 1,
      aux_sym_string_token2,
    ACTIONS(292), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(56), 1,
      sym_string,
  [1398] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(288), 1,
      aux_sym_string_token1,
    ACTIONS(290), 1,
      aux_sym_string_token2,
    ACTIONS(292), 1,
      anon_sym_DQUOTE_DQUOTE_DQUOTE,
    STATE(50), 1,
      sym_string,
  [1414] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(327), 1,
      anon_sym_wedge,
    ACTIONS(329), 1,
      anon_sym_RBRACE,
    STATE(75), 2,
      sym_wedge_stmt,
      aux_sym_spinner_block_repeat1,
  [1428] = 3,
    ACTIONS(3), 1,
      sym_comment,
    STATE(47), 1,
      sym_boolean,
    ACTIONS(331), 2,
      anon_sym_true,
      anon_sym_false,
  [1439] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(333), 1,
      anon_sym_COMMA,
    ACTIONS(335), 1,
      anon_sym_RPAREN,
    STATE(84), 1,
      aux_sym_set_list_repeat1,
  [1452] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(337), 3,
      anon_sym_required,
      anon_sym_optional,
      anon_sym_status_DASHeffect,
  [1461] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(339), 1,
      anon_sym_COMMA,
    ACTIONS(342), 1,
      anon_sym_RPAREN,
    STATE(83), 1,
      aux_sym_set_list_repeat1,
  [1474] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(333), 1,
      anon_sym_COMMA,
    ACTIONS(344), 1,
      anon_sym_RPAREN,
    STATE(83), 1,
      aux_sym_set_list_repeat1,
  [1487] = 3,
    ACTIONS(3), 1,
      sym_comment,
    STATE(53), 1,
      sym_boolean,
    ACTIONS(331), 2,
      anon_sym_true,
      anon_sym_false,
  [1498] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(348), 1,
      anon_sym_width,
    ACTIONS(346), 2,
      anon_sym_RBRACE,
      anon_sym_wedge,
  [1509] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(342), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [1517] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(350), 1,
      anon_sym_complete,
    ACTIONS(352), 1,
      anon_sym_in,
  [1527] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(354), 1,
      anon_sym_room,
    ACTIONS(356), 1,
      anon_sym_nowhere,
  [1537] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(358), 1,
      anon_sym_LBRACE,
    STATE(28), 1,
      sym_npc_block,
  [1547] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(360), 1,
      anon_sym_LBRACE,
    STATE(36), 1,
      sym_spinner_block,
  [1557] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(362), 1,
      anon_sym_LBRACE,
    STATE(25), 1,
      sym_item_block,
  [1567] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(21), 1,
      anon_sym_RBRACE,
    ACTIONS(23), 1,
      sym_identifier,
  [1577] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(364), 1,
      anon_sym_LBRACE,
    STATE(31), 1,
      sym_room_block,
  [1587] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(29), 1,
      anon_sym_RBRACE,
    ACTIONS(31), 1,
      sym_identifier,
  [1597] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(25), 1,
      anon_sym_RBRACE,
    ACTIONS(27), 1,
      sym_identifier,
  [1607] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(366), 1,
      anon_sym_LPAREN,
    STATE(39), 1,
      sym_set_list,
  [1617] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(368), 2,
      anon_sym_RBRACE,
      anon_sym_wedge,
  [1625] = 3,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(370), 1,
      sym_identifier,
    ACTIONS(372), 1,
      anon_sym_RBRACE,
  [1635] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(350), 2,
      anon_sym_item,
      anon_sym_flag,
  [1643] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(374), 1,
      sym_identifier,
  [1650] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(376), 1,
      sym_identifier,
  [1657] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(350), 1,
      anon_sym_complete,
  [1664] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(350), 1,
      anon_sym_flag,
  [1671] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(350), 1,
      anon_sym_room,
  [1678] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(378), 1,
      anon_sym_EQ,
  [1685] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(380), 1,
      sym_identifier,
  [1692] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(382), 1,
      sym_identifier,
  [1699] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(384), 1,
      sym_identifier,
  [1706] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(386), 1,
      sym_identifier,
  [1713] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(388), 1,
      sym_number,
  [1720] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(390), 1,
      anon_sym_when,
  [1727] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(392), 1,
      anon_sym_progress,
  [1734] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(394), 1,
      sym_identifier,
  [1741] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(396), 1,
      sym_identifier,
  [1748] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(398), 1,
      sym_identifier,
  [1755] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(400), 1,
      sym_identifier,
  [1762] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(402), 1,
      sym_identifier,
  [1769] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(404), 1,
      sym_identifier,
  [1776] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(406), 1,
      ts_builtin_sym_end,
  [1783] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(408), 1,
      sym_identifier,
  [1790] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(410), 1,
      anon_sym_LBRACE,
  [1797] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(412), 1,
      anon_sym_DASH_GT,
  [1804] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(414), 1,
      anon_sym_set,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 29,
  [SMALL_STATE(4)] = 58,
  [SMALL_STATE(5)] = 87,
  [SMALL_STATE(6)] = 119,
  [SMALL_STATE(7)] = 151,
  [SMALL_STATE(8)] = 189,
  [SMALL_STATE(9)] = 227,
  [SMALL_STATE(10)] = 259,
  [SMALL_STATE(11)] = 291,
  [SMALL_STATE(12)] = 323,
  [SMALL_STATE(13)] = 355,
  [SMALL_STATE(14)] = 372,
  [SMALL_STATE(15)] = 389,
  [SMALL_STATE(16)] = 406,
  [SMALL_STATE(17)] = 423,
  [SMALL_STATE(18)] = 440,
  [SMALL_STATE(19)] = 457,
  [SMALL_STATE(20)] = 485,
  [SMALL_STATE(21)] = 513,
  [SMALL_STATE(22)] = 541,
  [SMALL_STATE(23)] = 569,
  [SMALL_STATE(24)] = 597,
  [SMALL_STATE(25)] = 625,
  [SMALL_STATE(26)] = 639,
  [SMALL_STATE(27)] = 653,
  [SMALL_STATE(28)] = 667,
  [SMALL_STATE(29)] = 681,
  [SMALL_STATE(30)] = 695,
  [SMALL_STATE(31)] = 709,
  [SMALL_STATE(32)] = 723,
  [SMALL_STATE(33)] = 737,
  [SMALL_STATE(34)] = 751,
  [SMALL_STATE(35)] = 765,
  [SMALL_STATE(36)] = 779,
  [SMALL_STATE(37)] = 793,
  [SMALL_STATE(38)] = 807,
  [SMALL_STATE(39)] = 821,
  [SMALL_STATE(40)] = 835,
  [SMALL_STATE(41)] = 849,
  [SMALL_STATE(42)] = 863,
  [SMALL_STATE(43)] = 878,
  [SMALL_STATE(44)] = 892,
  [SMALL_STATE(45)] = 906,
  [SMALL_STATE(46)] = 920,
  [SMALL_STATE(47)] = 942,
  [SMALL_STATE(48)] = 956,
  [SMALL_STATE(49)] = 970,
  [SMALL_STATE(50)] = 985,
  [SMALL_STATE(51)] = 998,
  [SMALL_STATE(52)] = 1013,
  [SMALL_STATE(53)] = 1026,
  [SMALL_STATE(54)] = 1039,
  [SMALL_STATE(55)] = 1052,
  [SMALL_STATE(56)] = 1065,
  [SMALL_STATE(57)] = 1078,
  [SMALL_STATE(58)] = 1093,
  [SMALL_STATE(59)] = 1106,
  [SMALL_STATE(60)] = 1121,
  [SMALL_STATE(61)] = 1140,
  [SMALL_STATE(62)] = 1153,
  [SMALL_STATE(63)] = 1168,
  [SMALL_STATE(64)] = 1184,
  [SMALL_STATE(65)] = 1200,
  [SMALL_STATE(66)] = 1214,
  [SMALL_STATE(67)] = 1230,
  [SMALL_STATE(68)] = 1246,
  [SMALL_STATE(69)] = 1262,
  [SMALL_STATE(70)] = 1276,
  [SMALL_STATE(71)] = 1292,
  [SMALL_STATE(72)] = 1308,
  [SMALL_STATE(73)] = 1322,
  [SMALL_STATE(74)] = 1338,
  [SMALL_STATE(75)] = 1354,
  [SMALL_STATE(76)] = 1368,
  [SMALL_STATE(77)] = 1382,
  [SMALL_STATE(78)] = 1398,
  [SMALL_STATE(79)] = 1414,
  [SMALL_STATE(80)] = 1428,
  [SMALL_STATE(81)] = 1439,
  [SMALL_STATE(82)] = 1452,
  [SMALL_STATE(83)] = 1461,
  [SMALL_STATE(84)] = 1474,
  [SMALL_STATE(85)] = 1487,
  [SMALL_STATE(86)] = 1498,
  [SMALL_STATE(87)] = 1509,
  [SMALL_STATE(88)] = 1517,
  [SMALL_STATE(89)] = 1527,
  [SMALL_STATE(90)] = 1537,
  [SMALL_STATE(91)] = 1547,
  [SMALL_STATE(92)] = 1557,
  [SMALL_STATE(93)] = 1567,
  [SMALL_STATE(94)] = 1577,
  [SMALL_STATE(95)] = 1587,
  [SMALL_STATE(96)] = 1597,
  [SMALL_STATE(97)] = 1607,
  [SMALL_STATE(98)] = 1617,
  [SMALL_STATE(99)] = 1625,
  [SMALL_STATE(100)] = 1635,
  [SMALL_STATE(101)] = 1643,
  [SMALL_STATE(102)] = 1650,
  [SMALL_STATE(103)] = 1657,
  [SMALL_STATE(104)] = 1664,
  [SMALL_STATE(105)] = 1671,
  [SMALL_STATE(106)] = 1678,
  [SMALL_STATE(107)] = 1685,
  [SMALL_STATE(108)] = 1692,
  [SMALL_STATE(109)] = 1699,
  [SMALL_STATE(110)] = 1706,
  [SMALL_STATE(111)] = 1713,
  [SMALL_STATE(112)] = 1720,
  [SMALL_STATE(113)] = 1727,
  [SMALL_STATE(114)] = 1734,
  [SMALL_STATE(115)] = 1741,
  [SMALL_STATE(116)] = 1748,
  [SMALL_STATE(117)] = 1755,
  [SMALL_STATE(118)] = 1762,
  [SMALL_STATE(119)] = 1769,
  [SMALL_STATE(120)] = 1776,
  [SMALL_STATE(121)] = 1783,
  [SMALL_STATE(122)] = 1790,
  [SMALL_STATE(123)] = 1797,
  [SMALL_STATE(124)] = 1804,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_program, 0),
  [7] = {.entry = {.count = 1, .reusable = true}}, SHIFT(124),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(119),
  [11] = {.entry = {.count = 1, .reusable = true}}, SHIFT(117),
  [13] = {.entry = {.count = 1, .reusable = true}}, SHIFT(115),
  [15] = {.entry = {.count = 1, .reusable = true}}, SHIFT(110),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(108),
  [19] = {.entry = {.count = 1, .reusable = true}}, SHIFT(101),
  [21] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string, 3),
  [23] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string, 3),
  [25] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string, 1),
  [27] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string, 1),
  [29] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string, 2),
  [31] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string, 2),
  [33] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_def, 3),
  [35] = {.entry = {.count = 1, .reusable = true}}, SHIFT(67),
  [37] = {.entry = {.count = 1, .reusable = true}}, SHIFT(82),
  [39] = {.entry = {.count = 1, .reusable = true}}, SHIFT(112),
  [41] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_goal_def_repeat1, 2),
  [43] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_goal_def_repeat1, 2), SHIFT_REPEAT(67),
  [46] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_goal_def_repeat1, 2), SHIFT_REPEAT(82),
  [49] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_goal_def_repeat1, 2), SHIFT_REPEAT(112),
  [52] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2),
  [54] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2), SHIFT_REPEAT(124),
  [57] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2), SHIFT_REPEAT(119),
  [60] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2), SHIFT_REPEAT(117),
  [63] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2), SHIFT_REPEAT(115),
  [66] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2), SHIFT_REPEAT(110),
  [69] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2), SHIFT_REPEAT(108),
  [72] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_program_repeat1, 2), SHIFT_REPEAT(101),
  [75] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_program, 1),
  [77] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_def, 4),
  [79] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [81] = {.entry = {.count = 1, .reusable = true}}, SHIFT(71),
  [83] = {.entry = {.count = 1, .reusable = false}}, SHIFT(70),
  [85] = {.entry = {.count = 1, .reusable = true}}, SHIFT(70),
  [87] = {.entry = {.count = 1, .reusable = true}}, SHIFT(80),
  [89] = {.entry = {.count = 1, .reusable = true}}, SHIFT(60),
  [91] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_room_block_repeat1, 2),
  [93] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_room_block_repeat1, 2), SHIFT_REPEAT(71),
  [96] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_room_block_repeat1, 2), SHIFT_REPEAT(70),
  [99] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_room_block_repeat1, 2), SHIFT_REPEAT(70),
  [102] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_room_block_repeat1, 2), SHIFT_REPEAT(80),
  [105] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_room_block_repeat1, 2), SHIFT_REPEAT(60),
  [108] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [110] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_desc, 2),
  [112] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_cond, 4),
  [114] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_group, 2),
  [116] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_stmt, 1),
  [118] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_done, 3),
  [120] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_goal_cond, 3),
  [122] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_npc_block_repeat1, 2),
  [124] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_npc_block_repeat1, 2), SHIFT_REPEAT(78),
  [127] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_npc_block_repeat1, 2), SHIFT_REPEAT(77),
  [130] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_npc_block_repeat1, 2), SHIFT_REPEAT(77),
  [133] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_npc_block_repeat1, 2), SHIFT_REPEAT(89),
  [136] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [138] = {.entry = {.count = 1, .reusable = true}}, SHIFT(78),
  [140] = {.entry = {.count = 1, .reusable = false}}, SHIFT(77),
  [142] = {.entry = {.count = 1, .reusable = true}}, SHIFT(77),
  [144] = {.entry = {.count = 1, .reusable = true}}, SHIFT(89),
  [146] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_item_block_repeat1, 2),
  [148] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_item_block_repeat1, 2), SHIFT_REPEAT(63),
  [151] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_item_block_repeat1, 2), SHIFT_REPEAT(68),
  [154] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_item_block_repeat1, 2), SHIFT_REPEAT(68),
  [157] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_item_block_repeat1, 2), SHIFT_REPEAT(85),
  [160] = {.entry = {.count = 1, .reusable = true}}, SHIFT(35),
  [162] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [164] = {.entry = {.count = 1, .reusable = true}}, SHIFT(63),
  [166] = {.entry = {.count = 1, .reusable = false}}, SHIFT(68),
  [168] = {.entry = {.count = 1, .reusable = true}}, SHIFT(68),
  [170] = {.entry = {.count = 1, .reusable = true}}, SHIFT(85),
  [172] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [174] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_def, 3),
  [176] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_room_block, 2),
  [178] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_list, 4),
  [180] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_npc_def, 3),
  [182] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_room_block, 3),
  [184] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_spinner_block, 2),
  [186] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_room_def, 3),
  [188] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_npc_block, 2),
  [190] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_list, 3),
  [192] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_trigger, 4),
  [194] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_npc_block, 3),
  [196] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_spinner_def, 3),
  [198] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_block, 2),
  [200] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_trigger, 5),
  [202] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_set_decl, 5),
  [204] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_block, 3),
  [206] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_spinner_block, 3),
  [208] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_boolean, 1),
  [210] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_boolean, 1),
  [212] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_room_stmt, 1),
  [214] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_room_stmt, 1),
  [216] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_room_name, 2),
  [218] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_room_name, 2),
  [220] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_room_desc, 2),
  [222] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_room_desc, 2),
  [224] = {.entry = {.count = 1, .reusable = true}}, SHIFT(103),
  [226] = {.entry = {.count = 1, .reusable = true}}, SHIFT(100),
  [228] = {.entry = {.count = 1, .reusable = true}}, SHIFT(88),
  [230] = {.entry = {.count = 1, .reusable = true}}, SHIFT(104),
  [232] = {.entry = {.count = 1, .reusable = true}}, SHIFT(105),
  [234] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_room_visited, 2),
  [236] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_room_visited, 2),
  [238] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_exit_stmt, 4),
  [240] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_exit_stmt, 4),
  [242] = {.entry = {.count = 1, .reusable = false}}, SHIFT_EXTRA(),
  [244] = {.entry = {.count = 1, .reusable = false}}, SHIFT(4),
  [246] = {.entry = {.count = 1, .reusable = false}}, SHIFT(59),
  [248] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_npc_name, 2),
  [250] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_npc_name, 2),
  [252] = {.entry = {.count = 1, .reusable = false}}, SHIFT(93),
  [254] = {.entry = {.count = 1, .reusable = false}}, SHIFT(62),
  [256] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_stmt, 1),
  [258] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_item_stmt, 1),
  [260] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_portable, 2),
  [262] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_item_portable, 2),
  [264] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_desc, 2),
  [266] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_item_desc, 2),
  [268] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_item_name, 2),
  [270] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_item_name, 2),
  [272] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_npc_desc, 2),
  [274] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_npc_desc, 2),
  [276] = {.entry = {.count = 1, .reusable = false}}, SHIFT(95),
  [278] = {.entry = {.count = 1, .reusable = false}}, SHIFT(51),
  [280] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_npc_stmt, 1),
  [282] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_npc_stmt, 1),
  [284] = {.entry = {.count = 1, .reusable = false}}, SHIFT(2),
  [286] = {.entry = {.count = 1, .reusable = false}}, SHIFT(123),
  [288] = {.entry = {.count = 1, .reusable = false}}, SHIFT(3),
  [290] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [292] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [294] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_npc_location, 3),
  [296] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_npc_location, 3),
  [298] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_string_repeat1, 2),
  [300] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_string_repeat1, 2), SHIFT_REPEAT(62),
  [303] = {.entry = {.count = 1, .reusable = false}}, SHIFT(73),
  [305] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
  [307] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_trigger_repeat1, 2), SHIFT_REPEAT(73),
  [310] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_trigger_repeat1, 2),
  [312] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [314] = {.entry = {.count = 1, .reusable = false}}, SHIFT(96),
  [316] = {.entry = {.count = 1, .reusable = true}}, SHIFT(96),
  [318] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [320] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_spinner_block_repeat1, 2),
  [322] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_spinner_block_repeat1, 2), SHIFT_REPEAT(66),
  [325] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
  [327] = {.entry = {.count = 1, .reusable = true}}, SHIFT(66),
  [329] = {.entry = {.count = 1, .reusable = true}}, SHIFT(41),
  [331] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [333] = {.entry = {.count = 1, .reusable = true}}, SHIFT(107),
  [335] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [337] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [339] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_set_list_repeat1, 2), SHIFT_REPEAT(107),
  [342] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_set_list_repeat1, 2),
  [344] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [346] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_wedge_stmt, 2),
  [348] = {.entry = {.count = 1, .reusable = true}}, SHIFT(111),
  [350] = {.entry = {.count = 1, .reusable = true}}, SHIFT(102),
  [352] = {.entry = {.count = 1, .reusable = true}}, SHIFT(113),
  [354] = {.entry = {.count = 1, .reusable = true}}, SHIFT(109),
  [356] = {.entry = {.count = 1, .reusable = true}}, SHIFT(64),
  [358] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [360] = {.entry = {.count = 1, .reusable = true}}, SHIFT(76),
  [362] = {.entry = {.count = 1, .reusable = true}}, SHIFT(24),
  [364] = {.entry = {.count = 1, .reusable = true}}, SHIFT(10),
  [366] = {.entry = {.count = 1, .reusable = true}}, SHIFT(114),
  [368] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_wedge_stmt, 4),
  [370] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__trigger_stmt, 2),
  [372] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__trigger_stmt, 2),
  [374] = {.entry = {.count = 1, .reusable = false}}, SHIFT(74),
  [376] = {.entry = {.count = 1, .reusable = false}}, SHIFT(18),
  [378] = {.entry = {.count = 1, .reusable = true}}, SHIFT(97),
  [380] = {.entry = {.count = 1, .reusable = false}}, SHIFT(87),
  [382] = {.entry = {.count = 1, .reusable = false}}, SHIFT(90),
  [384] = {.entry = {.count = 1, .reusable = false}}, SHIFT(61),
  [386] = {.entry = {.count = 1, .reusable = false}}, SHIFT(91),
  [388] = {.entry = {.count = 1, .reusable = true}}, SHIFT(98),
  [390] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [392] = {.entry = {.count = 1, .reusable = true}}, SHIFT(118),
  [394] = {.entry = {.count = 1, .reusable = false}}, SHIFT(81),
  [396] = {.entry = {.count = 1, .reusable = false}}, SHIFT(92),
  [398] = {.entry = {.count = 1, .reusable = false}}, SHIFT(48),
  [400] = {.entry = {.count = 1, .reusable = false}}, SHIFT(94),
  [402] = {.entry = {.count = 1, .reusable = false}}, SHIFT(14),
  [404] = {.entry = {.count = 1, .reusable = false}}, SHIFT(122),
  [406] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [408] = {.entry = {.count = 1, .reusable = false}}, SHIFT(106),
  [410] = {.entry = {.count = 1, .reusable = true}}, SHIFT(65),
  [412] = {.entry = {.count = 1, .reusable = true}}, SHIFT(116),
  [414] = {.entry = {.count = 1, .reusable = true}}, SHIFT(121),
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
