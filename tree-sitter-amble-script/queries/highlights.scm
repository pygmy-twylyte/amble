((comment) @comment)
((string) @string)
((number) @constant.numeric)
((boolean) @constant.builtin)
((identifier) @variable)

"room" @keyword
"item" @keyword
"npc" @keyword
"goal" @keyword
"trigger" @keyword
"spinner" @keyword
"exit" @keyword

; Diagnostic: color everything if highlights are loading
; Remove after verification
(program) @keyword
