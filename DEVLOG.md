# Amble Informal Development Log

I've been looking for a way to keep informal notes on development of Amble for a couple of reasons:

- so I have a place to organize thoughts on works in progress
- so it's easy for others to see what's being done

For the *few* of you out there lurking and watching the my pet project so far, it may sometimes seem like nothing is changing, though I work on Amble just about every day in some aspect or another. Many times, these are small changes buried in the DSL, the demo game content, or updates to the companion Zed extension and language server. 

So, I intend to keep an informal log here, going forward. With the exception of this intro, newest entries will be at the top.
---
1-17-2026

I'm surprised it's been 10 days since I updated this. I've been working on the multi-game system (game-chooser branch) and it's about good to go… so I've been working on a second small mini-game-demo to distribute alongside the main amble demo, so that people can see how the game chooser works right away.

The new demo will have a more serious tone. A "wake up attached to a strange IV in an abandoned hospital" situation. 

Also been working through exercism and lessons learned there have led to use of a macro / little refactoring in the code -- looking for other opportunities to leverage that and the .fold() method. 

---
1-7-2026

Tried using Claude to improve diagnostics in the language server. Unmitigated disaster. Reverted all changes. I'm surprised, after having good luck with Claude on some other stuff some months ago. Oh well. That's what git is for.

User comment on Reddit inspired me to finally add abbreviations and an easier way to navigate. I looked at a few ways to do it... wound up finding the easiest way was to catch and _massage_ user input before passing it on to the parser. Just tested, committed, and pushed it.

---
1-5-2026

It's been nice to see a *little* reaction from people on this. Taking a little bit of a breather to let the new 0.65.0 release settle. Put in a little work on the language server and some calculations in medicalc. I'll probably toy with / polish the game demo before I do anything else.

Had a comment about adding shorthand commands (l for look, cardinal directions etc) -- creating an issue for it -- should be easy to implement. Just need to decide what the shorthand will be. On a related note, maybe add room exit names to autocomplete, too. 

---
1-3-2026

Inspiration wound up striking pretty hard after my 1-1 post and it led to a whirlwind of changes to the engine's data pipeline. The TOML intermediary files are all gone. We define everything in .amble files that isn't a static part of the engine. That improved and simplified the loader code in many ways. 

That BIG breaking change along with the bevy of other changes (markup, :note command) led me to go forward with the 0.65.0 release today.

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
