//! Loading logic for [`Item`] definitions.
//!
//! Items are first parsed into [`RawItem`] structures and later converted into
//! fully linked [`Item`] instances during world initialization.

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result, anyhow, bail};
use log::info;
use serde::Deserialize;
use toml;
use uuid::Uuid;

use crate::{
    ItemHolder, Location,
    idgen::{NAMESPACE_ITEM, uuid_from_token},
    item::{ConsumableOpts, ConsumeType, ContainerState, Item, ItemAbility, ItemInteractionType},
    world::AmbleWorld,
};

use super::{SymbolTable, resolve_location};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", content = "target")]
pub enum RawItemAbility {
    Attach,
    Clean,
    CutWood,
    Ignite,
    Insulate,
    Pluck,
    Pry,
    Read,
    Repair,
    Sharpen,
    Smash,
    TurnOn,
    TurnOff,
    Unlock(Option<String>),
    Use,
}
impl RawItemAbility {
    pub fn to_ability(&self, symbols: &SymbolTable) -> Result<ItemAbility> {
        let ability = match self {
            Self::Attach => ItemAbility::Attach,
            Self::Clean => ItemAbility::Clean,
            Self::CutWood => ItemAbility::CutWood,
            Self::Ignite => ItemAbility::Ignite,
            Self::Insulate => ItemAbility::Insulate,
            Self::Pluck => ItemAbility::Pluck,
            Self::Pry => ItemAbility::Pry,
            Self::Read => ItemAbility::Read,
            Self::Repair => ItemAbility::Repair,
            Self::Sharpen => ItemAbility::Sharpen,
            Self::Smash => ItemAbility::Smash,
            Self::TurnOn => ItemAbility::TurnOn,
            Self::TurnOff => ItemAbility::TurnOff,
            Self::Unlock(Some(sym)) => {
                let target = symbols
                    .items
                    .get(sym)
                    .with_context(|| format!("raw item ability Unlock({sym}): target not found in symbol table"))?;
                ItemAbility::Unlock(Some(*target))
            },
            Self::Unlock(None) => ItemAbility::Unlock(None),
            Self::Use => ItemAbility::Use,
        };
        Ok(ability)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RawConsumableOpts {
    pub uses_left: usize,
    pub consume_on: HashSet<RawItemAbility>,
    pub when_consumed: RawConsumeType,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RawConsumeType {
    Despawn,
    ReplaceInventory { replacement: String },
    ReplaceCurrentRoom { replacement: String },
}
impl RawConsumeType {
    pub fn to_consume_type(&self, symbols: &SymbolTable) -> Result<ConsumeType> {
        match self {
            Self::Despawn => Ok(ConsumeType::Despawn),
            Self::ReplaceInventory { replacement } => {
                let replacement_uuid = symbols
                    .items
                    .get(replacement)
                    .with_context(|| format!("looking up item symbol '{replacement}'"))?;
                Ok(ConsumeType::ReplaceInventory {
                    replacement: *replacement_uuid,
                })
            },
            Self::ReplaceCurrentRoom { replacement } => {
                let replacement_uuid = symbols
                    .items
                    .get(replacement)
                    .with_context(|| format!("looking up item symbol '{replacement}'"))?;
                Ok(ConsumeType::ReplaceCurrentRoom {
                    replacement: *replacement_uuid,
                })
            },
        }
    }
}

/// First stage of loading an `Item` from the items TOML file.
/// In the TOML, id(token), name, description, portable, and location are all mandatory.
/// Token IDs (e.g. "towel") are converted to UUIDs before second stage (placement)
/// `Container`, open, and locked only need be defined for containers (all default false).
/// `Contents` are populated dynamically from other item entries and should not be in the TOML at all.
/// `Restricted` is for puzzle/quest items that can't simply be "taken" from an NPC
/// `Abilities` designate additional abilities implemented in triggers (e.g. `TurnOn`)
/// `Interaction_requires`: e.g. to "burn" x with y, x requires y to have "ignite" capability
/// Text contains anything readable on the item or None.
#[derive(Debug, Deserialize)]
pub struct RawItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub portable: bool,
    pub container_state: Option<ContainerState>,
    #[serde(default)]
    pub restricted: bool,
    pub location: HashMap<String, String>,
    #[serde(default)]
    pub contents: HashSet<Uuid>,
    #[serde(default)]
    pub abilities: HashSet<RawItemAbility>,
    #[serde(default)]
    pub interaction_requires: HashMap<ItemInteractionType, ItemAbility>,
    #[serde(default)]
    pub text: Option<String>,
    pub consumable: Option<RawConsumableOpts>,
}
impl RawItem {
    /// Converts a `RawItem` loaded from TOML to an `Item` object.
    /// # Errors
    /// - if location of the item cannot be resolved from the loaded data
    /// - if the `RawItem`s is not found in the symbol table
    pub fn to_item(&self, symbols: &SymbolTable) -> Result<Item> {
        let loc = resolve_location(&self.location, symbols)?;
        let item_uuid = match symbols.items.get(&self.id) {
            Some(id) => *id,
            None => {
                return Err(anyhow!("item {} ({}) not found in symbol table", self.id, self.name));
            },
        };

        // set up consumable options if present for this item
        let consumable = if let Some(opts) = &self.consumable {
            let mut consume_on = HashSet::new();
            for ability in &opts.consume_on {
                _ = consume_on.insert(ability.to_ability(symbols)?);
            }
            let when_consumed = opts.when_consumed.to_consume_type(symbols)?;
            Some(ConsumableOpts {
                uses_left: opts.uses_left,
                consume_on,
                when_consumed,
            })
        } else {
            None
        };

        let mut abilities = HashSet::new();
        for raw_ability in &self.abilities {
            let ability = raw_ability.to_ability(symbols)?;
            abilities.insert(ability);
        }

        let real_item = Item {
            id: item_uuid,
            symbol: self.id.to_string(),
            name: self.name.to_string(),
            description: self.description.to_string(),
            location: loc,
            portable: self.portable,
            container_state: self.container_state,
            contents: HashSet::<Uuid>::default(),
            restricted: self.restricted,
            abilities,
            interaction_requires: self.interaction_requires.clone(),
            text: self.text.clone(),
            consumable,
        };
        Ok(real_item)
    }
}

/// Wrapper required by TOML limitations to allow deserialization of a bare `RawItem` vector.
#[derive(Deserialize)]
pub struct RawItemFile {
    items: Vec<RawItem>,
}

/// Determine whether an item meets requirements for a particular interaction
pub fn interaction_requirement_met(interaction: ItemInteractionType, target: &Item, tool: &Item) -> bool {
    if let Some(requirement) = target.interaction_requires.get(&interaction) {
        tool.abilities.contains(requirement)
    } else {
        true
    }
}

/// Load `RawItem` vector from file
/// # Errors
/// - if unable to read or parse the items.toml file
pub fn load_raw_items(toml_path: &Path) -> Result<Vec<RawItem>> {
    let item_file =
        fs::read_to_string(toml_path).with_context(|| format!("reading item data from '{}'", toml_path.display()))?;
    let wrapper: RawItemFile = toml::from_str(&item_file)?;
    info!(
        "{} raw items successfully loaded from '{}'",
        wrapper.items.len(),
        toml_path.display(),
    );
    Ok(wrapper.items)
}

/// Build `Items` from raw items.
/// # Errors
/// - if an item pre-registered during room loading is not found in items loaded from file
/// - if there is a failed lookup in the symbol table during raw item conversion
pub fn build_items(raw_items: &[RawItem], symbols: &mut SymbolTable) -> Result<Vec<Item>> {
    // save any pre-registered items and clear them from the symbol table
    let early_inserts = symbols.items.clone();
    symbols.items.clear();
    info!(
        "found {} items that were pre-registered while loading rooms",
        early_inserts.len()
    );

    // rebuild item symbol table from items.toml data
    for ri in raw_items {
        symbols
            .items
            .insert(ri.id.clone(), uuid_from_token(&NAMESPACE_ITEM, ri.id.as_str()));
    }

    // make sure pre-inserted items are in data loaded from items.toml
    for (item_symbol, item_uuid) in &early_inserts {
        if symbols.items.get(item_symbol).is_none_or(|id| id != item_uuid) {
            bail!("error while loading pre-registered item '{item_symbol}': symbol not found or uuid mismatch")
        }
    }
    info!("verified existence of all {} pre-registered items", early_inserts.len());

    // build items from raw_items
    let items: Vec<Item> = raw_items
        .iter()
        .map(|ri| ri.to_item(symbols))
        .collect::<Result<_, _>>()?;
    info!("{} items successfully built from raw items", items.len());
    Ok(items)
}

/// Place items in their starting locations, if any.
/// # Errors
/// - on failed lookups of items, rooms, or NPCs in the symbol table
pub fn place_items(world: &mut AmbleWorld) -> Result<()> {
    // build lists of placements for items
    info!("building item location lists for placement stage");
    let mut room_placements: Vec<(Uuid, Uuid)> = Vec::new();
    let mut chest_placements: Vec<(Uuid, Uuid)> = Vec::new();
    let mut npc_placements: Vec<(Uuid, Uuid)> = Vec::new();
    let mut inventory: Vec<Uuid> = Vec::new();
    let mut unspawned = 0;
    for item in world.items.values() {
        match item.location {
            Location::Room(room_id) => room_placements.push((room_id, item.id)),
            Location::Item(chest_id) => chest_placements.push((chest_id, item.id)),
            Location::Npc(npc_id) => npc_placements.push((npc_id, item.id)),
            Location::Inventory => inventory.push(item.id),
            Location::Nowhere => unspawned += 1,
        }
    }
    // NOTE: MUST be done before items are placed in rooms or inventory to allow nested objects to populate correctly
    info!("placing {} items into containers", chest_placements.len());
    for (chest_id, item_id) in chest_placements {
        let chest = world
            .items
            .get_mut(&chest_id)
            .with_context(|| format!("Container item UUID {chest_id} not found in world.items"))?;
        chest.add_item(item_id);
    }

    info!("placing {} items into rooms", room_placements.len());
    for (room_id, item_id) in room_placements {
        let room = world
            .rooms
            .get_mut(&room_id)
            .with_context(|| format!("Room UUID {room_id} not found in world.rooms"))?;
        room.contents.insert(item_id);
    }

    info!("placing {} items into NPC inventories", npc_placements.len());
    for (npc_id, item_id) in npc_placements {
        let npc = world
            .npcs
            .get_mut(&npc_id)
            .with_context(|| format!("NPC UUID {npc_id} not found in world.npcs"))?;
        npc.add_item(item_id);
    }

    info!("placing {} items into player inventory", inventory.len());
    for item_id in inventory {
        world.player.add_item(item_id);
    }

    info!("{unspawned} items remain unspawned (Location::Nowhere)");
    Ok(())
}
