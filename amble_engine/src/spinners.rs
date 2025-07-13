use gametools::spinners::{Spinner, Wedge, wedges_from_tuples};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpinnerType {
    AmbientAA3B,
    AmbientInterior,
    AmbientWoodland,
    EntityNotFound,
    Movement,
    Nauseated,
    NoEffect,
    NpcIgnore,
    QuitMsg,
    TakeVerb,
    UnrecognizedCommand,
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
        "Later, 'gator!",
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
            "Snagrilfromp. I can make words up too.",
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
            "That's another entry for the Journal of Failed Experiments.",
        ]),
    );

    #[rustfmt::skip]
    spinners.insert(
        SpinnerType::AmbientWoodland,
        Spinner::new(wedges_from_tuples(vec![
            ("", 16),
            ("A bird sings warily: \"Poo-tee-weet?\"", 1),
            ("A swallow flies past, carrying a coconut.", 1),
            ("A hummingbird flies by, but gliding like an albatross on an unseen wind.", 1),
            ("Something tromps through the leaves behind you, but stops when you turn around.", 1),
        ])),
    );

    spinners.insert(
        SpinnerType::AmbientAA3B,
        Spinner::new(wedges_from_tuples(vec![
            ("", 15),
            ("A shimmering blue police box flickers into existence for an instant, then vanishes.", 1),
            ("A robed figure mutters in an unknown tongue before dissolving into mist.", 1),
            ("For a moment, your surroundings flatten as if the world became two-dimensional.", 1),
            ("A penguin waddles past, wearing a lab coat and muttering about a narrative malfunction.", 1),
            ("A black obelisk taller than you shimmers into view, then fades as quickly as it appeared.", 1),
        ])),
    );

    spinners.insert(
        SpinnerType::AmbientInterior,
        Spinner::new(wedges_from_tuples(vec![
            ("", 20),
            ("The intercom crackles: \"Please return the time machine to the checkout desk.\"", 1),
            ("A siren whoops twice, then cuts off mid-blare.", 1),
            ("A calm female voice over the PA says: \"Welcome to Aperture Science. Enjoy your stay,\" followed by a beep.", 1),
            ("Somewhere behind the walls, gears grind and then stop abruptly.", 1),
            ("The lights flicker and buzz with a discordant hum.", 1),
            ("A gust of wind rushes through the corridor, but there are no open doors or windows.", 1),
        ])),
    );

    #[rustfmt::skip]
    spinners.insert(
        SpinnerType::Nauseated,
        Spinner::new(wedges_from_tuples(vec![
            ("", 10),
            ("Since reading the Vogon poetry, you can't shake a background queasiness.", 1 ),
            ("You throw up a little in your mouth.", 1),
            ("You regurgitate a bit of last night's dinner, but can't remember what it was.", 1,),
        ])),
    );

    info!("{} default spinners created", spinners.len());
    spinners
}
