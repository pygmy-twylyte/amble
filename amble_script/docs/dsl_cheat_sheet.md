# Amble Script DSL Cheat Sheet

Keep this quick reference open while authoring `.amble` files. It summarises the most common keywords, shapes, and options supported by the `amble_script` compiler.

---

## CLI Commands

| Command | Purpose | Key Flags |
| --- | --- | --- |
| `amble_script compile <file>` | Compile a single DSL file. | `--out-triggers`, `--out-rooms`, `--out-items`, `--out-spinners`, `--out-npcs`, `--out-goals` |
| `amble_script compile-dir <dir> --out-dir <data>` | Compile every `.amble`/`.able` file under `<dir>` into `<data>/*.toml`. | `--only triggers,rooms,…`, `--verbose` |
| `amble_script lint <file|dir>` | Validate references against `amble_engine/data`. | `--data-dir <dir>`, `--deny-missing` |

---

## Trigger Essentials

```amble
trigger "Name" when <event> {
  [note "Debug note"]
  [only once]
  [if <condition> { <actions> } ...]
  [do <action>]
}
```

**Events:**

- `enter room <room>` | `leave room <room>`
- `take item <item>` | `drop item <item>` | `look at item <item>` | `open item <item>` | `unlock item <item>`
- `use item <item> ability <ability>` | `act <verb> on item <item>` | `insert item <item> into item <container>`
- `take item <item> from npc <npc>` | `give item <item> to npc <npc>` | `talk to npc <npc>`
- `always`

**Condition atoms:**

- Flags: `has flag`, `missing flag`, `flag in progress`, `flag complete`
- Items: `has item`, `missing item`, `container <container> has item <item>`
- Location: `player in room`, `has visited room`
- NPCs: `with npc`, `npc has item`, `npc in state`
- Random: `chance <n>%`, `in rooms <r1,r2,…>`
- Groups: `all(...)`, `any(...)`

**Action atoms:**

- Feedback: `do show`, `do award points`
- Flags: `do add/remove/reset/advance flag`, `do add seq flag [limit n]`
- Spawn/Despawn: `do spawn item … into room|container|inventory|current room`, `do despawn item`
- Exits & locks: `do reveal/lock/unlock exit`, `do lock/unlock item`, `do set barred message`
- NPC dialogue/state: `do npc says`, `do npc says random`, `do npc refuse item`, `do set npc state`
- Item tweaks: `do set item description`
- Player/world: `do push player to`, `do restrict item`, `do deny read`
- Spinners: `do spinner message <spinner>`, `do add wedge "…" width <n> spinner <spinner>`
- Scheduling: `do schedule in/on <n> { … }`, `do schedule in/on … if <cond> onFalse <policy> [note "…"] { … }`

**OnFalse policies:** `cancel`, `retryAfter <turns>`, `retryNextTurn`

**Sets:** `let set <name> = (<room_a>, <room_b>, …)` then `if in rooms <name>`

---

## Room Essentials

```amble
room <id> {
  name "Title"
  desc "Base description"
  [visited true|false]
  [exit <dir> <to> [hidden] [locked] [barred "…"] [required_items(a,b)] [required_flags(flag_a,flag_b#3)]]
  [overlay if <conditions> { text "…" }]
}
```

Overlay conditions: `flag set`, `flag unset`, `flag complete`, `item present`, `item absent`, `player has item`, `player missing item`, `npc present`, `npc absent`, `npc in state`, `item in room`.

---

## Item Essentials

```amble
item <id> {
  name "Name"
  desc "Description"
  portable true|false
  location inventory <owner>|room <room>|npc <npc>|chest <container>|nowhere "note"
  [container state open|closed|locked|transparentClosed|transparentLocked]
  [restricted true]
  [ability <Ability> [<target>]]
  [text "Readable text"]
  [requires <ability> to <interaction>]
}
```

---

## NPC Essentials

```amble
npc <id> {
  name "Name"
  desc "Description"
  location room <room>|nowhere "note"
  [state <ident>|state custom <id>]
  [movement random|route rooms (<r1, r2, …>) [timing <schedule>] [active true|false] [loop true|false]]
  [dialogue <state>|dialogue custom <id> { "Line" "Line" }]
}
```

---

## Spinner Essentials

```amble
spinner <id> {
  wedge "Text" [width <n>]
  …
}
```

Width defaults to 1 when omitted.

---

## Goal Essentials

```amble
goal <id> {
  name "Display name"
  desc "Description"
  group required|optional|status-effect
  [activate when <condition>]
  complete when <condition>
  [fail when <condition>]
}
```

Goal conditions: `has flag`, `missing flag`, `has item`, `reached room`, `goal complete <other>`, `flag in progress`, `flag complete`.

---

## Handy Commands

```bash
# Lint everything, fail on missing references
cargo run -p amble_script -- lint content/ --deny-missing

# Compile entire content set into the engine data directory
cargo run -p amble_script -- compile-dir content/ --out-dir amble_engine/data
```

Happy world-building!
