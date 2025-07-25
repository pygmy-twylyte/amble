use std::{io, time::Duration};

use amble_engine::{item::Item, load_world, npc::Npc, room::Room, trigger::Trigger};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use uuid::Uuid;

enum EditKind {
    Room,
    Item,
    Npc,
    Trigger,
}

enum ViewMode {
    Rooms,
    Items,
    Npcs,

    Triggers,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load world data
    let world = load_world()?;

    let mut mode = ViewMode::Rooms;
    let mut selected_room = 0usize;
    let mut selected_item = 0usize;
    let mut selected_npc = 0usize;
    let mut selected_trigger = 0usize;
    let mut editing: Option<EditKind> = None;
    let mut input_buffer = String::new();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            if let Some(kind) = &editing {
                let size = f.size();
                let area = Rect {
                    x: 0,
                    y: size.height.saturating_sub(3),
                    width: size.width,
                    height: 3,
                };
                let title = match kind {
                    EditKind::Room => "Edit Room Name",
                    EditKind::Item => "Edit Item Name",
                    EditKind::Npc => "Edit NPC Name",
                    EditKind::Trigger => "Edit Trigger Name",
                };
                let paragraph =
                    Paragraph::new(input_buffer.as_str()).block(Block::default().title(title).borders(Borders::ALL));
                f.render_widget(paragraph, area);
                f.set_cursor(area.x + input_buffer.len() as u16 + 1, area.y + 1);
                return;
            }

