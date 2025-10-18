# Amble Playtest Notes 10/2/2025

## General
- [ ] now that exit directions can be strings with whitespace, spice up all of the bare N-E-S-W directions to be more descriptive
- [x] st-alfonzo-parish should be removed from ambientWoodland events, which are largely outdoorsy
- [ ] inspect/read item text style is a bit dim-looking in both default and seaside themes... brighter green/blue
- [ ] globally, when you talk to an NPC their movement activity should be frozen for a few turns -- so they don't walk away in the middle of a conversation
- [x] add status:bad-breath
- [ ] more delayed callbacks to player choices in ambients


## Specific Locations

### intro.txt
- [x] still describes starting at inca-road location; player now starts at high-ridge

### high-ridge
- [x] convert cupcake to cake that vanishes upon attempt to eat, replaced by no_cake in inventory
- [x] weathered_plaque desc "... suggests >the< utmost ..."

### parish-landing
- no problems, but nothing happens here, unless a scheduled event happens to fire
- [x] consider moving plaque 1 here

### st-alfonzo-parish
- [x] messiah kit description - the <<...>> placement is wrong-looking
- shoe and gourd are fun for those in the know, but otherwise appear random / meaningless (good!)
- margarine is the same, alludes to yellow-snow cure... maybe get yellow-snow blindness in snowfield area and cure with it?

### two-sheds-landing
- no issues
- fallen tree puzzle functional (both solutions)

### two-sheds
- no issues
- good overlay changes as the tools are taken/dropped

### guard-post
- [x] nothing happens here except the towel realization trigger

### inca-road
- no issues

### loading-dock
- [x] supply locker -- add "examine" text regarding the highly flammable special wood
- [x] strange liquid should be drinkable for some fun bit or hint, but can't be critical (doesn't spawn if locker broken)
- break and burn supply locker work with correct damage to internal items

### snowfield
- need the husky here as overlay or as NPC - not mentioned
- [x] trigger the husky after the player visits the ice pit for the first time

### ice-pit
- [x] need plaque 2 here
- [x] have firewood stacked next to it already or need to bring from snow-camp
- [x] light fire --> scheduled changes in description of plaque until done / legible
- [x] description -> "window" -> "opening".

### snow-camp
- [x] flamethrower desc formatting "Weyland-Yutani" indenting looks off

### east-of-building
- [x] "pride of montana" funny botanical sign next to the bush
- [ ] room overlays should cycle, intermittently allowing player to see through the wall into the restaurant

--------

(saved here as playtest-building-main)

### front-entrance
- [x] description, change security eye-bot to Initech Eye-Bot[tm]

### main-lobby
- [x] HAL module #2 shouldn't be visible in the vending machine until starts HAL sequence
- [ ] add ability to break open and get message from fortune cookie
- loitering trigger/penalty works nicely

### b-a-office
- [ ] typo: extra R_PARENS at end of "exasperated sigh" dialogue line
- [ ] rewording of poetry-performer description: "whichever" voice, "it" happens
- [ ] "robotic receptionist initially GLANCED at you" would read better  - past tense the whole passage because the dialogue shows her already awarding the pass just above


### restaurant
- [ ] desc: not jazz. Maybe Artcuran pop.
- [ ] desc: mention reservation book is affixed to the podium (not portable)


### patio
- denies entry w/o sunglasses
- [ ] Third plaque here
- [ ] NPC here
- [ ] make negative cocktail drinkable

### lift-bank-main
- prox card overlay working when no card
- [ ] start with two elevators on floor "?"
- [ ] add a button to call the lift to the main floor, only works when have keycard

### lift-main
- correctly locks down with aa-3 button dark in overlay
- [ ] change to push buttons for floors (use push player to)

### lounge
- [ ] add a brochure or magazine on the coffee table
- [ ] hot_coffee must be drinkable
- [ ] replace hot with cold after a few turns

### portal-room
- [x] **HAS NO EXITS!** Need exit back up ladder to lounge
- [ ] update portal gun descriptions based on when opened, battery taken, inserted

### aperture-lab
- [ ] burnt invitation -- change charred paper bits to "flaming" to encourage foamsafe
- [ ] have fire alarm continue for a bit... also to nudge toward foamsafe use
- [ ] "engraved" invitation --> "piping hot invitation" --> engraved after a long delay


### foam fire
- [ ] typo "extinguised"

### vip-bathroom
- [ ] need flavor item or maybe an Initech hand dryer that does something terribly wrong
- [ ] flavor triggers: when enter bathroom, if nauseated->run to toilet and hurl, if bad-breath, look for a mint or toothbrush, rinse mouth

### poetry-panic
- [ ] desc: "probably almost"
- [ ] some other item here...  teleporter bracelet??

(saved "playtest")
