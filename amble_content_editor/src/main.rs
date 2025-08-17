use adw::prelude::*;
use anyhow::Result;
use gtk4::{gio, glib};
use log::info;
use std::cell::RefCell;
use std::rc::Rc;

mod app;
mod data;
mod ui;
mod utils;
mod validation;

use app::AmbleEditorApp;

const APP_ID: &str = "com.amble.ContentEditor";
const APP_NAME: &str = "Amble Content Editor";

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting Amble Content Editor");

    // Initialize GTK and Adwaita
    let app = adw::Application::builder().application_id(APP_ID).build();

    // Connect application signals
    app.connect_startup(|app| {
        // Load CSS for custom styling
        load_css();

        // Setup actions
        setup_actions(app);

        info!("Application startup complete");
    });

    app.connect_activate(move |app| {
        // Create and show the main window
        let window = build_main_window(app);
        window.present();

        info!("Main window presented");
    });

    // Run the application
    let empty: Vec<String> = vec![];
    app.run_with_args(&empty);

    Ok(())
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(include_str!("style.css"));

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn setup_actions(app: &adw::Application) {
    // File actions
    let action_new = gio::SimpleAction::new("new", None);
    action_new.connect_activate(glib::clone!(@weak app => move |_, _| {
        info!("New project action triggered");
        // Implementation will be in app module
    }));
    app.add_action(&action_new);

    let action_open = gio::SimpleAction::new("open", None);
    action_open.connect_activate(glib::clone!(@weak app => move |_, _| {
        info!("Open project action triggered");
        // Implementation will be in app module
    }));
    app.add_action(&action_open);

    let action_save = gio::SimpleAction::new("save", None);
    action_save.connect_activate(glib::clone!(@weak app => move |_, _| {
        info!("Save action triggered");
        // Implementation will be in app module
    }));
    app.add_action(&action_save);

    let action_save_as = gio::SimpleAction::new("save-as", None);
    action_save_as.connect_activate(glib::clone!(@weak app => move |_, _| {
        info!("Save As action triggered");
        // Implementation will be in app module
    }));
    app.add_action(&action_save_as);

    // Edit actions
    let action_undo = gio::SimpleAction::new("undo", None);
    action_undo.connect_activate(glib::clone!(@weak app => move |_, _| {
        info!("Undo action triggered");
        // Implementation will be in app module
    }));
    app.add_action(&action_undo);

    let action_redo = gio::SimpleAction::new("redo", None);
    action_redo.connect_activate(glib::clone!(@weak app => move |_, _| {
        info!("Redo action triggered");
        // Implementation will be in app module
    }));
    app.add_action(&action_redo);

    // Validation actions
    let action_validate = gio::SimpleAction::new("validate", None);
    action_validate.connect_activate(glib::clone!(@weak app => move |_, _| {
        info!("Validate action triggered");
        // Implementation will be in app module
    }));
    app.add_action(&action_validate);

    // Help actions
    let action_about = gio::SimpleAction::new("about", None);
    action_about.connect_activate(glib::clone!(@weak app => move |_, _| {
        show_about_dialog(&app);
    }));
    app.add_action(&action_about);

    let action_quit = gio::SimpleAction::new("quit", None);
    action_quit.connect_activate(glib::clone!(@weak app => move |_, _| {
        app.quit();
    }));
    app.add_action(&action_quit);

    // Set keyboard shortcuts
    app.set_accels_for_action("app.new", &["<Ctrl>n"]);
    app.set_accels_for_action("app.open", &["<Ctrl>o"]);
    app.set_accels_for_action("app.save", &["<Ctrl>s"]);
    app.set_accels_for_action("app.save-as", &["<Ctrl><Shift>s"]);
    app.set_accels_for_action("app.undo", &["<Ctrl>z"]);
    app.set_accels_for_action("app.redo", &["<Ctrl><Shift>z", "<Ctrl>y"]);
    app.set_accels_for_action("app.validate", &["<Ctrl><Shift>v"]);
    app.set_accels_for_action("app.quit", &["<Ctrl>q"]);
}

