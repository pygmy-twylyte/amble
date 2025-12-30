# Amble Informal Development Log

I've been looking for a way to keep informal notes on development of Amble for a couple of reasons:

- so I have a place to organize thoughts on works in progress
- so it's easy for others to see what's being done

For the *few* of you out there lurking and watching the my pet project so far, it may sometimes seem like nothing is changing, though I work on Amble just about every day in some aspect or another. Many times, these are small changes buried in the DSL, the demo game content, or updates to the companion Zed extension and language server. 

So, I intend to keep an informal log here, going forward. With the exception of this intro, newest entries will be at the top.

---
12-28-2022

- started this DEVLOG
- caught up on content tweaks "todo" notes I made with the new :note system. Nothing fancy, mostly fixing some inconsistencies with descriptions after state changes in the world. 
- Now at least the poor Gonk Droid can get his family photo back!
- this is all in the demo-game-content-updates branch -- nothing merged to main yet
- considered possible ways of making new item abilities and interactions possible to create from within the DSL / at runtime. Gave me a friggin headache. The biggest problem here is that this is a parser game engine, and the parser has to translate near-natural English to Command:: variants that the engine can understand. The parser can't be taught new vocabulary at runtime. A custom command variant would be easy enough to create, but it would have to be tied to some type of a command handler -- that also couldn't be defined with runtime data. It may just not be possible with this design?
