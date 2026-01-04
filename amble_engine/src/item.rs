//! Item types and related helpers.
//!
//! Items represent objects the player can interact with. Some may act as
//! containers for other items. Functions here handle display logic and
//! movement between locations.

use crate::{Location, View, ViewItem, WorldObject, style::GameStyle, view::ContentLine, world::AmbleWorld};

use anyhow::{Context, Result};
use colored::Colorize;

use crate::Id;
use log::info;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
use variantly::Variantly;

/// Anything in '`AmbleWorld`' that can be inspected or manipulated apart from NPCs.
///
/// Some 'Items' can also act as containers for other items, if '`container_state`' is 'Some(_)'.
/// `symbol` and `id` refer to the same string -- a holdover from a previous engine version where they differed.
///
/// `movability` specifies whether the `Item` can change location, and why not if it can't.
///
/// 'abilities' are special things you can do with this item (e.g. read, smash, ignite, clean)
///
/// '`interaction_requires`' maps a type of interaction (a thing that can be done to this item using another item) to an ability.
///     e.g. `ItemInteractionType::Burn` => `ItemAbility::Ignite`
///
/// Combined with an appropriate ActOnItem-based trigger, this would mean any Item with ItemAbility::Ignite can be
/// used to Burn this item -- and importantly _only_ items with that ability can be used to Burn it.
///
/// 'consumable' makes an item consumable if present, with number of uses and what to do when
/// all uses are expended defined in `ConsumableOpts`.
///
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Item {
    /// The stable id of this item.
    pub id: Id,
    /// The symbol used to refer to this item in world data.
    pub symbol: String,
    /// The display name of the item.
    pub name: String,
    /// A general description of the item.
    pub description: String,
    /// The current `Location` of the item.
    pub location: Location,
    /// Determines whether the item can be moved from its current location.
    pub movability: Movability,
    /// Some state (open, locked, etc) for the item as a container, or `None` if it is not a container.
    pub container_state: Option<ContainerState>,
    /// Set of ids of other items contained by this one.
    pub contents: HashSet<Id>,
    /// Set of capabilities [`ItemAbility`] for this item.
    pub abilities: HashSet<ItemAbility>,
    /// Relates interactions to abilities. (Ex: to perform the "burn" interaction targeting this item, the other item must have the "ignite" capability.)
    pub interaction_requires: HashMap<ItemInteractionType, ItemAbility>,
    /// Any legible detail text on the item. **Also used as the detail text for the "examine" command.**
    pub text: Option<String>,
    /// Some consumable parameters [`ConsumableOpts`], or None it the item isn't consumable.
    pub consumable: Option<ConsumableOpts>,
}

impl WorldObject for Item {
    fn id(&self) -> Id {
        self.id.clone()
    }
    fn symbol(&self) -> &str {
        &self.symbol
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn description(&self) -> &str {
        &self.description
    }
    fn location(&self) -> &Location {
        &self.location
    }
}

impl ItemHolder for Item {
    fn add_item(&mut self, item_id: Id) {
        // ensure this is a container and disallow placing an item inside itself
        if self.container_state.is_some() && self.id.ne(&item_id) {
            self.contents.insert(item_id);
        }
    }
    fn remove_item(&mut self, item_id: Id) {
        if self.container_state.is_some() {
            self.contents.remove(&item_id);
        }
    }
    fn contains_item(&self, item_id: Id) -> bool {
        self.contents.contains(&item_id)
    }
}

impl Item {
    /// Returns true if item is consumable and has been consumed.
    pub fn is_consumed(&self) -> bool {
        match &self.consumable {
            Some(opts) => opts.uses_left == 0,
            None => false,
        }
    }

    /// Returns true if item's contents can be accessed directly.
    pub fn is_accessible(&self) -> bool {
        self.container_state
            .is_some_and(|cs| cs.is_open() || cs.is_transparent_open())
    }

