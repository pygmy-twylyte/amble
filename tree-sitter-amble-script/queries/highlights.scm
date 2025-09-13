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
"exit" @keyword
"wedge" @keyword
"width" @keyword
"group" @keyword
"done" @keyword

; Keywords - Conditionals and operators
"when" @keyword
"has" @keyword
"missing" @keyword
"reached" @keyword
"complete" @keyword
"in" @keyword
"progress" @keyword

; Keywords - Locations and states
"room" @keyword
"nowhere" @keyword
"flag" @keyword

; Keywords - Goal groups
"required" @keyword
"optional" @keyword
"status-effect" @keyword

; Booleans
(boolean) @boolean

; Strings
(string) @string

; Numbers
(number) @number

; Operators
"->" @operator
"=" @operator

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

; Flag references in conditions
(goal_cond "flag" (identifier) @variable)
(goal_cond "item" (identifier) @variable)
(goal_cond "room" @variable)
(goal_cond "goal" (identifier) @variable)

; NPC location room references
(npc_location "room" (identifier) @type)

; Generic identifiers
(identifier) @variable
