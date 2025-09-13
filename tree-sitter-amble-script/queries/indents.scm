; Simple indentation rules: increase after block openings

; room/item/npc/spinner blocks
(room_block "{" @indent)
(room_block "}" @outdent)

(item_block "{" @indent)
(item_block "}" @outdent)

(npc_block "{" @indent)
(npc_block "}" @outdent)

(spinner_block "{" @indent)
(spinner_block "}" @outdent)