fn build_main_window(app: &adw::Application) -> adw::ApplicationWindow {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(APP_NAME)
        .default_width(1400)
        .default_height(900)
        .build();

    // Create header bar
    let header_bar = adw::HeaderBar::new();

    // Create menu button
    let menu_button = gtk4::MenuButton::builder().icon_name("open-menu-symbolic").build();

    let menu = gio::Menu::new();
    let file_menu = gio::Menu::new();
    file_menu.append(Some("New Project"), Some("app.new"));
    file_menu.append(Some("Open Project"), Some("app.open"));
    file_menu.append(Some("Save"), Some("app.save"));
    file_menu.append(Some("Save As..."), Some("app.save-as"));
    menu.append_submenu(Some("File"), &file_menu);

    let edit_menu = gio::Menu::new();
    edit_menu.append(Some("Undo"), Some("app.undo"));
    edit_menu.append(Some("Redo"), Some("app.redo"));
    menu.append_submenu(Some("Edit"), &edit_menu);

    let tools_menu = gio::Menu::new();
    tools_menu.append(Some("Validate All"), Some("app.validate"));
    menu.append_submenu(Some("Tools"), &tools_menu);

    let help_menu = gio::Menu::new();
    help_menu.append(Some("About"), Some("app.about"));
    menu.append_submenu(Some("Help"), &help_menu);

    menu_button.set_menu_model(Some(&menu));
    header_bar.pack_end(&menu_button);

    // Create toolbar with quick actions
    let toolbar_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    toolbar_box.set_margin_start(6);

    let new_button = gtk4::Button::builder()
        .icon_name("document-new-symbolic")
        .tooltip_text("New Project (Ctrl+N)")
        .action_name("app.new")
        .build();
    toolbar_box.append(&new_button);

    let open_button = gtk4::Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text("Open Project (Ctrl+O)")
        .action_name("app.open")
        .build();
    toolbar_box.append(&open_button);

    let save_button = gtk4::Button::builder()
        .icon_name("document-save-symbolic")
        .tooltip_text("Save (Ctrl+S)")
        .action_name("app.save")
        .build();
    toolbar_box.append(&save_button);

    toolbar_box.append(&gtk4::Separator::new(gtk4::Orientation::Vertical));

    let undo_button = gtk4::Button::builder()
        .icon_name("edit-undo-symbolic")
        .tooltip_text("Undo (Ctrl+Z)")
        .action_name("app.undo")
        .build();
    toolbar_box.append(&undo_button);

    let redo_button = gtk4::Button::builder()
        .icon_name("edit-redo-symbolic")
        .tooltip_text("Redo (Ctrl+Y)")
        .action_name("app.redo")
        .build();
    toolbar_box.append(&redo_button);

    toolbar_box.append(&gtk4::Separator::new(gtk4::Orientation::Vertical));

    let validate_button = gtk4::Button::builder()
        .icon_name("dialog-information-symbolic")
        .tooltip_text("Validate All (Ctrl+Shift+V)")
        .action_name("app.validate")
        .build();
    toolbar_box.append(&validate_button);

    header_bar.pack_start(&toolbar_box);

    // Create main content area
    let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // Create navigation sidebar and main content area
    let paned = gtk4::Paned::new(gtk4::Orientation::Horizontal);
    paned.set_position(300);
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);

    // Create sidebar with navigation
    let sidebar = create_sidebar();
    paned.set_start_child(Some(&sidebar));

    // Create main editor area with tabs
    let editor_notebook = adw::TabView::new();
    let tab_bar = adw::TabBar::new();
    tab_bar.set_view(Some(&editor_notebook));
    tab_bar.set_autohide(false);

    let editor_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    editor_box.append(&tab_bar);
    editor_box.append(&editor_notebook);

    paned.set_end_child(Some(&editor_box));

    // Add status bar
    let status_bar = create_status_bar();

    // Assemble the window
    content_box.append(&header_bar);
    content_box.append(&paned);
    content_box.append(&status_bar);

    window.set_content(Some(&content_box));

    // Store the editor state
    let editor_state = Rc::new(RefCell::new(EditorState::new(editor_notebook)));
    window.set_data("editor_state", editor_state);

    window
}

