use std::{io, time::Duration};

use amble_engine::{item::Item, load_world, npc::Npc, room::Room};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

enum ViewMode {
    Rooms,
    Items,
    Npcs,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load world data
    let world = load_world()?;

    let mut mode = ViewMode::Rooms;
    let mut selected_room = 0usize;
    let mut selected_item = 0usize;
    let mut selected_npc = 0usize;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(30), Constraint::Min(10)].as_ref())
                .split(f.size());

            match mode {
                ViewMode::Rooms => {
                    let rooms: Vec<&Room> = world.rooms.values().collect();
                    let list_items: Vec<ListItem> =
                        rooms.iter().map(|room| ListItem::new(room.symbol.as_str())).collect();
                    let list = List::new(list_items)
                        .block(Block::default().title("Rooms").borders(Borders::ALL))
                        .highlight_style(Style::default().fg(Color::Yellow));
                    let mut state = ratatui::widgets::ListState::default();
                    state.select(Some(selected_room));
                    f.render_stateful_widget(list, chunks[0], &mut state);

                    let room = rooms[selected_room];
                    let detail = format!(
                        "Name: {}\nSymbol: {}\nUUID: {}\nVisited: {}\nDescription: {}\nLocation: {:?}",
                        room.name, room.symbol, room.id, room.visited, room.base_description, room.location
                    );
                    let paragraph =
                        Paragraph::new(detail).block(Block::default().title("Detail").borders(Borders::ALL));
                    f.render_widget(paragraph, chunks[1]);
                },
                ViewMode::Items => {
                    let items: Vec<&Item> = world.items.values().collect();
                    let list_items: Vec<ListItem> =
                        items.iter().map(|item| ListItem::new(item.symbol.as_str())).collect();
                    let list = List::new(list_items)
                        .block(Block::default().title("Items").borders(Borders::ALL))
                        .highlight_style(Style::default().fg(Color::Yellow));
                    let mut state = ratatui::widgets::ListState::default();
                    state.select(Some(selected_item));
                    f.render_stateful_widget(list, chunks[0], &mut state);

                    let item = items[selected_item];
                    let detail = format!(
                        "Name: {}\nSymbol: {}\nUUID: {}\nDescription: {}\nLocation: {:?}",
                        item.name, item.symbol, item.id, item.description, item.location
                    );
                    let paragraph =
                        Paragraph::new(detail).block(Block::default().title("Detail").borders(Borders::ALL));
                    f.render_widget(paragraph, chunks[1]);
                },
                ViewMode::Npcs => {
                    let npcs: Vec<&Npc> = world.npcs.values().collect();
                    let list_items: Vec<ListItem> = npcs.iter().map(|npc| ListItem::new(npc.symbol.as_str())).collect();
                    let list = List::new(list_items)
                        .block(Block::default().title("NPCs").borders(Borders::ALL))
                        .highlight_style(Style::default().fg(Color::Yellow));
                    let mut state = ratatui::widgets::ListState::default();
                    state.select(Some(selected_npc));
                    f.render_stateful_widget(list, chunks[0], &mut state);

                    let npc = npcs[selected_npc];
                    let detail = format!(
                        "Name: {}\nSymbol: {}\nUUID: {}\nMood: {:?}\nDescription: {}\nLocation: {:?}",
                        npc.name, npc.symbol, npc.id, npc.mood, npc.description, npc.location
                    );
                    let paragraph =
                        Paragraph::new(detail).block(Block::default().title("Detail").borders(Borders::ALL));
                    f.render_widget(paragraph, chunks[1]);
                },
            }
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('r') => {
                        mode = ViewMode::Rooms;
                    },
                    KeyCode::Char('i') => {
                        mode = ViewMode::Items;
                    },
                    KeyCode::Char('n') => {
                        mode = ViewMode::Npcs;
                    },
                    KeyCode::Down => match mode {
                        ViewMode::Rooms => {
                            let len = world.rooms.len();
                            if len > 0 {
                                selected_room = (selected_room + 1).min(len - 1);
                            }
                        },
                        ViewMode::Items => {
                            let len = world.items.len();
                            if len > 0 {
                                selected_item = (selected_item + 1).min(len - 1);
                            }
                        },
                        ViewMode::Npcs => {
                            let len = world.npcs.len();
                            if len > 0 {
                                selected_npc = (selected_npc + 1).min(len - 1);
                            }
                        },
                    },
                    KeyCode::Up => match mode {
                        ViewMode::Rooms => {
                            selected_room = selected_room.saturating_sub(1);
                        },
                        ViewMode::Items => {
                            selected_item = selected_item.saturating_sub(1);
                        },
                        ViewMode::Npcs => {
                            selected_npc = selected_npc.saturating_sub(1);
                        },
                    },
                    _ => {},
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
