use gametools::spinners::{Spinner, Wedge};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpinnerType {
    Movement,
    TakeVerb,
    NpcIgnore,
    EntityNotFound,
    QuitMsg,
    UnrecognizedCommand,
    NoEffect,
}

pub trait SpinnerExt {
    fn from_strs(words: &[&'static str]) -> Self;
}
impl SpinnerExt for Spinner<&'static str> {
    fn from_strs(words: &[&'static str]) -> Self {
        let wedges = words.iter().map(|w| Wedge::new(*w)).collect::<Vec<_>>();
        Spinner::new(wedges)
    }
}

pub fn default_spinners() -> HashMap<SpinnerType, Spinner<&'static str>> {
    let mut spinners = HashMap::new();

    let quit_msg_spinner = Spinner::from_strs(&[
        "Bye.",
        "Auf Wiedersehen.",
        "Later, gator",
        "Aww. Come back soon.",
    ]);
    spinners.insert(SpinnerType::QuitMsg, quit_msg_spinner);

    let movement_spinner = Spinner::from_strs(&[
        "Staying alert, you go on...",
        "You move that direction...",
        "Coolly, you roll ahead...",
        "Heading that direction...",
        "You saunter along...",
        "Merrily, you traipse ahead...",
        "You ramble on...",
        "Cruising right along...",
        "Your trek continues...",
        "Strolling right along...",
        "Roam, if you want to...",
        "You amble ahead...",
        "You practice your silly walk in that direction...",
        "Your path decided, you tromp forth...",
        "Your hike continues...",
        "\"Why not jog?\", you think, as you plod along...",
        "You shuffle onward...",
        "You trot that way, like you have to poop...",
        "Lightly, you skip forth...",
        "Onward, you lope like a tauntaun...",
        "You sally forth...",
        "With no chance of an Uber here, you decide to hoof it...",
    ]);
    spinners.insert(SpinnerType::Movement, movement_spinner);

    let take_verb_spinner =
        Spinner::from_strs(&["take", "grab", "get", "snag", "nab", "bag", "pocket"]);
    spinners.insert(SpinnerType::TakeVerb, take_verb_spinner);

    let npc_ignore_spinner = Spinner::from_strs(&[
        "Has nothing to say.",
        "Ignores you.",
        "Hums a tune, hoping you'll go away.",
        "Stands mute.",
        "Disregards you completely. Are you sure you said something out loud?",
        "Isn't in the mood to talk.",
    ]);
    spinners.insert(SpinnerType::NpcIgnore, npc_ignore_spinner);

    let entity_not_found = Spinner::from_strs(&[
        "What's that?",
        "You made that up.",
        "Never heard of it.",
        "You don't see that here.",
        "404 Not Found - Try Again",
        "I don't recognize that.",
    ]);
    spinners.insert(SpinnerType::EntityNotFound, entity_not_found);

    spinners.insert(
        SpinnerType::UnrecognizedCommand,
        Spinner::from_strs(&[
            "Syntax error. Please insert quarter and try again.",
            "That command has been fed to a hungry goat and can no longer be retrieved.",
            "The universe tilts its head, confused.",
            "I'm sorry, Dave. I'm afraid I can't do that.",
            "The parser emits a low, judgmental beep.",
            "That seems like something you'd do in a *different* text adventure.",
            "Your words echo into the void, unacknowledged.",
            "Command not recognized. Try 'help', 'yell', or 'cry'.",
            "That input caused a minor existential crisis in the REPL. It's fine now.",
            "Unrecognized verb. Maybe try it in ALL CAPS for emphasis?",
        ]),
    );

    spinners.insert(
        SpinnerType::NoEffect,
        Spinner::from_strs(&[
            "You try it. It doesn't seem to help.",
            "The world shrugs.",
            "You stare at the result. The result stares back.",
            "Well... it *could* have worked.",
            "Absolutely nothing happens. Not even a dramatic pause.",
            "You feel a vague sense of anticlimax.",
            "Somewhere, a cricket hesitates, then chirps once.",
            "The laws of physics remain disappointingly unaltered.",
            "Reality refuses to acknowledge your efforts.",
            "No errors, no effect, no fanfare. Just... stillness.",
        ]),
    );

    info!("{} default spinners created", spinners.len());
    spinners
}
