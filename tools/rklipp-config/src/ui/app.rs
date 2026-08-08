use crate::schema::RklippConfig;
use crossterm::event::KeyEvent;
use ratatui::prelude::*;

pub enum MenuItem {
    Machine,
    Mcu,
    Axes,
    Pins,
    Save,
}

pub struct App {
    pub config: RklippConfig,
    pub active_menu_item: MenuItem,
    // Add state for widgets, e.g., list selection, text input
}

impl App {
    pub fn new() -> Self {
        Self {
            config: RklippConfig::default(),
            active_menu_item: MenuItem::Machine,
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // Handle key events to navigate menus and edit config
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(area);

        self.render_menu(f, chunks[0]);
        self.render_form(f, chunks[1]);
    }

    fn render_menu(&mut self, f: &mut Frame, area: Rect) {
        // Render the main menu
    }

    fn render_form(&mut self, f: &mut Frame, area: Rect) {
        // Render the form for the active menu item
    }
}