    /// Returns true if the item is a container and its contents are visible even when closed or locked.
    pub fn is_transparent(&self) -> bool {
        self.container_state
            .is_some_and(|cs| cs.is_transparent_closed() || cs.is_transparent_locked() || cs.is_transparent_open())
    }
    /// Set location to a `Room` by id.
    pub fn set_location_room(&mut self, room_id: Id) {
        self.location = Location::Room(room_id);
    }
    /// Set location to inside another container `Item` by id.
    pub fn set_location_item(&mut self, container_id: Id) {
        self.location = Location::Item(container_id);
    }
    /// Set location to player inventory
    pub fn set_location_inventory(&mut self) {
        // once a restricted item has been obtained, it must be unrestricted (or else the player wouldn't
        // be able to pick it up again after dropping it)
        // if given back to an NPC or "locked in" to a receiver item,
        // it can be optionally re-restricted using a trigger action
        if matches!(self.movability, Movability::Restricted { .. }) {
            self.movability = Movability::Free;
        }
        self.location = Location::Inventory;
    }
    /// Set location to NPC inventory by id.
    pub fn set_location_npc(&mut self, npc_id: Id) {
        self.location = Location::Npc(npc_id);
    }
    /// Show item description (and any contents if a container and open).
    pub fn show(&self, world: &AmbleWorld, view: &mut View) {
        // push general desccription to View
        view.push(ViewItem::ItemDescription {
            name: self.name.clone(),
            description: self.description.clone(),
        });

        // push any consumable status to View
        if let Some(opts) = &self.consumable {
            let consuming_abilities: Vec<_> = opts.consume_on.iter().map(ItemAbility::to_string).collect();
            let uses = consuming_abilities.join(" or ").underline();
            view.push(ViewItem::ItemConsumableStatus(format!(
                "You can {uses} this item {} more time{}.",
                opts.uses_left.to_string().yellow(),
                if opts.uses_left == 1 { "" } else { "s" }
            )));
        }

        // handle container / contained item display
        if self.container_state.is_some() {
            if self.is_accessible() || self.is_transparent() {
                self.show_contents(world, view);
                if self.is_transparent() {
                    self.show_transparency_note(view);
                }
            } else {
                self.show_contents_obscured(view);
            }
        }
    }

    fn show_contents_obscured(&self, view: &mut View) {
        let action = if self.container_state.is_some_and(|cs| cs.is_locked()) {
            "unlock".bold().red()
        } else {
            "open".bold().green()
        };
        view.push(ViewItem::ActionFailure(format!(
            "You must {action} it to see what's inside."
        )));
    }

    fn show_transparency_note(&self, view: &mut View) {
        let action = if self
            .container_state
            .is_some_and(|cs| cs.is_locked() || cs.is_transparent_locked())
        {
            "unlock".bold().red()
        } else {
            "open".bold().green()
        };
        view.push(ViewItem::ActionFailure(format!(
            "You can see inside, but you must {action} it to access the contents."
        )));
    }

    fn show_contents(&self, world: &AmbleWorld, view: &mut View) {
        if self.contents.is_empty() {
            view.push(ViewItem::ItemContents(Vec::new()));
        } else {
            view.push(ViewItem::ItemContents(
                self.contents
                    .iter()
                    .filter_map(|id| world.items.get(id))
                    .map(|i| ContentLine {
                        item_name: i.name.clone(),
                        restricted: matches!(i.movability, Movability::Restricted { .. }),
                    })
                    .collect(),
            ));
        }
    }

    /// Determine what ability is required for certain interactions with this item.
    ///
    /// In `<verb> <target> with <tool>` commands, this returns whatever ability the `<tool>` must have
    /// in order to successfully `<verb>` the `<target>` (this item).
    ///
    /// Example -- if this item is `candle`, then:
    ///
    /// `candle.requires_capability_for(ItemInteractionType::Burn)`
    /// might return `Some(ItemAbility::Ignite)`.
    pub fn requires_capability_for(&self, inter: ItemInteractionType) -> Option<ItemAbility> {
        self.interaction_requires.get(&inter).cloned()
    }

