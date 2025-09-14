; Simplified Amble DSL syntax highlighting
; Uses only patterns confirmed to work

; Comments
(comment) @comment

; Strings
(string) @string

; Numbers
(number) @number

; Booleans
(boolean) @constant

; Keywords as string literals
"room" @keyword
"item" @keyword
"npc" @keyword
"goal" @keyword
"trigger" @keyword
"spinner" @keyword
"let" @keyword
"done" @keyword
"set" @keyword
"unset" @keyword
"name" @keyword
"desc" @keyword
"description" @keyword
"portable" @keyword
"visited" @keyword
"location" @keyword
"exit" @keyword
"wedge" @keyword
"width" @keyword
"text" @keyword
"overlay" @keyword
"dialogue" @keyword
"movement" @keyword
"required_flags" @keyword
"required_items" @keyword
"ability" @keyword
"container" @keyword
"timing" @keyword
"state" @keyword
"restricted" @keyword
"requires" @keyword
"to" @keyword
"open" @keyword
"closed" @keyword
; Trigger keywords (structural to avoid substring matches inside identifiers)
(do_stmt "do" @keyword)
(if_block "if" @keyword)
(trigger_mod "when" @keyword)
(trigger_mod "only" @keyword)
(trigger_mod "once" @keyword)

; (intentionally no additional trigger action/condition highlighting)
; Contextual goal keywords
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
; (contextual only; no global fallbacks to avoid partial matches)
"true" @constant
"false" @constant

; Operators
"->" @operator
"=" @operator

; Punctuation
"{" @punctuation
"}" @punctuation
"(" @punctuation
")" @punctuation
"," @punctuation

; Basic identifiers
(identifier) @variable

; Item requires: requires <ability> to <interaction>
(item_requires
  "requires" @keyword
  (identifier) @function
  "to" @keyword
  (identifier) @method)

; Contextual: highlight identifier tokens that read 'requires' and 'to'
; This supports lines like: "requires <ability> to <interaction>"
((identifier) @keyword (#match? @keyword "^requires$"))
((identifier) @keyword (#match? @keyword "^to$"))
