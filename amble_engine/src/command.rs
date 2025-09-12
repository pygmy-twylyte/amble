//! Command module
//!
//! Describes possible commands used during gameplay.

use pest::{Parser, iterators::Pair};
use pest_derive::Parser;
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

#[derive(Parser)]
#[grammar = "repl_grammar.pest"]
pub struct CommandParser;

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

    let command = CommandParser::parse(Rule::command, &lc_input).unwrap().next().unwrap();
    match command.as_rule() {
        Rule::EOI => Command::Unknown,
        Rule::inventory => Command::Inventory,
        Rule::help => Command::Help,
        Rule::goals => Command::Goals,
        Rule::look => Command::Look,
        Rule::quit => Command::Quit,
        Rule::go_back => Command::GoBack,
        Rule::vm_clear => Command::SetViewMode(ViewMode::ClearVerbose),
        Rule::vm_verbose => Command::SetViewMode(ViewMode::Verbose),
        Rule::vm_brief => Command::SetViewMode(ViewMode::Brief),
        Rule::look_at => Command::LookAt(inner_string(command)),
        Rule::load => Command::Load(inner_string(command)),
        Rule::save => Command::Save(inner_string(command)),
        Rule::take => Command::Take(inner_string(command)),
        Rule::drop => Command::Drop(inner_string(command)),
        Rule::talk_to => Command::TalkTo(inner_string(command)),
        Rule::turn_on => Command::TurnOn(inner_string(command)),
        Rule::theme => Command::Theme(inner_string(command)),
        Rule::open => Command::Open(inner_string(command)),
        Rule::close => Command::Close(inner_string(command)),
        Rule::lock => Command::LockItem(inner_string(command)),
        Rule::unlock => Command::UnlockItem(inner_string(command)),
        Rule::move_to => Command::MoveTo(inner_string(command)),
        Rule::read => Command::Read(inner_string(command)),
        Rule::give_to_npc => {
            let (item, npc) = inner_string_duo(command);
            Command::GiveToNpc { item, npc }
        },
        Rule::take_from => {
            let (item, container) = inner_string_duo(command);
            Command::TakeFrom { item, container }
        },
        Rule::put_in => {
            let (item, container) = inner_string_duo(command);
            Command::PutIn { item, container }
        },
        // twt = "target with tool"
        Rule::attach_twt => twt_command(ItemInteractionType::Attach, command),
        Rule::break_twt => twt_command(ItemInteractionType::Break, command),
        Rule::burn_twt => twt_command(ItemInteractionType::Burn, command),
        Rule::extinguish_twt => twt_command(ItemInteractionType::Extinguish, command),
        Rule::clean_twt => twt_command(ItemInteractionType::Clean, command),
        Rule::cover_twt => twt_command(ItemInteractionType::Cover, command),
        Rule::cut_twt => twt_command(ItemInteractionType::Cut, command),
        Rule::handle_twt => twt_command(ItemInteractionType::Handle, command),
        Rule::move_twt => twt_command(ItemInteractionType::Move, command),
        Rule::open_twt => twt_command(ItemInteractionType::Open, command),
        Rule::repair_twt => twt_command(ItemInteractionType::Repair, command),
        Rule::sharpen_twt => twt_command(ItemInteractionType::Sharpen, command),
        Rule::turn_twt => twt_command(ItemInteractionType::Turn, command),
        Rule::unlock_twt => twt_command(ItemInteractionType::Unlock, command),
        _ => unreachable!(),
    }
}

/// Helper to extract user input from a Pair<Rule> (when a single string).
pub fn inner_string(pair: Pair<Rule>) -> String {
    if let Some(inner) = pair.into_inner().next() {
        inner.as_str().to_string()
    } else {
        "".to_string()
    }
}

/// Helper to extract a pair of strings from inner rules.
pub fn inner_string_duo(pair: Pair<Rule>) -> (String, String) {
    let mut inner = pair.into_inner();
    if let Some(first) = inner.next()
        && let Some(second) = inner.next()
    {
        (first.as_str().to_string(), second.as_str().to_string())
    } else {
        ("".to_string(), "".to_string())
    }
}

/// Create a verb-target-with-tool form (`UseItemOn`) command from ItemInteractionType and a Pair.
pub fn twt_command(verb: ItemInteractionType, pair: Pair<Rule>) -> Command {
    let (target, tool) = inner_string_duo(pair);
    Command::UseItemOn { verb, tool, target }
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
            "walk x",
            "climb x",
            "move to x",
            "run to x",
            "climb through x",
            "climb into x",
            "climb on x",
            "walk through the x",
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
        let test_inputs = &["back", "go back"];
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