    /// Returns the reason the item can't be accessed (as a container), if any
    pub fn access_denied_reason(&self) -> Option<String> {
        match self.container_state {
            Some(ContainerState::Open | ContainerState::TransparentOpen) => None,
            Some(ContainerState::Closed) => {
                let reason = format!("The {} is {}.", self.name().item_style(), "closed".bold());
                Some(reason)
            },
            Some(ContainerState::Locked) => {
                let reason = format!("The {} is {}.", self.name().item_style(), "locked".bold());
                Some(reason)
            },
            Some(ContainerState::TransparentClosed) => {
                let reason = format!(
                    "The {} is {}. You can see inside but can't access the contents.",
                    self.name().item_style(),
                    "closed".bold()
                );
                Some(reason)
            },
            Some(ContainerState::TransparentLocked) => {
                let reason = format!(
                    "The {} is {}. You can see inside but can't access the contents.",
                    self.name().item_style(),
                    "locked".bold()
                );
                Some(reason)
            },
            None => {
                let reason = format!("The {} isn't a container.", self.name().item_style());
                Some(reason)
            },
        }
    }

    /// Returns the reason an item can't be taken into inventory, if any
    pub fn take_denied_reason(&self) -> Option<String> {
        match &self.movability {
            Movability::Fixed { reason } | Movability::Restricted { reason } => Some(reason.clone()),
            Movability::Free => None,
        }
    }
}

/// Consumes one use of the item with the specified ability.
///
/// # Arguments
/// * `world` - Mutable reference to the game world
/// * `item_id` - id of the item to consume
/// * `ability` - The ability that triggered the consumption (e.g. `ItemAbility::Ignite`)
///
/// # Returns
/// * `Ok(Some(uses_left))` - Item was consumable and consumed, returns remaining uses
/// * `Ok(None)` - Item is not consumable or ability doesn't trigger consumption
/// * `Err(_)` - Item lookup failed
///
/// # Errors
/// * Returns error if the item id is not found in world.items
/// * Context will include the item id that failed lookup
pub fn consume(world: &mut AmbleWorld, item_id: &Id, ability: ItemAbility) -> Result<Option<usize>> {
    let item = world
        .items
        .get_mut(item_id)
        .with_context(|| format!("failed lookup trying to consume() item '{item_id}'"))?;

    let item_id = item.id.clone();
    let item_sym = item.symbol.clone();

    // if consumable, decrement and set to # of remaining uses
    // if not consumable, return early with None
    let (uses_left, when_consumed) = if let Some(opts) = &mut item.consumable {
        // decrement uses_left if right ability was used
        if opts.consume_on.contains(&ability) && opts.uses_left > 0 {
            opts.uses_left -= 1;
        }
        (opts.uses_left, opts.when_consumed.clone())
    } else {
        return Ok(None);
    };

    // if uses_left is now zero, handle the consumption, current options are just to despawn,
    // or to despawn and replace with another item either in inventory or the current room
    if uses_left == 0 {
        // release the mutable borrow on the item before performing actions that
        // require another mutable borrow of the world
        let _ = item;

        match when_consumed {
            ConsumeType::ReplaceInventory { replacement } => {
                crate::trigger::despawn_item(world, &item_id)?;
                crate::trigger::spawn_item_in_inventory(world, &replacement)?;
            },
            ConsumeType::ReplaceCurrentRoom { replacement } => {
                crate::trigger::despawn_item(world, &item_id)?;
                crate::trigger::spawn_item_in_current_room(world, &replacement)?;
            },
            ConsumeType::Despawn => crate::trigger::despawn_item(world, &item_id)?,
        }
    }
    info!("used ({ability}) ability of consumable item '{item_sym}': {uses_left} uses left");
    Ok(Some(uses_left))
}

/// Determine whether a tool item satisfies requirements for an interaction on a target item.
pub fn interaction_requirement_met(interaction: ItemInteractionType, target: &Item, tool: &Item) -> bool {
    if let Some(requirement) = target.interaction_requires.get(&interaction) {
        tool.abilities.contains(requirement)
    } else {
        true
    }
}

/// Modes of using ingestible items.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IngestMode {
    Eat,
    Drink,
    Inhale,
}
impl Display for IngestMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IngestMode::Eat => write!(f, "eat"),
            IngestMode::Drink => write!(f, "drink"),
            IngestMode::Inhale => write!(f, "inhale"),
        }
    }
}

