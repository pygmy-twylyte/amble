//! Command module
//!
//! Describes possible commands used during gameplay.
use variantly::Variantly;

use crate::{dev_command::parse_dev_command, item::ItemInteractionType, style::GameStyle};

/// Commands that can be executed by the player.
#[derive(Debug, Clone, PartialEq, Variantly)]
pub enum Command {
    AdvanceSeq(String), // DEV_MODE only
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
    ResetSeq(String), // DEV_MODE only
    Save(String),
    SetFlag(String),   // DEV_MODE only
    SpawnItem(String), // DEV_MODE only
    StartSeq {
        // DEV_MODE only
        seq_name: String,
        end: String,
    },
    Take(String),
    TakeFrom {
        item: String,
        container: String,
    },
    TalkTo(String),
    Teleport(String), // DEV_MODE only
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
///
/// The parser is case-insensitive; the input is converted to lowercase before
/// being tokenized and matched against known commands.
pub fn parse_command(input: &str) -> Command {
    // normalize user input to lowercase so commands are case-insensitive
    let lc_input = input.to_lowercase();

    // check for and parse developer commands if available
    if let Some(command) = parse_dev_command(input) {
        return command;
    }

    let words: Vec<&str> = lc_input.split_whitespace().collect();
    match words.as_slice() {
        ["goals"] | ["what", "now" | "next"] => Command::Goals,
        ["look"] => Command::Look,
        ["give", item, "to", npc] => Command::GiveToNpc {
            item: (*item).to_string(),
            npc: (*npc).to_string(),
        },
        ["look", "at" | "in", thing] => Command::LookAt((*thing).to_string()),
        ["go" | "climb", "to" | "up" | "down" | "through", dir] | ["move" | "go" | "enter" | "climb", dir] => {
            Command::MoveTo((*dir).to_string())
        },
        ["take", thing] => Command::Take((*thing).to_string()),
        ["take" | "remove" | "get" | "grab", thing, "from", container] => Command::TakeFrom {
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
        ["quit"] => Command::Quit,
        ["drop" | "leave", thing] | ["put", thing, "down"] => Command::Drop((*thing).to_string()),
        ["talk" | "speak", "to" | "with", npc_name] => Command::TalkTo((*npc_name).to_string()),
        ["turn" | "switch", thing, "on"] | ["start" | "trigger", thing] => Command::TurnOn((*thing).to_string()),
        ["help" | "?"] => Command::Help,
        ["read", thing] => Command::Read((*thing).to_string()),
        ["load", gamefile] => Command::Load((*gamefile).to_string()),
        ["save", gamefile] => Command::Save((*gamefile).to_string()),
        [verb, target, "with" | "using", tool] => parse_interaction_type(verb).map_or_else(
            || {
                println!("I don't understand {} in this context.", (*verb).error_style());
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

/// Takes a verb from user input and returns a matching `ItemInteractionType` if any.
///
/// The provided verb should be lowercase; [`parse_command`] handles lowering
/// player input before delegating here.
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
        "repair" | "fix" => Some(ItemInteractionType::Repair),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;

    #[test]
    fn parse_goals_command() {
        let test_inputs = &["goals", "what now", "what next"];
        for input in test_inputs {
            assert_eq!(parse_command(input), Command::Goals);
        }
    }

    #[test]
    fn parse_give_to_npc_command() {
        let test_input = "give item_name to npc_name";
        assert_eq!(
            parse_command(test_input),
            Command::GiveToNpc {
                item: "item_name".into(),
                npc: "npc_name".into(),
            }
        )
    }

    #[test]
    fn parse_look_at_command() {
        let test_inputs = &["look at foo", "look in foo"];
        for input in test_inputs {
            assert_eq!(parse_command(input), Command::LookAt("foo".into()));
        }
    }

    #[test]
    fn parse_move_to_command() {
        let test_inputs = &[
            "go x",
            "move x",
            "enter x",
            "climb x",
            "go to x",
            "go up x",
            "go through x",
            "climb up x",
            "climb down x",
            "climb through x",
        ];
        for input in test_inputs {
            assert_eq!(parse_command(input), Command::MoveTo("x".into()))
        }
    }

    #[test]
    fn parse_take_command() {
        let input = "take x";
        assert_eq!(parse_command(input), Command::Take("x".into()))
    }

    #[test]
    fn parse_take_from_command() {
        let test_inputs = &[
            "take foo from bar",
            "remove foo from bar",
            "get foo from bar",
            "grab foo from bar",
        ];
        for input in test_inputs {
            assert_eq!(
                parse_command(input),
                Command::TakeFrom {
                    item: "foo".into(),
                    container: "bar".into()
                }
            )
        }
    }

    #[test]
    fn parse_put_in_command() {
        let test_inputs = &["put item in chest", "place item in chest"];
        test_inputs.iter().for_each(|input| {
            assert_eq!(
                parse_command(input),
                Command::PutIn {
                    item: "item".into(),
                    container: "chest".into()
                }
            )
        })
    }

    #[test]
    fn parse_open_command() {
        assert_eq!(parse_command("open box"), Command::Open("box".into()))
    }

    #[test]
    fn parse_close_command() {
        let inputs = &["close box", "shut box"];
        inputs
            .iter()
            .for_each(|input| assert_eq!(parse_command(input), Command::Close("box".into())))
    }

    #[test]
    fn parse_lock_command() {
        assert_eq!(parse_command("unlock box"), Command::UnlockItem("box".into()))
    }

    #[test]
    fn parse_unlock_command() {
        assert_eq!(parse_command("lock box"), Command::LockItem("box".into()))
    }

    #[test]
    fn parse_inventory_command() {
        let inputs = &["inventory", "inv"];
        inputs
            .iter()
            .for_each(|input| assert_eq!(parse_command(input), Command::Inventory));
    }

    #[test]
    fn parse_quit_command() {
        assert_eq!(parse_command("quit"), Command::Quit);
    }

    #[test]
    fn parse_drop_command() {
        let inputs = &["drop x", "leave x", "put x down"];
        inputs
            .iter()
            .for_each(|input| assert_eq!(parse_command(input), Command::Drop("x".into())));
    }

    #[test]
    fn parse_talk_to_command() {
        let inputs = &["talk to npc", "talk with npc", "speak to npc", "speak with npc"];
        inputs
            .iter()
            .for_each(|input| assert_eq!(parse_command(input), Command::TalkTo("npc".into())));
    }

    #[test]
    fn parse_turn_on_command() {
        let inputs = &["turn x on", "switch x on", "start x", "trigger x"];
        inputs
            .iter()
            .for_each(|input| assert_eq!(parse_command(input), Command::TurnOn("x".into())));
    }

    #[test]
    fn parse_help_command() {
        assert_eq!(parse_command("help"), Command::Help);
    }

    #[test]
    fn parse_save_command() {
        assert_eq!(parse_command("save save_name"), Command::Save("save_name".into()));
    }

    #[test]
    fn parse_load_command() {
        assert_eq!(parse_command("load save_name"), Command::Load("save_name".into()));
    }

    #[test]
    fn parse_read_command() {
        assert_eq!(parse_command("read item"), Command::Read("item".into()));
    }

    #[test]
    fn parse_use_item_on_command() {
        let answer_key = &[
            (
                "break target with tool",
                Command::UseItemOn {
                    verb: ItemInteractionType::Break,
                    tool: "tool".into(),
                    target: "target".into(),
                },
            ),
            (
                "burn paper using match",
                Command::UseItemOn {
                    verb: ItemInteractionType::Burn,
                    tool: "match".into(),
                    target: "paper".into(),
                },
            ),
            (
                "cover vent with towel",
                Command::UseItemOn {
                    verb: ItemInteractionType::Cover,
                    tool: "towel".into(),
                    target: "vent".into(),
                },
            ),
            (
                "wipe lens with cloth",
                Command::UseItemOn {
                    verb: ItemInteractionType::Clean,
                    tool: "cloth".into(),
                    target: "lens".into(),
                },
            ),
            (
                "cut rope with knife",
                Command::UseItemOn {
                    verb: ItemInteractionType::Cut,
                    tool: "knife".into(),
                    target: "rope".into(),
                },
            ),
            (
                "grasp eel with tongs",
                Command::UseItemOn {
                    verb: ItemInteractionType::Handle,
                    tool: "tongs".into(),
                    target: "eel".into(),
                },
            ),
            (
                "move item with cart",
                Command::UseItemOn {
                    verb: ItemInteractionType::Move,
                    tool: "cart".into(),
                    target: "item".into(),
                },
            ),
            (
                "turn valve with wrench",
                Command::UseItemOn {
                    verb: ItemInteractionType::Turn,
                    tool: "wrench".into(),
                    target: "valve".into(),
                },
            ),
            (
                "open chest with magic_wand",
                Command::UseItemOn {
                    verb: ItemInteractionType::Unlock,
                    tool: "magic_wand".into(),
                    target: "chest".into(),
                },
            ),
            (
                "sharpen blade with grinder",
                Command::UseItemOn {
                    verb: ItemInteractionType::Sharpen,
                    tool: "grinder".into(),
                    target: "blade".into(),
                },
            ),
        ];

        for (input, answer) in answer_key {
            assert_eq!(parse_command(input), *answer);
        }
    }
}
