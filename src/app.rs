use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListItem};
use ratatui::{DefaultTerminal, Frame};
use std::process::Command;

// use crate::profiles::{Profile, ProfileList};
use ratatui::widgets::ListState;

#[derive(PartialEq, Eq)]
pub enum Profile {
    Performance,
    Balanced,
    PowerSaver,
}

pub struct ProfileList {
    items: Vec<ProfileListItem>,
    state: ListState,
}

impl Default for ProfileList {
    fn default() -> Self {
        Self {
            items: vec![
                ProfileListItem {
                    profile: Profile::Performance,
                    display: String::from("Performance"),
                },
                ProfileListItem {
                    profile: Profile::Balanced,
                    display: String::from("Balanced"),
                },
                ProfileListItem {
                    profile: Profile::PowerSaver,
                    display: String::from("Power Saver"),
                },
            ],
            state: ListState::default().with_selected(Some(0)),
        }
    }
}

pub struct ProfileListItem {
    profile: Profile,
    display: String,
}

/// The main application which holds the state and logic of the application.
pub struct App {
    /// Is the application running?
    running: bool,
    /// The active profile
    active_profile: Profile,
    /// The list of available profiles
    profile_list: ProfileList,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        let mut app = App {
            running: true,
            active_profile: Profile::Balanced,
            profile_list: ProfileList::default(),
        };
        app.update_active_profile();
        app
    }

    /// Update the currently active profile to the correct one
    fn update_active_profile(&mut self) {
        /// Use powerprofilesctl to check the active profile
        fn check_active_profile() -> Profile {
            // Query powerprofilesctl
            let profile_query_output = Command::new("powerprofilesctl")
                .arg("get")
                .output()
                .expect("Failed running powerprofilesctl");

            // Get the response as a string
            let profile_res = String::from_utf8_lossy(&profile_query_output.stdout).into_owned();
            // Return the apppropriate Profile
            match profile_res.as_str() {
                "performance\n" => Profile::Performance,
                "balanced\n" => Profile::Balanced,
                "power-saver\n" => Profile::PowerSaver,
                _ => panic!("Unexpected output from powerprofilesctl"),
            }
        }

        // Set the active profile
        self.active_profile = check_active_profile();
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Render the UI
    fn render(&mut self, frame: &mut Frame) {
        // Set the title
        let title = Line::from("Select Power Profile").bold().green().centered();

        let selected_style = Style::new().bg(Color::White).fg(Color::Black);

        // Map the list items
        let mut items: Vec<ListItem> = self
            .profile_list
            .items
            .iter()
            .map(|item| {
                ListItem::from(format!(
                    "{} {}",
                    if item.profile == self.active_profile {
                        ">"
                    } else {
                        " "
                    },
                    item.display.clone()
                ))
            })
            .collect();

        // Set the selected item to the proper style
        if let Some(i) = self.profile_list.state.selected() {
            items[i] = items[i].clone().style(selected_style);
        }

        frame.render_widget(
            List::new(items).block(Block::bordered().title(title)),
            frame.area(),
        )
    }

    /// Reads the crossterm events and updates the state of [`App`].
    fn handle_crossterm_events(&mut self) -> color_eyre::Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char('j') | KeyCode::Down) => self.select_next_profile(),
            (_, KeyCode::Char('k') | KeyCode::Up) => self.select_prev_profile(),
            (_, KeyCode::Enter) => self.set_profile(),
            _ => {}
        }
    }

    /// Selects the next profile in the list
    fn select_next_profile(&mut self) {
        if let Some(i) = self.profile_list.state.selected() {
            if i < self.profile_list.items.len() - 1 {
                self.profile_list.state.select_next();
            } else {
                self.profile_list.state.select_first();
            }
        }
    }

    /// Selects the previous profile in the list
    fn select_prev_profile(&mut self) {
        if let Some(i) = self.profile_list.state.selected() {
            if i > 0 {
                self.profile_list.state.select_previous();
            } else {
                self.profile_list
                    .state
                    .select(Some(self.profile_list.items.len() - 1));
            }
        }
    }

    /// Sets a new profile based on the one selected in the list, then updates the active profile
    fn set_profile(&mut self) {
        if let Some(i) = self.profile_list.state.selected() {
            // Get the apppropriate argument
            let profile_arg = match self.profile_list.items[i].profile {
                Profile::Performance => String::from("performance"),
                Profile::Balanced => String::from("balanced"),
                Profile::PowerSaver => String::from("power-saver"),
            };

            // Run the command
            Command::new("powerprofilesctl")
                .arg("set")
                .arg(profile_arg)
                .status()
                .expect("Failed running powerprofilesctl");

            // Update the active profile
            self.update_active_profile();
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
