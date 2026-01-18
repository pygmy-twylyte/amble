//! Player command parsing and representation.
//!
//! Defines the command vocabulary understood by the REPL, including
//! parsing helpers that convert raw input into strongly typed commands.

use pest::{Parser, iterators::Pair};
use pest_derive::Parser;
use variantly::Variantly;

use crate::{
    dev_command::parse_dev_command,
    item::{IngestMode, ItemInteractionType},
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
    Ingest {
        item: String,
        mode: IngestMode,
    },
    Inventory,
    ListSaves,
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
    Touch(String),
    TurnOff(String),
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
    DevNote(String),
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

/// PEG parser for player input, generated from [`repl_grammar.pest`].
#[derive(Parser)]
#[grammar = "repl_grammar.pest"]
pub struct CommandParser;

/// Parses an input string and returns a corresponding `Command`.
///
/// The parser is case-insensitive; the input is converted to lowercase before
/// being tokenized and matched against grammar rules. If no valid command can
/// be determined, returns `Command::Unknown`
pub fn parse_command(input: &str, view: &mut View) -> Command {
    let lc_input = input.to_lowercase();
    // isolated handling for `dev-mode` commands
    if let Some(command) = parse_dev_command(input, view) {
        return command;
    }
    let Some(command_pair) = parse_pair_from_input(lc_input.as_str()) else {
        return Command::Unknown;
    };
    build_command_from_pair(command_pair)
}

fn build_command_from_pair(command_pair: Pair<'_, Rule>) -> Command {
    match command_pair.as_rule() {
        Rule::EOI => Command::Unknown,
        Rule::inventory => Command::Inventory,
        Rule::list_saves => Command::ListSaves,
        Rule::help => Command::Help,
        Rule::goals => Command::Goals,
        Rule::look => Command::Look,
        Rule::quit => Command::Quit,
        Rule::go_back => Command::GoBack,
        Rule::vm_clear => Command::SetViewMode(ViewMode::ClearVerbose),
        Rule::vm_verbose => Command::SetViewMode(ViewMode::Verbose),
        Rule::vm_brief => Command::SetViewMode(ViewMode::Brief),
        Rule::look_at => Command::LookAt(inner_string(command_pair)),
        Rule::load => Command::Load(inner_string(command_pair)),
        Rule::save => Command::Save(inner_string(command_pair)),
        Rule::take => Command::Take(inner_string(command_pair)),
        Rule::drop => Command::Drop(inner_string(command_pair)),
        Rule::talk_to => Command::TalkTo(inner_string(command_pair)),
        Rule::turn_on => Command::TurnOn(inner_string(command_pair)),
        Rule::turn_off => Command::TurnOff(inner_string(command_pair)),
        Rule::theme => Command::Theme(inner_string(command_pair)),
        Rule::open => Command::Open(inner_string(command_pair)),
        Rule::close => Command::Close(inner_string(command_pair)),
        Rule::lock => Command::LockItem(inner_string(command_pair)),
        Rule::unlock => Command::UnlockItem(inner_string(command_pair)),
        Rule::move_to => Command::MoveTo(inner_string(command_pair)),
        Rule::read => Command::Read(inner_string(command_pair)),
        Rule::touch => Command::Touch(inner_string(command_pair)),
        Rule::eat => Command::Ingest {
            item: inner_string(command_pair),
            mode: IngestMode::Eat,
        },
        Rule::drink => Command::Ingest {
            item: inner_string(command_pair),
            mode: IngestMode::Drink,
        },
        Rule::inhale => Command::Ingest {
            item: inner_string(command_pair),
            mode: IngestMode::Inhale,
        },
        Rule::give_to_npc => {
            let (item, npc) = inner_string_duo(command_pair);
            Command::GiveToNpc { item, npc }
        },
        Rule::take_from => {
            let (item, container) = inner_string_duo(command_pair);
            Command::TakeFrom { item, container }
        },
        Rule::put_in => {
            let (item, container) = inner_string_duo(command_pair);
            Command::PutIn { item, container }
        },
        // twt = "target with tool"
        Rule::attach_twt => verb_target_with_tool(ItemInteractionType::Attach, command_pair),
        Rule::detach_twt => verb_target_with_tool(ItemInteractionType::Detach, command_pair),
        Rule::break_twt => verb_target_with_tool(ItemInteractionType::Break, command_pair),
        Rule::burn_twt => verb_target_with_tool(ItemInteractionType::Burn, command_pair),
        Rule::extinguish_twt => verb_target_with_tool(ItemInteractionType::Extinguish, command_pair),
        Rule::clean_twt => verb_target_with_tool(ItemInteractionType::Clean, command_pair),
        Rule::cover_twt => verb_target_with_tool(ItemInteractionType::Cover, command_pair),
        Rule::cut_twt => verb_target_with_tool(ItemInteractionType::Cut, command_pair),
        Rule::handle_twt => verb_target_with_tool(ItemInteractionType::Handle, command_pair),
        Rule::move_twt => verb_target_with_tool(ItemInteractionType::Move, command_pair),
        Rule::open_twt => verb_target_with_tool(ItemInteractionType::Open, command_pair),
        Rule::repair_twt => verb_target_with_tool(ItemInteractionType::Repair, command_pair),
        Rule::sharpen_twt => verb_target_with_tool(ItemInteractionType::Sharpen, command_pair),
        Rule::turn_twt => verb_target_with_tool(ItemInteractionType::Turn, command_pair),
        Rule::unlock_twt => verb_target_with_tool(ItemInteractionType::Unlock, command_pair),
        _ => unreachable!(),
    }
}

/// Takes user input and returns a parsed command (as a Pest `Pair`) that matches any
/// defined `Rule`, otherwise `None`.
fn parse_pair_from_input(input_text: &str) -> Option<Pair<'_, Rule>> {
    let Ok(mut pairs) = CommandParser::parse(Rule::repl_input, input_text) else {
        return None;
    };
    let Some(repl_input_pair) = pairs.next() else {
        return None;
    };
    let mut inner = repl_input_pair.into_inner();
    let Some(command) = inner.next() else {
        return None;
    };
    Some(command)
}

