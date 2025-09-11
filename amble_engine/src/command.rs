//! Command module
//!
//! Describes possible commands used during gameplay.

use variantly::Variantly;

use crate::{
    dev_command::parse_dev_command,
    item::ItemInteractionType,
    view::{View, ViewMode},
};

/// Commands that can be executed by the player.
#[derive(Debug, Clone, PartialEq, Variantly)]
pub enum Command {
    Close(String),
    Drop(String),
    GiveToNpc {
        item: String,
        npc: String,
    },
    Goals,
    GoBack,
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
    SetViewMode(ViewMode),
    Take(String),
    TakeFrom {
        item: String,
        container: String,
    },
    TalkTo(String),
    Theme(String),
    TurnOn(String),
    Unknown,
    UnlockItem(String),
    UseItemOn {
        verb: ItemInteractionType,
        tool: String,
        target: String,
    },
    // Commands below can only be used when crate::DEV_MODE is set when built.
    HelpDev,
    ListNpcs,
    ListFlags,
    ListSched,
    AdvanceSeq(String),
    ResetSeq(String),
    SetFlag(String),
    SpawnItem(String),
    StartSeq {
        // DEV_MODE only
        seq_name: String,
        end: String,
    },
    Teleport(String),
    // Scheduler management (DEV_MODE only)
    SchedCancel(usize),
    SchedDelay {
        idx: usize,
        turns: usize,
    },
}

/// Parses an input string and returns a corresponding `Command` if recognized.
///
/// The parser is case-insensitive; the input is converted to lowercase before
/// being tokenized and matched against known commands.
pub fn parse_command(input: &str, view: &mut View) -> Command {
    // normalize user input to lowercase so commands are case-insensitive
    let lc_input = input.to_lowercase();

    // check for and parse developer commands if available
    if let Some(command) = parse_dev_command(input, view) {
        return command;
    }

    let words: Vec<&str> = lc_input.split_whitespace().collect();
    match words.as_slice() {
        ["goals"] | ["what", "now" | "next"] => Command::Goals,
        ["back" | "return"] | ["go", "back"] => Command::GoBack,
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
        ["touch", "monolith"] => Command::TurnOn("monolith".to_string()),
        ["turn" | "switch", thing, "on"] | ["start" | "trigger", thing] => Command::TurnOn((*thing).to_string()),
        ["help" | "?"] => Command::Help,
        ["read", thing] => Command::Read((*thing).to_string()),
        ["load", gamefile] => Command::Load((*gamefile).to_string()),
        ["save", gamefile] => Command::Save((*gamefile).to_string()),
        [verb, target, "with" | "using" | "to", tool] => {
            parse_interaction_type(verb).map_or(Command::Unknown, |interaction| Command::UseItemOn {
                verb: interaction,
                tool: (*tool).to_string(),
                target: (*target).to_string(),
            })
        }, // ex. burn wood with torch
        ["brief"] => Command::SetViewMode(ViewMode::Brief),
        ["clear"] => Command::SetViewMode(ViewMode::ClearVerbose),
        ["verbose"] => Command::SetViewMode(ViewMode::Verbose),
        ["theme"] => Command::Theme("list".to_string()),
        ["theme", theme_name] => Command::Theme((*theme_name).to_string()),
        _ => Command::Unknown,
    }
}

/// Takes a verb from user input and returns a matching `ItemInteractionType` if any.
///
/// The provided verb should be lowercase; `parse_command` handles lowering
/// player input before delegating here.
///
/// # Examples
/// ```
/// # use amble_engine::command::parse_interaction_type;
/// # use amble_engine::item::ItemInteractionType;
/// assert_eq!(parse_interaction_type("burn"), Some(ItemInteractionType::Burn));
/// assert_eq!(parse_interaction_type("invalid"), None);
/// ```
pub fn parse_interaction_type(verb: &str) -> Option<ItemInteractionType> {
    INTERACTION_VERBS.get(verb).copied()
}