/// Methods common to things that can hold items.
pub trait ItemHolder {
    /// Insert an item into the holder's contents.
    fn add_item(&mut self, item_id: Id);
    /// Remove an item from the holder's contents.
    fn remove_item(&mut self, item_id: Id);
    /// Return `true` when the holder already contains the given item.
    fn contains_item(&self, item_id: Id) -> bool;
}

/// Things an item can do.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ItemAbility {
    Attach,
    Clean,
    Cut,
    CutWood,
    Drink,
    Eat,
    Extinguish,
    Ignite,
    Inhale,
    Insulate,
    Magnify,
    Pluck,
    Pry,
    Read,
    Repair,
    Sharpen,
    Smash,
    TurnOn,
    TurnOff,
    Unlock(Option<Id>),
    Use,
}
impl Display for ItemAbility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Attach => write!(f, "attach"),
            Self::Clean => write!(f, "clean"),
            Self::Cut => write!(f, "cut"),
            Self::CutWood => write!(f, "cut wood"),
            Self::Drink => write!(f, "drink"),
            Self::Eat => write!(f, "eat"),
            Self::Extinguish => write!(f, "extinguish"),
            Self::Ignite => write!(f, "ignite"),
            Self::Inhale => write!(f, "inhale"),
            Self::Insulate => write!(f, "insulate"),
            Self::Magnify => write!(f, "magnify"),
            Self::Read => write!(f, "read"),
            Self::Repair => write!(f, "repair"),
            Self::Sharpen => write!(f, "sharpen"),
            Self::TurnOn => write!(f, "turn on"),
            Self::TurnOff => write!(f, "turn off"),
            Self::Unlock(_) => write!(f, "unlock"),
            Self::Use => write!(f, "use"),
            Self::Pluck => write!(f, "pluck"),
            Self::Pry => write!(f, "pry"),
            Self::Smash => write!(f, "smash"),
        }
    }
}

/// Things you can do to an item, but only with certain other items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ItemInteractionType {
    Attach,
    Break,
    Burn,
    Extinguish,
    Clean,
    Cover,
    Cut,
    Handle,
    Move,
    Open,
    Repair,
    Sharpen,
    Turn,
    Unlock,
}

impl Display for ItemInteractionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Attach => write!(f, "attach"),
            Self::Break => write!(f, "break"),
            Self::Burn => write!(f, "burn"),
            Self::Extinguish => write!(f, "extinguish"),
            Self::Clean => write!(f, "clean"),
            Self::Cover => write!(f, "cover"),
            Self::Cut => write!(f, "cut"),
            Self::Handle => write!(f, "handle"),
            Self::Move => write!(f, "move"),
            Self::Open => write!(f, "open"),
            Self::Repair => write!(f, "repair"),
            Self::Sharpen => write!(f, "sharpen"),
            Self::Turn => write!(f, "turn"),
            Self::Unlock => write!(f, "unlock"),
        }
    }
}

/// All of the valid states a container can be in.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Variantly)]
#[serde(rename_all = "camelCase")]
pub enum ContainerState {
    Open,
    Closed,
    Locked,
    TransparentOpen,
    TransparentClosed,
    TransparentLocked,
}

/// Possible movability states for an `Item`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Movability {
    Fixed {
        reason: String,
    },
    Restricted {
        reason: String,
    },
    #[default]
    Free,
}

/// Extra options / data for consumable items are represented here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsumableOpts {
    pub uses_left: usize,
    pub consume_on: HashSet<ItemAbility>,
    pub when_consumed: ConsumeType,
}

