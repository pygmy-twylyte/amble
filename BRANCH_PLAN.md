# The Goal

Replace the Item `portable` and `restricted` bools with a single `ItemMovability` state.
- prevents nonsensical combinations
- allows attaching strings to give individualized reasons that something is fixed or restricted.

# Engine Work
- [ ] Create ItemMovability enum
- [ ] replace the Item portable/restricted fields with it
- [ ] update take and drop handlers
- [ ] update give to npc and take from npc handlers
- [ ] update insert item handlers
- [ ] update modify item handler / item patch
- [ ] remove restrict item action 
- [ ] trigger action to set movability 

# DSL Work
- [ ] remove portable / restricted statements from item definitions
- [ ] add movability to replace those
- [ ] same for item patch statements
- [ ] new statement for set movability action

# Content Work
- [ ] ALL item definitions will have to be updated
- [ ] Item patches and trigger actions will have to be checked and updated


# DSL sketches

For `Item` defs:
```
item some_item {
    ..
    movability fixed "reason" 
    or
    movability restricted "reason"
    or
    movability free
}

trigger "Some Trigger" when always {
    do set item movability free | fixed "reason" | restricted "reason"
}
```
