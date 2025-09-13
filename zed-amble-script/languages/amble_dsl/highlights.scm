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
"set" @keyword
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
"when" @keyword
"has" @keyword
"missing" @keyword
"reached" @keyword
"complete" @keyword
"in" @keyword
"progress" @keyword
"nowhere" @keyword
"flag" @keyword
"required" @keyword
"optional" @keyword
"status-effect" @keyword
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
