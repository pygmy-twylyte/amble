; Simple indentation rules for Amble DSL
; Uses the same block patterns as folds.scm

; Block structures that should indent content
(room_block "{" @indent)
(room_block "}" @outdent)

(item_block "{" @indent)
(item_block "}" @outdent)

(npc_block "{" @indent)
(npc_block "}" @outdent)

(spinner_block "{" @indent)
(spinner_block "}" @outdent)

; Set declarations with parentheses
(set_list "(" @indent)
(set_list ")" @outdent)

; General bracket indentation fallback
("{" @indent)
("}" @outdent)
("(" @indent)
(")" @outdent)
