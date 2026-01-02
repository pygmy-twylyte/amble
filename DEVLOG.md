# Amble Informal Development Log

I've been looking for a way to keep informal notes on development of Amble for a couple of reasons:

- so I have a place to organize thoughts on works in progress
- so it's easy for others to see what's being done

For the *few* of you out there lurking and watching the my pet project so far, it may sometimes seem like nothing is changing, though I work on Amble just about every day in some aspect or another. Many times, these are small changes buried in the DSL, the demo game content, or updates to the companion Zed extension and language server. 

So, I intend to keep an informal log here, going forward. With the exception of this intro, newest entries will be at the top.

---
1-1-2026

Feeling uninspired today. Looked at code to refactor -- meh. Thought about starting a dialogue feature upgrade. Meh again. I even went and looked at a couple of my other repos -- meh.

So, I went through some previous notes I made about content adjustments for the demo game and implemented them. I guess that's it for today.

---
12-31-2025

I've done some recent reading about refactoring and code readability. When I go back to look at some of the code both the engine and the script compiler, there are abundant "code smells". I spent some time doing some of the simpler types of refactors today -- renaming variables, extracting complicted logic into functions, using guard clauses to reduce indentation hell. I'm really surprised and pleased at how much more readable and maintainable these simple chnages can make the code. 

Didn't get anything done on updating content.

Also sad to say I had to lean on Codex to get some semblance of a refactor going for amble_script. It was a giant mess that had come to the point where I had no idea how it worked in many areas. I'm hoping the simple changes Codex made (mostly separating things into modules and extracting functions) will make it easier for me to get back on top of that part of Amble. If I'm ever to tackle macros (even compiler-defined ones), I'll need a much more solid understanding before I can insert that logic.

---
12-30-2025

**Amble's Birthday!**
I got curious and looked back to figure out when exactly I started working on Amble. The conversation with ChatGPT about "type driven development" that led to me starting work on the engine was on **July 25, 2025**.  

**Amble work today**
Today... after doing a *tiny* bit of work on another project (medicalc) I came back to Amble and refactored the View module heavily, adding the ability to use markup in the triggered `do show` messages now. Already merged.

**Looking Forward a Bit**
After asking GPT when we had that conversation, "we" chatted a bit more about where Amble is and next steps. I think the next Big Thing™ is going to be an overhaul / recreation of NPC dialogue, so that actual (scripted) conversations are possible. 

The other couple of ideas I had (macros / meta-programming for the DSL, and DSL-definable item abilities and interactions) are a bust, I think. The first would be high effort with low impact. The second would have really high impact, but is nearly impossible to implement with a parser engine. The DSL would have to be able to teach new verbs to the parser, how to translate that to a Command variant, and then the DSL would have to have some way to tell the engine how to process and display results from it... and that point, the DSL would be getting complicated enough that they might as well just learn Rust and add it to the engine!

---
12-29-2025

I merged the content update branch (that also had a few minor engine tweaks and refactors and docstring updates). There's still about half the demo game for me to play through including a bunch of puzzle content that will likely need tuning up, but I wanted to get this `DEVLOG` into `main`, so I went ahead with a merge.

###### Thinking 0.65.0?

Probably soon. With the markup module, entity search and some other refactoring we can already call this a minor version -- but I'd like to refresh the rest of the demo content before a new release. 

---
12-28-2025

- started this DEVLOG
- caught up on content tweaks "todo" notes I made with the new :note system. Nothing fancy, mostly fixing some inconsistencies with descriptions after state changes in the world. 
- Now at least the poor Gonk Droid can get his family photo back!
- this is all in the demo-game-content-updates branch -- nothing merged to main yet
- considered possible ways of making new item abilities and interactions possible to create from within the DSL / at runtime. Gave me a friggin headache. The biggest problem here is that this is a parser game engine, and the parser has to translate near-natural English to Command:: variants that the engine can understand. The parser can't be taught new vocabulary at runtime. A custom command variant would be easy enough to create, but it would have to be tied to some type of a command handler -- that also couldn't be defined with runtime data. It may just not be possible with this design?
