//! Command module
//!
//! Describes possible commands used during gameplay.
use variantly;

use crate::{item::ItemInteractionType, style::GameStyle};

/// Commands that can be executed by the player.
#[derive(Debug, variantly::Variantly)]
pub enum Command {
    Close(String),
    Drop(String),
    GiveToNpc {
        item: String,
        npc: String,
    },
    Goals,
    Help,
    Inventory,
    Load(String),
    LockItem(String),
    Look,
    LookAt(String),
    MoveTo(String),
    Open(String),
    PutIn {
        item: String,
        container: String,
    },
    Quit,
    Read(String),
    Save(String),
    Take(String),
    TakeFrom {
        item: String,
        container: String,
    },
    TalkTo(String),
    Teleport(String),
    TurnOn(String),
    Unknown,
    UnlockItem(String),
    UseItemOn {
        verb: ItemInteractionType,
        tool: String,
        target: String,
    },
}

/// Parses an input string and returns a corresponding `Command` if recognized.
pub fn parse_command(input: &str) -> Command {
    let words: Vec<&str> = input.split_whitespace().collect();
    match words.as_slice() {
        ["!port", room_toml_id] => Command::Teleport((*room_toml_id).to_string()),
        ["goals"] | ["what", "now" | "next"] => Command::Goals,
        ["look"] => Command::Look,
        ["give", item, "to", npc] => Command::GiveToNpc {
            item: (*item).to_string(),
            npc: (*npc).to_string(),
        },
        ["look", "at" | "in", thing] => Command::LookAt((*thing).to_string()),
        ["go" | "climb", "to" | "up" | "down" | "through", dir]
        | ["move" | "go" | "enter" | "climb", dir] => Command::MoveTo((*dir).to_string()),
        ["take", thing] => Command::Take((*thing).to_string()),
        ["take" | "remove", thing, "from", container] => Command::TakeFrom {
            item: (*thing).to_string(),
            container: (*container).to_string(),
        },
        ["put" | "place", thing, "in", container] => Command::PutIn {
            item: (*thing).to_string(),
            container: (*container).to_string(),
        },
        ["open", thing] => Command::Open((*thing).to_string()),
        ["close" | "shut", thing] => Command::Close((*thing).to_string()),
        ["lock", thing] => Command::LockItem((*thing).to_string()),
        ["unlock", thing] => Command::UnlockItem((*thing).to_string()),
        ["inventory" | "inv"] => Command::Inventory,
        ["quit" | "exit"] => Command::Quit,
        ["drop", thing] => Command::Drop((*thing).to_string()),
        ["talk" | "speak", "to" | "with", npc_name] => Command::TalkTo((*npc_name).to_string()),
        ["turn" | "switch", thing, "on"] | ["start", thing] => {
            Command::TurnOn((*thing).to_string())
        },
        ["help" | "?"] => Command::Help,
        ["read", thing] => Command::Read((*thing).to_string()),
        ["load", gamefile] => Command::Load((*gamefile).to_string()),
        ["save", gamefile] => Command::Save((*gamefile).to_string()),
        [verb, target, "with" | "using", tool] => parse_interaction_type(verb).map_or_else(
            || {
                println!(
                    "I don't understand {} in this context.",
                    (*verb).error_style()
                );
                Command::Unknown
            },
            |interaction| Command::UseItemOn {
                verb: interaction,
                tool: (*tool).to_string(),
                target: (*target).to_string(),
            },
        ), // ex. burn wood with torch
        _ => Command::Unknown,
    }
}

/// Takes a verb from user input and returns a matching `ItemInteractionType` if any determined
pub fn parse_interaction_type(verb: &str) -> Option<ItemInteractionType> {
    match verb {
        "break" | "smash" | "crack" | "shatter" => Some(ItemInteractionType::Break),
        "burn" | "ignite" | "light" | "melt" => Some(ItemInteractionType::Burn),
        "cover" | "wrap" | "shroud" | "mask" => Some(ItemInteractionType::Cover),
        "cut" | "slice" | "sever" | "slash" | "carve" | "chop" => Some(ItemInteractionType::Cut),
        "handle" | "take" | "grasp" | "hold" | "grab" => Some(ItemInteractionType::Handle),
        "move" | "remove" | "shift" | "shove" | "budge" => Some(ItemInteractionType::Move),
        "turn" | "spin" | "twist" | "swivel" => Some(ItemInteractionType::Turn),
        "unlock" | "undo" | "open" => Some(ItemInteractionType::Unlock),
        "sharpen" | "hone" => Some(ItemInteractionType::Sharpen),
        "clean" | "wipe" | "shine" | "buff" => Some(ItemInteractionType::Clean),
        _ => None,
    }
}