            let sidebar_width = if matches!(mode, ViewMode::Triggers) { 40 } else { 30 };
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(sidebar_width), Constraint::Min(10)].as_ref())
                .split(f.size());

            match mode {
                ViewMode::Rooms => {
                    let mut rooms: Vec<(uuid::Uuid, &Room)> = world.rooms.iter().map(|(id, r)| (*id, r)).collect();
                    rooms.sort_by_key(|(_, r)| r.symbol.clone());
                    let list_items: Vec<ListItem> = rooms
                        .iter()
                        .map(|(_, room)| ListItem::new(room.symbol.as_str()))
                        .collect();
                    let list = List::new(list_items)
                        .block(Block::default().title("Rooms").borders(Borders::ALL))
                        .highlight_style(Style::default().fg(Color::Yellow));
                    let mut state = ratatui::widgets::ListState::default();
                    state.select(Some(selected_room));
                    f.render_stateful_widget(list, chunks[0], &mut state);

                    let room = rooms[selected_room];
                    let mut detail = vec![
                        Line::from(vec![
                            Span::styled("Name: ", Style::default().fg(Color::Blue)),
                            Span::raw(room.name.clone()),
                        ]),
                        Line::from(vec![
                            Span::styled("Symbol: ", Style::default().fg(Color::Blue)),
                            Span::raw(room.symbol.clone()),
                        ]),
                        Line::from(vec![
                            Span::styled("UUID: ", Style::default().fg(Color::Blue)),
                            Span::raw(room.id.to_string()),
                        ]),
                        Line::from(vec![
                            Span::styled("Visited: ", Style::default().fg(Color::Blue)),
                            Span::raw(room.visited.to_string()),
                        ]),
                        Line::from(vec![
                            Span::styled("Description: ", Style::default().fg(Color::Blue)),
                            Span::raw(room.base_description.clone()),
                        ]),
                        Line::from(vec![
                            Span::styled("Location: ", Style::default().fg(Color::Blue)),
                            Span::raw(format!("{:?}", room.location)),
                        ]),
                    ];
                    if !room.exits.is_empty() {
                        detail.push(Line::from(vec![Span::styled(
                            "Exits:",
                            Style::default().fg(Color::Blue),
                        )]));
                        for (dir, exit) in &room.exits {
                            let target = world
                                .rooms
                                .get(&exit.to)
                                .map_or_else(|| exit.to.to_string(), |r| r.symbol.clone());
                            let mut line = format!("  {dir} -> {target}");
                            if exit.locked {
                                line.push_str(" [locked]");
                            }
                            detail.push(Line::from(Span::raw(line)));
                        }
                    }
                    if !room.overlays.is_empty() {
                        detail.push(Line::from(vec![Span::styled(
                            "Overlays:",
                            Style::default().fg(Color::Blue),
                        )]));
                        for ov in &room.overlays {
                            detail.push(Line::from(Span::raw(format!("  if {:?} => {}", ov.condition, ov.text))));
                        }
                    }
                    let paragraph = Paragraph::new(detail)
                        .wrap(Wrap { trim: true })
                        .block(Block::default().title("Detail").borders(Borders::ALL));
                    f.render_widget(paragraph, chunks[1]);
                },
                ViewMode::Items => {
                    let mut items: Vec<(uuid::Uuid, &Item)> = world.items.iter().map(|(id, i)| (*id, i)).collect();
                    items.sort_by_key(|(_, i)| i.symbol.clone());
                    let list_items: Vec<ListItem> = items
                        .iter()
                        .map(|(_, item)| ListItem::new(item.symbol.as_str()))
                        .collect();
                    let list = List::new(list_items)
                        .block(Block::default().title("Items").borders(Borders::ALL))
                        .highlight_style(Style::default().fg(Color::Yellow));
                    let mut state = ratatui::widgets::ListState::default();
                    state.select(Some(selected_item));
                    f.render_stateful_widget(list, chunks[0], &mut state);

                    let (_, item) = items[selected_item];
                    let mut detail = vec![
                        Line::from(vec![
                            Span::styled("Name: ", Style::default().fg(Color::Blue)),
                            Span::raw(item.name.clone()),
                        ]),
                        Line::from(vec![
                            Span::styled("Symbol: ", Style::default().fg(Color::Blue)),
                            Span::raw(item.symbol.clone()),
                        ]),
                        Line::from(vec![
                            Span::styled("UUID: ", Style::default().fg(Color::Blue)),
                            Span::raw(item.id.to_string()),
                        ]),
                        Line::from(vec![
                            Span::styled("Description: ", Style::default().fg(Color::Blue)),
                            Span::raw(item.description.clone()),
                        ]),
                        Line::from(vec![
                            Span::styled("Location: ", Style::default().fg(Color::Blue)),
                            Span::raw(format!("{:?}", item.location)),
                        ]),
                    ];
                    if let Some(state) = item.container_state {
                        detail.push(Line::from(vec![
                            Span::styled("Container: ", Style::default().fg(Color::Blue)),
                            Span::raw(format!("{:?}", state)),
                        ]));
                        detail.push(Line::from(vec![Span::styled(
                            "Contents:",
                            Style::default().fg(Color::Blue),
                        )]));
                        if item.contents.is_empty() {
                            detail.push(Line::from(Span::raw("  (empty)")));
                        } else {
                            for cid in &item.contents {
                                if let Some(cont) = world.items.get(cid) {
                                    detail.push(Line::from(Span::raw(format!("  {}", cont.name))));
                                }
                            }
                        }
                    }
                    let paragraph = Paragraph::new(detail)
                        .wrap(Wrap { trim: true })
                        .block(Block::default().title("Detail").borders(Borders::ALL));
                    f.render_widget(paragraph, chunks[1]);
                },
                ViewMode::Npcs => {
                    let mut npcs: Vec<(uuid::Uuid, &Npc)> = world.npcs.iter().map(|(id, n)| (*id, n)).collect();
                    npcs.sort_by_key(|(_, n)| n.symbol.clone());
                    let list_items: Vec<ListItem> =
                        npcs.iter().map(|(_, npc)| ListItem::new(npc.symbol.as_str())).collect();
                    let list = List::new(list_items)
                        .block(Block::default().title("NPCs").borders(Borders::ALL))
                        .highlight_style(Style::default().fg(Color::Yellow));
                    let mut state = ratatui::widgets::ListState::default();
                    state.select(Some(selected_npc));
                    f.render_stateful_widget(list, chunks[0], &mut state);

                    let (_, npc) = npcs[selected_npc];
                    let mut detail = vec![
                        Line::from(vec![
                            Span::styled("Name: ", Style::default().fg(Color::Blue)),
                            Span::raw(npc.name.clone()),
                        ]),
                        Line::from(vec![
                            Span::styled("Symbol: ", Style::default().fg(Color::Blue)),
                            Span::raw(npc.symbol.clone()),
                        ]),
                        Line::from(vec![
                            Span::styled("UUID: ", Style::default().fg(Color::Blue)),
                            Span::raw(npc.id.to_string()),
                        ]),
                        Line::from(vec![
                            Span::styled("Mood: ", Style::default().fg(Color::Blue)),
                            Span::raw(format!("{:?}", npc.mood)),
                        ]),
                        Line::from(vec![
                            Span::styled("Description: ", Style::default().fg(Color::Blue)),
                            Span::raw(npc.description.clone()),
                        ]),
                        Line::from(vec![
                            Span::styled("Location: ", Style::default().fg(Color::Blue)),
                            Span::raw(format!("{:?}", npc.location)),
                        ]),
                    ];
                    if !npc.dialogue.is_empty() {
                        detail.push(Line::from(vec![Span::styled(
                            "Dialogue:",
                            Style::default().fg(Color::Blue),
                        )]));
                        for (mood, lines) in &npc.dialogue {
                            detail.push(Line::from(Span::raw(format!("  {:?}:", mood))));
                            for line in lines {
                                detail.push(Line::from(Span::raw(format!("    {}", line))));
                            }
                        }
                    }
                    let paragraph = Paragraph::new(detail)
                        .wrap(Wrap { trim: true })
                        .block(Block::default().title("Detail").borders(Borders::ALL));
                    f.render_widget(paragraph, chunks[1]);
                },
                ViewMode::Triggers => {
                    let triggers: Vec<&Trigger> = world.triggers.iter().collect();
                    let list_items: Vec<ListItem> = triggers
                        .iter()
                        .enumerate()
                        .map(|(i, t)| {
                            let bg = if i % 2 == 0 { Color::Black } else { Color::DarkGray };
                            ListItem::new(t.name.as_str()).style(Style::default().bg(bg))
                        })
                        .collect();
                    let list = List::new(list_items)
                        .block(Block::default().title("Triggers").borders(Borders::ALL))
                        .highlight_style(Style::default().fg(Color::Yellow));
                    let mut state = ratatui::widgets::ListState::default();
                    state.select(Some(selected_trigger));
                    f.render_stateful_widget(list, chunks[0], &mut state);

                    let trg = triggers[selected_trigger];
                    let mut detail = vec![
                        Line::from(vec![
                            Span::styled("Name: ", Style::default().fg(Color::Blue)),
                            Span::raw(trg.name.clone()),
                        ]),
                        Line::from(vec![
                            Span::styled("Only Once: ", Style::default().fg(Color::Blue)),
                            Span::raw(trg.only_once.to_string()),
                        ]),
                        Line::from(vec![
                            Span::styled("Fired: ", Style::default().fg(Color::Blue)),
                            Span::raw(trg.fired.to_string()),
                        ]),
                        Line::from(vec![Span::styled("Conditions:", Style::default().fg(Color::Blue))]),
                    ];
                    for cond in &trg.conditions {
                        detail.push(Line::from(Span::raw(format!("  {:?}", cond))));
                    }
                    detail.push(Line::from(vec![Span::styled(
                        "Actions:",
                        Style::default().fg(Color::Blue),
                    )]));
                    for act in &trg.actions {
                        detail.push(Line::from(Span::raw(format!("  {:?}", act))));
                    }
                    let paragraph = Paragraph::new(detail)
                        .wrap(Wrap { trim: true })
                        .block(Block::default().title("Detail").borders(Borders::ALL));

                    f.render_widget(paragraph, chunks[1]);
                },
            }
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if let Some(kind) = &editing {
                    match key.code {
                        KeyCode::Esc => {
                            editing = None;
                            input_buffer.clear();
                        },
                        KeyCode::Enter => {
                            match kind {
                                EditKind::Room => {
                                    let mut ids: Vec<(Uuid, String)> =
                                        world.rooms.iter().map(|(id, r)| (*id, r.symbol.clone())).collect();
                                    ids.sort_by_key(|(_, sym)| sym.clone());
                                    if let Some((room_id, _)) = ids.get(selected_room) {
                                        if let Some(room) = world.rooms.get_mut(room_id) {
                                            room.name = input_buffer.clone();
                                        }
                                    }
                                },
                                EditKind::Item => {
                                    let mut ids: Vec<(Uuid, String)> =
                                        world.items.iter().map(|(id, i)| (*id, i.symbol.clone())).collect();
                                    ids.sort_by_key(|(_, sym)| sym.clone());
                                    if let Some((item_id, _)) = ids.get(selected_item) {
                                        if let Some(item) = world.items.get_mut(item_id) {
                                            item.name = input_buffer.clone();
                                        }
                                    }
                                },
                                EditKind::Npc => {
                                    let mut ids: Vec<(Uuid, String)> =
                                        world.npcs.iter().map(|(id, n)| (*id, n.symbol.clone())).collect();
                                    ids.sort_by_key(|(_, sym)| sym.clone());
                                    if let Some((npc_id, _)) = ids.get(selected_npc) {
                                        if let Some(npc) = world.npcs.get_mut(npc_id) {
                                            npc.name = input_buffer.clone();
                                        }
                                    }
                                },
                                EditKind::Trigger => {
                                    if let Some(trg) = world.triggers.get_mut(selected_trigger) {
                                        trg.name = input_buffer.clone();
                                    }
                                },
                            }
                            editing = None;
                            input_buffer.clear();
                        },
                        KeyCode::Char(c) => {
                            input_buffer.push(c);
                        },
                        KeyCode::Backspace => {
                            input_buffer.pop();
                        },
                        _ => {},
                    }
                    continue;
                }
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
                    KeyCode::Char('t') => {
                        mode = ViewMode::Triggers;
                    },

                    KeyCode::Char('e') => {
                        if editing.is_none() {
                            editing = Some(match mode {
                                ViewMode::Rooms => EditKind::Room,
                                ViewMode::Items => EditKind::Item,
                                ViewMode::Npcs => EditKind::Npc,
                                ViewMode::Triggers => EditKind::Trigger,
                            });
                            input_buffer = match mode {
                                ViewMode::Rooms => {
                                    let mut items: Vec<(Uuid, String)> =
                                        world.rooms.iter().map(|(id, r)| (*id, r.symbol.clone())).collect();
                                    items.sort_by_key(|(_, sym)| sym.clone());
                                    if let Some((id, _)) = items.get(selected_room) {
                                        world.rooms.get(id).map(|r| r.name.clone()).unwrap_or_default()
                                    } else {
                                        String::new()
                                    }
                                },
                                ViewMode::Items => {
                                    let mut items: Vec<(Uuid, String)> =
                                        world.items.iter().map(|(id, i)| (*id, i.symbol.clone())).collect();
                                    items.sort_by_key(|(_, sym)| sym.clone());
                                    if let Some((id, _)) = items.get(selected_item) {
                                        world.items.get(id).map(|i| i.name.clone()).unwrap_or_default()
                                    } else {
                                        String::new()
                                    }
                                },
                                ViewMode::Npcs => {
                                    let mut items: Vec<(Uuid, String)> =
                                        world.npcs.iter().map(|(id, n)| (*id, n.symbol.clone())).collect();
                                    items.sort_by_key(|(_, sym)| sym.clone());
                                    if let Some((id, _)) = items.get(selected_npc) {
                                        world.npcs.get(id).map(|n| n.name.clone()).unwrap_or_default()
                                    } else {
                                        String::new()
                                    }
                                },
                                ViewMode::Triggers => world
                                    .triggers
                                    .get(selected_trigger)
                                    .map(|t| t.name.clone())
                                    .unwrap_or_default(),
                            };
                        }
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

                        ViewMode::Triggers => {
                            let len = world.triggers.len();
                            if len > 0 {
                                selected_trigger = (selected_trigger + 1).min(len - 1);
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

                        ViewMode::Triggers => {
                            selected_trigger = selected_trigger.saturating_sub(1);
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
