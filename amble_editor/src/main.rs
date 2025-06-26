use std::{collections::HashMap, io, time::Duration};

use amble_engine::{load_world, Item};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use uuid::Uuid;

fn load_items_from_engine() -> anyhow::Result<HashMap<Uuid, Item>> {
    let world = load_world()?;

    Ok(world.items.clone())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load items from engine
    let item_map = load_items_from_engine()?;
    let items: Vec<_> = item_map.values().collect();
    let mut selected = 0;

    // Setup terminal UI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main event loop
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(30), Constraint::Min(10)].as_ref())
                .split(f.size());

            // Left sidebar: list of item names
            let list_items: Vec<ListItem> = items
                .iter()
                .map(|item| ListItem::new(item.name.as_str()))
                .collect();

            let list = List::new(list_items)
                .block(Block::default().title("Items").borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow));

            let mut state = ratatui::widgets::ListState::default();
            state.select(Some(selected));
            f.render_stateful_widget(list, chunks[0], &mut state);

            // Right panel: item detail
            let detail = format!(
                "Name: {}\nUUID: {}\nDescription: {}\nLocation: {:?}",
                items[selected].name,
                items[selected].id,
                items[selected].description,
                items[selected].location
            );

            let paragraph = Paragraph::new(detail)
                .block(Block::default().title("Detail").borders(Borders::ALL));
            f.render_widget(paragraph, chunks[1]);
        })?;

        // Handle key input
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => {
                        selected = (selected + 1).min(items.len().saturating_sub(1));
                    }
                    KeyCode::Up => {
                        selected = selected.saturating_sub(1);
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