fn create_sidebar() -> gtk4::ScrolledWindow {
    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .build();

    let sidebar_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sidebar_box.set_margin_start(6);
    sidebar_box.set_margin_end(6);
    sidebar_box.set_margin_top(6);
    sidebar_box.set_margin_bottom(6);

    // Create expandable sections for different content types
    let sections = vec![
        ("Rooms", "room-symbolic", vec!["rooms.toml"]),
        ("Items", "item-symbolic", vec!["items.toml"]),
        ("NPCs", "npc-symbolic", vec!["npcs.toml"]),
        ("Triggers", "trigger-symbolic", vec!["triggers.toml"]),
        ("Goals", "goal-symbolic", vec!["goals.toml"]),
        ("Player", "player-symbolic", vec!["player.toml"]),
        ("Spinners", "spinner-symbolic", vec!["spinners.toml"]),
        ("Help", "help-symbolic", vec!["help_commands.toml", "help_basic.txt"]),
    ];

    for (title, icon, files) in sections {
        let expander = adw::ExpanderRow::builder().title(title).icon_name(icon).build();

        for file in files {
            let row = adw::ActionRow::builder().title(file).activatable(true).build();

            row.connect_activated(move |_| {
                info!("Opening file: {}", file);
                // File opening will be handled by the app module
            });

            expander.add_row(&row);
        }

        sidebar_box.append(&expander);
    }

    scrolled.set_child(Some(&sidebar_box));
    scrolled
}

fn create_status_bar() -> gtk4::Box {
    let status_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    status_bar.set_margin_start(6);
    status_bar.set_margin_end(6);
    status_bar.set_margin_top(3);
    status_bar.set_margin_bottom(3);

    let status_label = gtk4::Label::new(Some("Ready"));
    status_label.set_halign(gtk4::Align::Start);
    status_label.set_hexpand(true);
    status_bar.append(&status_label);

    let validation_indicator = gtk4::Label::new(Some("✓ Valid"));
    validation_indicator.add_css_class("success");
    status_bar.append(&validation_indicator);

    let position_label = gtk4::Label::new(Some("Ln 1, Col 1"));
    status_bar.append(&position_label);

    status_bar
}

fn show_about_dialog(app: &adw::Application) {
    let about = adw::AboutWindow::builder()
        .application_name(APP_NAME)
        .application_icon(APP_ID)
        .developer_name("Amble Development Team")
        .version(env!("CARGO_PKG_VERSION"))
        .comments("A comprehensive content editor for the Amble game engine")
        .website("https://github.com/amble/amble")
        .license_type(gtk4::License::MitX11)
        .build();

    if let Some(window) = app.active_window() {
        about.set_transient_for(Some(&window));
    }

    about.present();
}

// Editor state management
struct EditorState {
    tab_view: adw::TabView,
    open_files: Vec<String>,
    undo_stack: Vec<EditAction>,
    redo_stack: Vec<EditAction>,
    current_file: Option<String>,
}

impl EditorState {
    fn new(tab_view: adw::TabView) -> Self {
        Self {
            tab_view,
            open_files: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_file: None,
        }
    }
}

#[derive(Clone, Debug)]
enum EditAction {
    Insert {
        file: String,
        position: usize,
        text: String,
    },
    Delete {
        file: String,
        position: usize,
        text: String,
    },
    Replace {
        file: String,
        position: usize,
        old_text: String,
        new_text: String,
    },
}
