; Comprehensive indentation rules for Amble DSL
; Handles all block structures and multi-line constructs

; Block definitions - main structures
(room_def "{" @indent)
(room_def "}" @outdent)

(item_def "{" @indent)
(item_def "}" @outdent)

(npc_def "{" @indent)
(npc_def "}" @outdent)

(trigger "{" @indent)
(trigger "}" @outdent)

(spinner_def "{" @indent)
(spinner_def "}" @outdent)

; Block contents - individual block types
(room_block "{" @indent)
(room_block "}" @outdent)

(item_block "{" @indent)
(item_block "}" @outdent)

(npc_block "{" @indent)
(npc_block "}" @outdent)

(spinner_block "{" @indent)
(spinner_block "}" @outdent)

; Set declarations with parentheses
(set_decl "(" @indent)
(set_decl ")" @outdent)

(set_list "(" @indent)
(set_list ")" @outdent)

; Multi-line string handling
; Triple-quoted strings should maintain internal formatting
(string "\"\"\"" @indent.begin)
(string "\"\"\"" @indent.end)

; Goal definitions - indent continuation lines
(goal_def @indent.begin)
(goal_stmt @indent)

; Trigger statements - indent content
(_trigger_stmt @indent)

; Wedge statements in spinners
(wedge_stmt @indent)

; General fallback for any block structure
("{" @indent)
("}" @outdent)
("(" @indent)
(")" @outdent)
