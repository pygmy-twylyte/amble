# Brainstorming Ideas for Amble

## CODING TODOs
- add ambient triggers checked each repl look in a check_ambient_triggers() function
  - TriggerCondition::Ambient{ Option<room_uuid>, spinner_type }
  - function runs at end of repl, finds and fires those Ambients related actions if the location matches (if specified)
- need a call of some kind to check_triggers with any recognized command so state triggers can fire too?
- dev commands to teleport (DONE), get items / achievs
- add "clean" / "wipe" item interaction
- spinner for when no destination matches on "go" command
- more go synonyms that work with things like "ladder" or "trapdoor" (climb, go through, enter, etc.)
- giveitemtoNPC should block (or reverse) transfer if there is no trigger for NPC response. "NPC doesn't want that." spinner.
- trigger action to change the .read field on an item to update displays etc

## FIXITs from runthroughs / CONTENT additions
- remove key access to security suppy crate to avoid inventory problems if crate is destroyed after contents removed
- make Vogon Poetry readable
- make reservation book readable
- rename abandoned game from Brains Anonymous: Pong Construction Set '84
- A. Jackson needs a mailbox and something funny / lore-building inside it (jackson's landing)
- add trigger so something happens if you sharpen the axe with the stone
- maybe add river bank location south of inca-road (already mentioned in description) and put whetstone there
- turn west of building into a snowfield?
- charged battery should be restricted once placed in portal gun. Might require modification of take_from handler.
- add warning / instruction label to read on poetry performer, and make it Initech


## 🧠 World Concept: The AA-3B Breach & Amble's Origin

Amble Adventures is not the cause of the anomaly — it’s the **opportunistic theme park** built on top of it.

### 📍 The Real Origin
Deep beneath the facility lies **Room AA-3B**, once part of a high-energy experimental complex run by an unknown actor (e.g., Brains Anonymous R&D, or a defunct predecessor). Something went catastrophically wrong there — a breach event that punctured the boundary between **reality and fiction**.

This unleashed a **fictional leakage effect**: characters, logic systems, genres, and metaphors began seeping into the world in **unpredictable ways**.

### 🎭 Amble Adventures: Built on Top
Seeing a commercial opportunity in the weirdness, Amble Corp acquired the site and built a guided adventure experience around it, presenting the surreal anomalies as curated content. They *don’t* understand the full scope — they just try to monetize it.

---

## 🧃 Writing Guide for Content Creators

When designing **rooms, items, NPCs, or text**, consider:

### ✳️ Tone
- Blend mundane corporate enthusiasm with reality-bending consequences
- Channel Douglas Adams, Black Mesa, Zork, and Portal
- Humor should coexist with genuine eeriness or mystery

### 🌐 Setting Consistency
- Most of the world behaves *normally*, but fiction has **bled through**
- Strange objects or logic may be rationalized in-universe (“That’s just the Vogon exhibit.”) — but players may suspect deeper origin
- AA-3B references should be **indirect but recurring**

### 🧩 Useful Themes & Tropes
- Bureaucratic paperwork describing narrative paradoxes
- Office memos trying to define “meta-containment protocols”
- Vacation photos taken in places that don’t exist anymore
- NPCs unsure if they're *roles* or *people*
- Redacted elevator buttons, terminals that glitch between genres

### 📍 Existing Hooks
- `guard-post` already references **anachronism**
- `room-aa-3b` exists but is unreachable (for now)
- Gonk’s family photo includes a caption referencing **AA-3B staff & families**
- `visitor_pass` and `elevator_keycard` open the way deeper into the structure

---

## 🚪 Thematic Endgame Possibilities (for later)
- Discover Room AA-3B (or what's left of it)
- Patch or widen the breach
- Meet a version of yourself from a different fictional structure
- Choose whether to contain or embrace the seepage




(VG = video game, FZ = Zappa, MP = Monty Python, DW = DiscWorld, SF = General SciFi, HH = Hitchhiker's Guide)
## Possible Themed Locations?
- (VG: Portal) Cake room (with No Cake)
- (FZ) Parish of St. Alfonzo
x (FZ) Inca Road(s)
- (VG: DS3) Firelink Shrine
x (VG: Portal2) Aperture Science Lab
- (VG: Zelda) ??? Somewhere in Hyrule?
- (MP) Cave of Caerbannog, Swamp Castle, Castle Anthrax, Enchanted Forest, Cheese Shop, Square where "Romans Go Home" is painted
- (MP) Two Sheds
- (DW) *SOMETHING* created by Bloody Stupid Johnson!
- (SF) TARDIS Control
x (HH) Restaurant at the End of the Universe

## Possible Items
- (VG) No Cake
- (FZ) Zircon-encrusted Tweezers, Dental Floss Bush (= (MP) a shrubbery?)
- (FZ) St. Alfonzo's Margarine, Father O'Blivion's Smock, Pancakes
- (FZ) Photo of Punky Meadows (Oh Punky!)
- (FZ) Python Boots (stinkfoot)
- (VG) Golden Spork (from GlaDOS)
- (MP) Holy Hand Grenade, Brian's Gourd/Shoe, Salmon Mousse, Wafer Thin Mint, No Cheese!, Dead Parrot
- (DW) Vimes' Boots, CMOT's Sausage inna Bun, Death's Scythe, Rincewind's "Wizzard" hat.
- (HH) Vogon Poetry Book, Peril-Sensitive Sunglasses, Babel Fish, Towel
- (SF) Monolith (2001), Tricorder


## Possible Characters (NPCs)
- (FZ) Father O'Blivion, Thing-Fish
- (VG) GlaDOS (in potato)
- (MP) Brave Sir Robin, Mr. Creosote, Knights of Ni!, Tim the Enchanter, Hermit (from Life of Brian), Ex-Leper
- (DW) DEATH, Sam Vimes, Corporal Carrot, Granny Weatherwax, Rincewind, The Luggage, Detritus, CMOT Dibbler
- (SF) Dalek, K-9, The Doctor, HAL-9000, 7 of 9
- (HH) Marvin the Paranoid Android