lazy_static::lazy_static! {
    static ref INTERACTION_VERBS: std::collections::HashMap<&'static str, ItemInteractionType> = {
        use ItemInteractionType::*;
        let mut verbs = std::collections::HashMap::new();

        for verb in ["open", "pry"] {
            verbs.insert(verb, Open);
        }
        for verb in ["attach", "connect", "join"] {
            verbs.insert(verb, Attach);
        }
        for verb in ["break", "smash", "crack", "shatter"] {
            verbs.insert(verb, Break);
        }
        for verb in ["burn", "ignite", "light", "melt"] {
            verbs.insert(verb, Burn);
        }
        for verb in ["extinguish", "spray"] {
            verbs.insert(verb, Extinguish);
        }
        for verb in ["cover", "wrap", "shroud", "mask"] {
            verbs.insert(verb, Cover);
        }
        for verb in ["cut", "slice", "sever", "slash", "carve", "chop"] {
            verbs.insert(verb, Cut);
        }
        for verb in ["handle", "take", "grasp", "hold", "grab"] {
            verbs.insert(verb, Handle);
        }
        for verb in ["move", "remove", "shift", "shove", "budge"] {
            verbs.insert(verb, Move);
        }
        for verb in ["turn", "spin", "twist", "swivel"] {
            verbs.insert(verb, Turn);
        }
        for verb in ["unlock", "undo"] {
            verbs.insert(verb, Unlock);
        }
        for verb in ["sharpen", "hone"] {
            verbs.insert(verb, Sharpen);
        }
        for verb in ["clean", "wipe", "shine", "buff"] {
            verbs.insert(verb, Clean);
        }
        for verb in ["repair", "fix"] {
            verbs.insert(verb, Repair);
        }

        verbs

    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::View;
    use crate::command::Command;

    fn pc(input: &str) -> Command {
        let mut view = View::new();
        parse_command(input, &mut view)
    }

    #[test]
    fn parse_goals_command() {
        let test_inputs = &["goals", "what now", "what next"];
        for input in test_inputs {
            assert_eq!(pc(input), Command::Goals);
        }
    }

    #[test]
    fn parse_theme_command() {
        assert_eq!(pc("theme seaside"), Command::Theme("seaside".into()));
        assert_eq!(pc("theme default"), Command::Theme("default".into()));
    }

    #[test]
    fn parse_give_to_npc_command() {
        let test_input = "give item_name to npc_name";
        assert_eq!(
            pc(test_input),
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
            assert_eq!(pc(input), Command::LookAt("foo".into()));
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
            assert_eq!(pc(input), Command::MoveTo("x".into()))
        }
    }

    #[test]
    fn parse_take_command() {
        let input = "take x";
        assert_eq!(pc(input), Command::Take("x".into()))
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
                pc(input),
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
                pc(input),
                Command::PutIn {
                    item: "item".into(),
                    container: "chest".into()
                }
            )
        })
    }

    #[test]
    fn parse_open_command() {
        assert_eq!(pc("open box"), Command::Open("box".into()))
    }

    #[test]
    fn parse_close_command() {
        let inputs = &["close box", "shut box"];
        inputs
            .iter()
            .for_each(|input| assert_eq!(pc(input), Command::Close("box".into())))
    }

    #[test]
    fn parse_lock_command() {
        assert_eq!(pc("unlock box"), Command::UnlockItem("box".into()))
    }

    #[test]
    fn parse_unlock_command() {
        assert_eq!(pc("lock box"), Command::LockItem("box".into()))
    }

    #[test]
    fn parse_inventory_command() {
        let inputs = &["inventory", "inv"];
        inputs
            .iter()
            .for_each(|input| assert_eq!(pc(input), Command::Inventory));
    }

    #[test]
    fn parse_quit_command() {
        assert_eq!(pc("quit"), Command::Quit);
    }

    #[test]
    fn parse_drop_command() {
        let inputs = &["drop x", "leave x", "put x down"];
        inputs
            .iter()
            .for_each(|input| assert_eq!(pc(input), Command::Drop("x".into())));
    }

    #[test]
    fn parse_talk_to_command() {
        let inputs = &["talk to npc", "talk with npc", "speak to npc", "speak with npc"];
        inputs
            .iter()
            .for_each(|input| assert_eq!(pc(input), Command::TalkTo("npc".into())));
    }

    #[test]
    fn parse_turn_on_command() {
        let inputs = &["turn x on", "switch x on", "start x", "trigger x"];
        inputs
            .iter()
            .for_each(|input| assert_eq!(pc(input), Command::TurnOn("x".into())));
    }

    #[test]
    fn parse_touch_monolith_command() {
        assert_eq!(pc("touch monolith"), Command::TurnOn("monolith".into()));
    }

    #[test]
    fn parse_help_command() {
        assert_eq!(pc("help"), Command::Help);
    }

    #[test]
    fn parse_save_command() {
        assert_eq!(pc("save save_name"), Command::Save("save_name".into()));
    }

    #[test]
    fn parse_load_command() {
        assert_eq!(pc("load save_name"), Command::Load("save_name".into()));
    }

    #[test]
    fn parse_read_command() {
        assert_eq!(pc("read item"), Command::Read("item".into()));
    }

    #[test]
    fn parse_go_back_command() {
        let test_inputs = &["back", "go back", "return"];
        for input in test_inputs {
            assert_eq!(pc(input), Command::GoBack);
        }
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
                "extinguish fire with foam",
                Command::UseItemOn {
                    verb: ItemInteractionType::Extinguish,
                    tool: "foam".into(),
                    target: "fire".into(),
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
                    verb: ItemInteractionType::Open,
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
            (
                "spray blaze with extinguisher",
                Command::UseItemOn {
                    verb: ItemInteractionType::Extinguish,
                    tool: "extinguisher".into(),
                    target: "blaze".into(),
                },
            ),
        ];

        for (input, answer) in answer_key {
            assert_eq!(pc(input), *answer);
        }
    }
}
