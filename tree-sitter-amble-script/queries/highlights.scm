; Amble DSL syntax highlighting for tree-sitter

; Comments
(comment) @comment

; Keywords - Definition types
"room" @keyword
"item" @keyword
"npc" @keyword
"goal" @keyword
"trigger" @keyword
"spinner" @keyword

; Keywords - Declaration
"let" @keyword
"set" @keyword

; Keywords - Properties
"name" @keyword
"desc" @keyword
"description" @keyword
"portable" @keyword
"visited" @keyword
"location" @keyword
"text" @keyword
"overlay" @keyword
"dialogue" @keyword
"movement" @keyword
"exit" @keyword
"wedge" @keyword
"width" @keyword
"group" @keyword
"done" @keyword

; Operators
"->" @operator
"=" @operator

; Contextual keywords — goal statements and conditions
(goal_done "done" @keyword)
(goal_done "when" @keyword)
(goal_start "start" @keyword)
(goal_start "when" @keyword)
(goal_group "group" @keyword)
(goal_group "required" @keyword)
(goal_group "optional" @keyword)
(goal_group "status-effect" @keyword)
(goal_cond "has" @keyword)
(goal_cond "missing" @keyword)
(goal_cond "reached" @keyword)
(goal_cond "complete" @keyword)
(goal_cond "in" @keyword)
(goal_cond "progress" @keyword)
(goal_cond "flag" @keyword)
(goal_cond "item" @keyword)
(goal_cond "goal" @keyword)
(goal_cond "room" @keyword)

; Keywords - Goal groups (removed global; handled above)

; Booleans
(boolean) @boolean

; Strings
(string) @string

; Numbers
(number) @number

; Punctuation - Brackets
"{" @punctuation.bracket
"}" @punctuation.bracket
"(" @punctuation.bracket
")" @punctuation.bracket

; Punctuation - Delimiters
"," @punctuation.delimiter

; Identifiers - context-specific highlighting
(room_def (identifier) @type)
(item_def (identifier) @type)
(npc_def (identifier) @type)
(goal_def (identifier) @type)
(trigger (identifier) @type)
(spinner_def (identifier) @type)
(set_decl (identifier) @type)

; Exit destinations
(exit_stmt . (identifier) @variable . "->" . (identifier) @type)
(exit_stmt . (string) @string . "->" . (identifier) @type)

; Exit attribute keywords
(exit_required_flags "required_flags" @keyword)
(exit_required_items "required_items" @keyword)
(exit_barred "barred" @keyword)

; Overlay entry keywords
(overlay_entry ("set" "unset" "text" "normal" "happy" "bored") @keyword)

; Triggers: keywords and action names
(trigger_mod "when" @keyword)
(trigger_mod "only" @keyword)
(trigger_mod "once" @keyword)
(if_block "if" @keyword)
(do_stmt "do" @keyword)
(do_stmt (identifier) @function
  (#match? @function "^(spawn|despawn|add|award|schedule|show|give|set|npc)$"))

; Trigger conditions (selected words by text)
(cond_line (identifier) @keyword
  (#match? @keyword "^(chance|has|missing|flag|act|on|item|npc|enter|look|give|to|player|room|in|present|absent)$"))

; Flag/Item/Room references in conditions
(goal_cond "flag" (identifier) @variable)
(goal_cond "item" (identifier) @variable)
(goal_cond "room" (identifier) @variable)
(goal_cond "goal" (identifier) @variable)

; Location references (items and NPCs)
(location "room" (identifier) @type)
(location "npc" (identifier) @type)
(location "chest" (identifier) @type)
(location "inventory" "player")
(location "nowhere" (string) @string)

; Item-specific keywords (contextual)
(item_text "text" @keyword)
(item_ability "ability" @keyword)
(item_container_state "container" @keyword)
(item_container_state "state" @keyword)
(item_container_state "open" @keyword)
(item_container_state "closed" @keyword)
(item_restricted "restricted" @keyword)

; Generic identifiers
(identifier) @variable