/// Types of things that can happen when an item has been consumed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConsumeType {
    Despawn,
    ReplaceInventory { replacement: Id },   // put replacement in inventory
    ReplaceCurrentRoom { replacement: Id }, // put replacement in current room
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Id;
    use crate::world::{AmbleWorld, Location};
    use std::collections::{HashMap, HashSet};

    fn create_test_item(id: Id) -> Item {
        Item {
            id,
            symbol: "test_item".into(),
            name: "Test Item".into(),
            description: "A test item".into(),
            location: Location::Nowhere,
            movability: Movability::Free,
            container_state: None,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        }
    }

    fn create_test_world() -> AmbleWorld {
        let mut world = AmbleWorld::new_empty();

        let item_id = crate::idgen::new_id();
        let item = create_test_item(item_id.clone());
        world.items.insert(item_id.clone(), item);

        world
    }

    #[test]
    fn item_is_consumed_returns_false_for_non_consumable() {
        let item = create_test_item(crate::idgen::new_id());
        assert!(!item.is_consumed());
    }

    #[test]
    fn item_is_consumed_returns_false_for_unconsumed_consumable() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.consumable = Some(ConsumableOpts {
            uses_left: 3,
            consume_on: HashSet::new(),
            when_consumed: ConsumeType::Despawn,
        });
        assert!(!item.is_consumed());
    }

    #[test]
    fn item_is_consumed_returns_true_for_consumed_consumable() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.consumable = Some(ConsumableOpts {
            uses_left: 0,
            consume_on: HashSet::new(),
            when_consumed: ConsumeType::Despawn,
        });
        assert!(item.is_consumed());
    }

    #[test]
    fn item_is_accessible_returns_false_for_non_container() {
        let item = create_test_item(crate::idgen::new_id());
        assert!(!item.is_accessible());
    }

    #[test]
    fn item_is_accessible_returns_true_for_open_container() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.container_state = Some(ContainerState::Open);
        assert!(item.is_accessible());
    }

    #[test]
    fn item_is_accessible_returns_false_for_closed_container() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.container_state = Some(ContainerState::Closed);
        assert!(!item.is_accessible());
    }

    #[test]
    fn item_is_accessible_returns_false_for_locked_container() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.container_state = Some(ContainerState::Locked);
        assert!(!item.is_accessible());
    }

    #[test]
    fn item_is_accessible_returns_false_for_transparent_closed_container() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.container_state = Some(ContainerState::TransparentClosed);
        assert!(!item.is_accessible());
    }

    #[test]
    fn item_is_accessible_returns_false_for_transparent_locked_container() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.container_state = Some(ContainerState::TransparentLocked);
        assert!(!item.is_accessible());
    }

    #[test]
    fn item_is_transparent_returns_true_for_transparent_containers() {
        let mut item = create_test_item(crate::idgen::new_id());

        item.container_state = Some(ContainerState::TransparentClosed);
        assert!(item.is_transparent());

        item.container_state = Some(ContainerState::TransparentLocked);
        assert!(item.is_transparent());

        item.container_state = Some(ContainerState::Closed);
        assert!(!item.is_transparent());

        item.container_state = Some(ContainerState::Locked);
        assert!(!item.is_transparent());

        item.container_state = Some(ContainerState::Open);
        assert!(!item.is_transparent());

        item.container_state = None;
        assert!(!item.is_transparent());
    }

    #[test]
    fn set_location_room_updates_location() {
        let mut item = create_test_item(crate::idgen::new_id());
        let room_id = crate::idgen::new_id();
        item.set_location_room(room_id.clone());
        assert_eq!(item.location, Location::Room(room_id));
    }

    #[test]
    fn set_location_item_updates_location() {
        let mut item = create_test_item(crate::idgen::new_id());
        let container_id = crate::idgen::new_id();
        item.set_location_item(container_id.clone());
        assert_eq!(item.location, Location::Item(container_id));
    }

    #[test]
    fn set_location_inventory_updates_location_and_unrestricts() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.movability = Movability::Restricted {
            reason: "You haven't earned it yet.".to_string(),
        };
        item.set_location_inventory();
        assert_eq!(item.location, Location::Inventory);
        assert!(matches!(item.movability, Movability::Free));
    }

    #[test]
    fn set_location_inventory_preserves_fixed_movability() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.movability = Movability::Fixed {
            reason: "Bolted down.".to_string(),
        };
        item.set_location_inventory();
        assert_eq!(item.location, Location::Inventory);
        assert!(matches!(item.movability, Movability::Fixed { .. }));
    }

    #[test]
    fn set_location_npc_updates_location() {
        let mut item = create_test_item(crate::idgen::new_id());
        let npc_id = crate::idgen::new_id();
        item.set_location_npc(npc_id.clone());
        assert_eq!(item.location, Location::Npc(npc_id));
    }

    #[test]
    fn requires_capability_for_returns_none_for_no_requirement() {
        let item = create_test_item(crate::idgen::new_id());
        assert_eq!(item.requires_capability_for(ItemInteractionType::Break), None);
    }

    #[test]
    fn requires_capability_for_returns_ability_when_required() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.interaction_requires
            .insert(ItemInteractionType::Break, ItemAbility::Smash);
        assert_eq!(
            item.requires_capability_for(ItemInteractionType::Break),
            Some(ItemAbility::Smash)
        );
    }

    #[test]
    fn access_denied_reason_returns_none_for_open_container() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.container_state = Some(ContainerState::Open);
        assert_eq!(item.access_denied_reason(), None);
    }

    #[test]
    fn access_denied_reason_returns_reason_for_closed_container() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.container_state = Some(ContainerState::Closed);
        let reason = item.access_denied_reason().unwrap();
        assert!(reason.contains("closed"));
    }

    #[test]
    fn access_denied_reason_returns_reason_for_locked_container() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.container_state = Some(ContainerState::Locked);
        let reason = item.access_denied_reason().unwrap();
        assert!(reason.contains("locked"));
    }

    #[test]
    fn access_denied_reason_returns_reason_for_transparent_closed_container() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.container_state = Some(ContainerState::TransparentClosed);
        let reason = item.access_denied_reason().unwrap();
        assert!(reason.contains("closed"));
        assert!(reason.contains("see inside"));
    }

    #[test]
    fn access_denied_reason_returns_reason_for_transparent_locked_container() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.container_state = Some(ContainerState::TransparentLocked);
        let reason = item.access_denied_reason().unwrap();
        assert!(reason.contains("locked"));
        assert!(reason.contains("see inside"));
    }

    #[test]
    fn access_denied_reason_returns_reason_for_non_container() {
        let item = create_test_item(crate::idgen::new_id());
        let reason = item.access_denied_reason().unwrap();
        assert!(reason.contains("isn't a container"));
    }

    #[test]
    fn take_denied_reason_returns_none_for_movable() {
        let item = create_test_item(crate::idgen::new_id());
        assert_eq!(item.take_denied_reason(), None);
    }

    #[test]
    fn take_denied_reason_returns_reason_for_fixed() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.movability = Movability::Fixed {
            reason: "The item isn't portable.".to_string(),
        };
        let reason = item.take_denied_reason().unwrap();
        assert!(reason.contains("isn't portable"));
    }

    #[test]
    fn take_denied_reason_returns_reason_for_restricted() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.movability = Movability::Restricted {
            reason: "You can't take that yet.".to_string(),
        };
        let reason = item.take_denied_reason().unwrap();
        assert!(reason.contains("can't take"));
    }

    #[test]
    fn item_holder_add_item_works() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.container_state = Some(ContainerState::Open);
        let item_to_add = crate::idgen::new_id();

        item.add_item(item_to_add.clone());
        assert!(item.contents.contains(&item_to_add));
    }

    #[test]
    fn item_holder_add_item_ignores_self_reference() {
        let item_id = crate::idgen::new_id();
        let mut item = create_test_item(item_id.clone());
        item.container_state = Some(ContainerState::Open);

        item.add_item(item_id.clone());
        assert!(!item.contents.contains(&item_id));
    }

    #[test]
    fn item_holder_add_item_ignores_non_container() {
        let mut item = create_test_item(crate::idgen::new_id());
        let item_to_add = crate::idgen::new_id();

        item.add_item(item_to_add.clone());
        assert!(!item.contents.contains(&item_to_add));
    }

    #[test]
    fn item_holder_remove_item_works() {
        let mut item = create_test_item(crate::idgen::new_id());
        item.container_state = Some(ContainerState::Open);
        let item_to_remove = crate::idgen::new_id();
        item.contents.insert(item_to_remove.clone());

        item.remove_item(item_to_remove.clone());
        assert!(!item.contents.contains(&item_to_remove));
    }

    #[test]
    fn item_holder_contains_item_works() {
        let mut item = create_test_item(crate::idgen::new_id());
        let contained_item = crate::idgen::new_id();
        item.contents.insert(contained_item.clone());

        assert!(item.contains_item(contained_item));
        assert!(!item.contains_item(crate::idgen::new_id()));
    }

    #[test]
    fn consume_returns_none_for_non_consumable() {
        let mut world = create_test_world();
        let item_id = world.items.keys().next().unwrap().clone();

        let result = consume(&mut world, &item_id, ItemAbility::Use).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn consume_decrements_uses_for_correct_ability() {
        let mut world = create_test_world();
        let item_id = world.items.keys().next().unwrap().clone();

        let mut consume_on = HashSet::new();
        consume_on.insert(ItemAbility::Ignite);

        world.items.get_mut(&item_id).unwrap().consumable = Some(ConsumableOpts {
            uses_left: 3,
            consume_on,
            when_consumed: ConsumeType::Despawn,
        });

        let result = consume(&mut world, &item_id, ItemAbility::Ignite).unwrap();
        assert_eq!(result, Some(2));
        assert_eq!(world.items[&item_id].consumable.as_ref().unwrap().uses_left, 2);
    }

    #[test]
    fn consume_does_not_decrement_for_wrong_ability() {
        let mut world = create_test_world();
        let item_id = world.items.keys().next().unwrap().clone();

        let mut consume_on = HashSet::new();
        consume_on.insert(ItemAbility::Ignite);

        world.items.get_mut(&item_id).unwrap().consumable = Some(ConsumableOpts {
            uses_left: 3,
            consume_on,
            when_consumed: ConsumeType::Despawn,
        });

        let result = consume(&mut world, &item_id, ItemAbility::Use).unwrap();
        assert_eq!(result, Some(3));
        assert_eq!(world.items[&item_id].consumable.as_ref().unwrap().uses_left, 3);
    }

    #[test]
    fn container_state_is_open_works() {
        assert!(ContainerState::Open.is_open());
        assert!(!ContainerState::Closed.is_open());
        assert!(!ContainerState::Locked.is_open());
    }

    #[test]
    fn item_ability_display_works() {
        assert_eq!(format!("{}", ItemAbility::Attach), "attach");
        assert_eq!(format!("{}", ItemAbility::Clean), "clean");
        assert_eq!(format!("{}", ItemAbility::CutWood), "cut wood");
        assert_eq!(format!("{}", ItemAbility::Extinguish), "extinguish");
        assert_eq!(format!("{}", ItemAbility::TurnOn), "turn on");
        assert_eq!(format!("{}", ItemAbility::TurnOff), "turn off");
        assert_eq!(format!("{}", ItemAbility::Unlock(None)), "unlock");
    }

    #[test]
    fn item_interaction_type_display_works() {
        assert_eq!(format!("{}", ItemInteractionType::Attach), "attach");
        assert_eq!(format!("{}", ItemInteractionType::Break), "break");
        assert_eq!(format!("{}", ItemInteractionType::Burn), "burn");
        assert_eq!(format!("{}", ItemInteractionType::Extinguish), "extinguish");
        assert_eq!(format!("{}", ItemInteractionType::Clean), "clean");
        assert_eq!(format!("{}", ItemInteractionType::Cover), "cover");
    }

    #[test]
    fn world_object_trait_works() {
        let item = create_test_item(crate::idgen::new_id());
        assert_eq!(item.id(), item.id);
        assert_eq!(item.symbol(), "test_item");
        assert_eq!(item.name(), "Test Item");
        assert_eq!(item.description(), "A test item");
        assert_eq!(item.location(), &Location::Nowhere);
    }
}
