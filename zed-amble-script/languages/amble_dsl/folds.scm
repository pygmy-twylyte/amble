; Fold blocks enclosed in braces
(room_block "{" @fold.start "}" @fold.end)
(item_block "{" @fold.start "}" @fold.end)
(npc_block "{" @fold.start "}" @fold.end)
(spinner_block "{" @fold.start "}" @fold.end)

; Test: Add highlighting queries to the working folds file
; Comments
(comment) @comment

; Strings
(string) @string

; Numbers
(number) @number

; Keywords
"room" @keyword
"item" @keyword
"npc" @keyword
"goal" @keyword
"name" @keyword
"desc" @keyword

; Identifiers
(identifier) @variable