/// Extract a single string argument from a parsed rule.
fn inner_string(pair: Pair<Rule>) -> String {
    if let Some(inner) = pair.into_inner().next() {
        inner.as_str().to_string()
    } else {
        String::new()
    }
}

/// Extract two string arguments from a parsed rule.
fn inner_string_duo(pair: Pair<Rule>) -> (String, String) {
    let mut inner = pair.into_inner();
    if let Some(first) = inner.next()
        && let Some(second) = inner.next()
    {
        (first.as_str().to_string(), second.as_str().to_string())
    } else {
        (String::new(), String::new())
    }
}

/// Build a `UseItemOn` command for interaction rules that follow
/// "verb target with tool" (e.g. "light fuse with candle") grammar.
fn verb_target_with_tool(verb: ItemInteractionType, pair: Pair<Rule>) -> Command {
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
    fn parse_unknown_command() {
        assert_eq!(pc("foobar"), Command::Unknown);
        assert_eq!(pc(":notadev"), Command::Unknown);
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
        );
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
            assert_eq!(pc(input), Command::MoveTo("x".into()));
        }
    }

    #[test]
    fn parse_take_command() {
        let input = "take x";
        assert_eq!(pc(input), Command::Take("x".into()));
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
            );
        }
    }

    #[test]
    fn parse_put_in_command() {
        let test_inputs = &["put item in chest", "place item in chest"];
        for input in test_inputs {
            assert_eq!(
                pc(input),
                Command::PutIn {
                    item: "item".into(),
                    container: "chest".into()
                }
            );
        }
    }

    #[test]
    fn parse_open_command() {
        assert_eq!(pc("open box"), Command::Open("box".into()));
    }

    #[test]
    fn parse_close_command() {
        let inputs = &["close box", "shut box"];
        for input in inputs {
            assert_eq!(pc(input), Command::Close("box".into()));
        }
    }

    #[test]
    fn parse_lock_command() {
        assert_eq!(pc("unlock box"), Command::UnlockItem("box".into()));
    }

    #[test]
    fn parse_unlock_command() {
        assert_eq!(pc("lock box"), Command::LockItem("box".into()));
    }

    #[test]
    fn parse_inventory_command() {
        let inputs = &["inventory", "inv"];
        for input in inputs {
            assert_eq!(pc(input), Command::Inventory);
        }
    }

    #[test]
    fn parse_list_saves_command() {
        let inputs = &["saves", "list saves"];
        for input in inputs {
            assert_eq!(pc(input), Command::ListSaves);
        }
    }

    #[test]
    fn parse_quit_command() {
        assert_eq!(pc("quit"), Command::Quit);
    }

    #[test]
    fn parse_drop_command() {
        let inputs = &["drop x", "leave x", "put x down"];
        for input in inputs {
            assert_eq!(pc(input), Command::Drop("x".into()));
        }
    }

    #[test]
    fn parse_talk_to_command() {
        let inputs = &["talk to npc", "talk with npc", "speak to npc", "speak with npc"];
        for input in inputs {
            assert_eq!(pc(input), Command::TalkTo("npc".into()));
        }
    }

    #[test]
    fn parse_turn_on_command() {
        let inputs = &["turn x on", "switch x on", "start x", "trigger x"];
        for input in inputs {
            assert_eq!(pc(input), Command::TurnOn("x".into()));
        }
    }

    #[test]
    fn parse_touch_monolith_command() {
        assert_eq!(pc("touch monolith"), Command::Touch("monolith".into()));
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
    #[allow(clippy::too_many_lines)]
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
