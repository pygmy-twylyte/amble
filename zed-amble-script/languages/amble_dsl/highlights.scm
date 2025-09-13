; Comprehensive Amble DSL syntax highlighting for Zed
; Uses standard TextMate scopes for maximum theme compatibility

; Comments
(comment) @comment.line

; Strings (all types including triple-quoted)
(string) @string.quoted

; Numbers
(number) @constant.numeric

; Booleans
(boolean) @constant.language.boolean

; Top-level definition keywords
"room" @keyword.control
"item" @keyword.control
"npc" @keyword.control
"goal" @keyword.control
"trigger" @keyword.control
"spinner" @keyword.control

; Declaration keywords
"let" @keyword.control
"set" @keyword.control

; Property keywords
"name" @keyword.other
"desc" @keyword.other
"description" @keyword.other
"portable" @keyword.other
"visited" @keyword.other
"location" @keyword.other
"exit" @keyword.other
"wedge" @keyword.other
"width" @keyword.other
"group" @keyword.other
"done" @keyword.other

; Conditional and state keywords
"when" @keyword.control
"has" @keyword.control
"missing" @keyword.control
"reached" @keyword.control
"complete" @keyword.control
"in" @keyword.control
"progress" @keyword.control

; Special location and state keywords
"nowhere" @keyword.other
"flag" @keyword.other
"room" @keyword.other

; Goal group types
"required" @constant.language
"optional" @constant.language
"status-effect" @constant.language

; Boolean literals (explicit)
"true" @constant.language.boolean
"false" @constant.language.boolean

; Operators
"->" @keyword.operator
"=" @keyword.operator

; Punctuation
"{" @punctuation.definition.block.begin
"}" @punctuation.definition.block.end
"(" @punctuation.definition.group.begin
")" @punctuation.definition.group.end
"," @punctuation.separator.comma

; Definition identifiers (entity names)
(room_def (identifier) @entity.name.type.room)
(item_def (identifier) @entity.name.type.item)
(npc_def (identifier) @entity.name.type.npc)
(goal_def (identifier) @entity.name.type.goal)
(trigger (identifier) @entity.name.type.trigger)
(spinner_def (identifier) @entity.name.type.spinner)
(set_decl (identifier) @entity.name.type.set)

; Property value identifiers in definitions
(room_name (string) @string.quoted.room-name)
(room_desc (string) @string.quoted.description)
(item_name (string) @string.quoted.item-name)
(item_desc (string) @string.quoted.description)
(npc_name (string) @string.quoted.npc-name)
(npc_desc (string) @string.quoted.description)

; Exit statements - direction and destination
(exit_stmt . (identifier) @variable.other.direction . "->" . (identifier) @entity.name.type.room)

; Goal conditions - highlight different reference types
(goal_cond "flag" (identifier) @variable.other.flag)
(goal_cond "item" (identifier) @variable.other.item)
(goal_cond "room" (identifier) @variable.other.room)
(goal_cond "goal" (identifier) @variable.other.goal)

; NPC location room references
(npc_location "room" (identifier) @entity.name.type.room)

; Set list identifiers
(set_list (identifier) @variable.other.set-member)

; Generic identifiers (fallback)
(identifier) @variable.other
