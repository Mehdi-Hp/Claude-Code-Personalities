//! Interactive TUI for configuring display preferences with live preview

use ansi_to_tui::IntoText;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Wrap},
};
use std::io;

use crate::config::PersonalityPreferences;
use crate::state::SessionState;
use crate::statusline::{WorkspaceInfo, build_statusline};
use crate::types::Activity;

/// Available separator character options
const SEPARATOR_OPTIONS: &[&str] = &[
    "\u{2022}", // • bullet (default)
    "\u{22c5}", // ⋅ dot operator
    "|",        // | pipe
    "\u{2731}", // ✱ heavy asterisk
    "\u{271c}", // ✜ heavy open centre cross
    "\u{2748}", // ❈ heavy sparkle
    "\u{2743}", // ❃ heavy teardrop-spoked pinwheel
    "\u{2724}", // ✤ heavy four balloon-spoked asterisk
    "\u{274b}", // ❋ heavy eight teardrop-spoked propeller
    "\u{2726}", // ✦ black four pointed star
    "\u{2014}", // — em dash
    "\u{fe31}", // ︱ presentation form vertical line
];

/// Type of configuration option
#[derive(Clone)]
enum OptionType {
    /// Simple on/off toggle
    Toggle,
    /// Selection from a list of choices
    Select {
        choices: &'static [&'static str],
        current_index: usize,
    },
}

/// A hierarchical config option with tree structure information
#[derive(Clone)]
struct ConfigOption {
    /// Display name shown in the menu
    name: &'static str,
    /// Key used for preferences lookup/update
    pref_key: &'static str,
    /// Whether this option is currently enabled
    enabled: bool,
    /// Nesting depth: 0 = top-level, 1 = child
    depth: usize,
    /// Parent option name (for checking if parent is enabled)
    parent: Option<&'static str>,
    /// Whether this is the last child in its group (for └─ vs ├─)
    is_last_child: bool,
    /// Type of option (Toggle or Select)
    option_type: OptionType,
}

/// Application state for the interactive config TUI
struct ConfigApp {
    /// Current preferences being edited
    prefs: PersonalityPreferences,
    /// Hierarchical list of config options
    options: Vec<ConfigOption>,
    /// Current cursor position
    cursor: usize,
    /// Whether the app should quit
    should_quit: bool,
}

impl ConfigApp {
    fn new(prefs: PersonalityPreferences) -> Self {
        let options = Self::build_options(&prefs);
        Self {
            prefs,
            options,
            cursor: 0,
            should_quit: false,
        }
    }

    /// Build the hierarchical options list from preferences
    fn build_options(prefs: &PersonalityPreferences) -> Vec<ConfigOption> {
        // Find current separator index
        let separator_index = SEPARATOR_OPTIONS
            .iter()
            .position(|&s| s == prefs.display.separator_char)
            .unwrap_or(0);

        vec![
            ConfigOption {
                name: "Personality",
                pref_key: "Personality",
                depth: 0,
                parent: None,
                is_last_child: false,
                enabled: prefs.show_personality,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Activity",
                pref_key: "Activity",
                depth: 0,
                parent: None,
                is_last_child: false,
                enabled: prefs.show_activity,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Context",
                pref_key: "Activity Context",
                depth: 1,
                parent: Some("Activity"),
                is_last_child: true,
                enabled: prefs.show_context,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Git",
                pref_key: "Git",
                depth: 0,
                parent: None,
                is_last_child: false,
                enabled: prefs.show_git,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Branch",
                pref_key: "Git Branch",
                depth: 1,
                parent: Some("Git"),
                is_last_child: false,
                enabled: prefs.show_git_branch,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Status",
                pref_key: "Git Status",
                depth: 1,
                parent: Some("Git"),
                is_last_child: true,
                enabled: prefs.show_git_status,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Current Directory",
                pref_key: "Current Directory",
                depth: 0,
                parent: None,
                is_last_child: false,
                enabled: prefs.show_current_dir,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Model",
                pref_key: "Model",
                depth: 0,
                parent: None,
                is_last_child: false,
                enabled: prefs.show_model,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Update Available",
                pref_key: "Update Available",
                depth: 0,
                parent: None,
                is_last_child: false,
                enabled: prefs.show_update_available,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Icons",
                pref_key: "Icons",
                depth: 0,
                parent: None,
                is_last_child: false,
                enabled: prefs.use_icons,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Activity",
                pref_key: "Activity Icon",
                depth: 1,
                parent: Some("Icons"),
                is_last_child: false,
                enabled: prefs.show_activity_icon,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Git",
                pref_key: "Git Icon",
                depth: 1,
                parent: Some("Icons"),
                is_last_child: false,
                enabled: prefs.show_git_icon,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Directory",
                pref_key: "Directory Icon",
                depth: 1,
                parent: Some("Icons"),
                is_last_child: false,
                enabled: prefs.show_directory_icon,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Model",
                pref_key: "Model Icon",
                depth: 1,
                parent: Some("Icons"),
                is_last_child: true,
                enabled: prefs.show_model_icon,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Colors",
                pref_key: "Colors",
                depth: 0,
                parent: None,
                is_last_child: false,
                enabled: prefs.use_colors,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Separators",
                pref_key: "Separators",
                depth: 0,
                parent: None,
                is_last_child: false,
                enabled: prefs.display.show_separators,
                option_type: OptionType::Select {
                    choices: SEPARATOR_OPTIONS,
                    current_index: separator_index,
                },
            },
            ConfigOption {
                name: "Debug Info",
                pref_key: "Debug Info",
                depth: 0,
                parent: None,
                is_last_child: false,
                enabled: prefs.display.show_debug_info,
                option_type: OptionType::Toggle,
            },
        ]
    }

    /// Cycle through selection options (for Select type options)
    fn cycle_selection(&mut self, direction: i32) {
        if self.cursor >= self.options.len() {
            return; // On Done button or separator
        }

        let opt = &self.options[self.cursor];
        if let OptionType::Select {
            choices,
            current_index,
        } = &opt.option_type
        {
            let new_index =
                ((*current_index as i32 + direction).rem_euclid(choices.len() as i32)) as usize;

            // Update preferences based on pref_key
            if opt.pref_key == "Separators" {
                self.prefs.display.separator_char = choices[new_index].to_string();
            }

            // Rebuild options to reflect the change
            self.options = Self::build_options(&self.prefs);
        }
    }

    /// Check if a parent option is enabled (for determining if children are interactive)
    fn is_parent_enabled(&self, parent_name: &str) -> bool {
        self.options
            .iter()
            .find(|opt| opt.pref_key == parent_name || opt.name == parent_name)
            .map(|opt| opt.enabled)
            .unwrap_or(true)
    }

    /// Check if the option at the given index is interactive (can be toggled)
    fn is_option_interactive(&self, idx: usize) -> bool {
        if idx >= self.options.len() {
            return true; // "Done" button is always interactive
        }
        let opt = &self.options[idx];
        if opt.depth == 0 {
            return true; // Top-level options are always interactive
        }
        // Children are interactive only if their parent is enabled
        if let Some(parent) = opt.parent {
            self.is_parent_enabled(parent)
        } else {
            true
        }
    }

    /// Toggle the option at the current cursor position
    fn toggle_current(&mut self) {
        if self.cursor < self.options.len() {
            // Don't toggle disabled children
            if !self.is_option_interactive(self.cursor) {
                return;
            }

            // Get the pref_key of the option to toggle
            let pref_key = self.options[self.cursor].pref_key;

            // Get current selections based on pref_key
            let mut selections: Vec<&str> = self
                .options
                .iter()
                .filter_map(|opt| {
                    if opt.enabled {
                        Some(opt.pref_key)
                    } else {
                        None
                    }
                })
                .collect();

            // Toggle the selected one
            if selections.contains(&pref_key) {
                selections.retain(|&x| x != pref_key);
            } else {
                selections.push(pref_key);
            }

            // Update preferences
            self.prefs.update_from_selections(&selections);

            // Refresh options list
            self.options = Self::build_options(&self.prefs);
        } else if self.cursor == self.options.len() {
            // "Done" option selected
            self.should_quit = true;
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            // Skip disabled children
            while self.cursor > 0 && !self.is_option_interactive(self.cursor) {
                self.cursor -= 1;
            }
        }
    }

    fn move_cursor_down(&mut self) {
        // +1 for "Done" option
        if self.cursor < self.options.len() {
            self.cursor += 1;
            // Skip disabled children
            while self.cursor < self.options.len() && !self.is_option_interactive(self.cursor) {
                self.cursor += 1;
            }
        }
    }
}

/// Create a sample session state for preview
fn create_preview_state() -> SessionState {
    SessionState {
        session_id: "preview".to_string(),
        activity: Activity::Coding,
        current_job: None,
        current_file: Some("main.rs".to_string()),
        git_branch: Some("main".to_string()),
        git_dirty: Some(true),    // Show dirty state in preview
        git_dirty_count: Some(3), // Show 3 dirty files in preview
        git_status_checked_at: None,
        personality: "ლ(╹◡╹ლ) Cowder".to_string(),
        previous_personality: None,
        consecutive_actions: 5,
        error_count: 1,
        recent_activities: vec![Activity::Editing, Activity::Reading],
        mood: crate::state::MoodState::default(),
    }
}

/// Create a sample workspace for preview
fn create_preview_workspace() -> WorkspaceInfo {
    WorkspaceInfo {
        current_dir: Some("/home/user/projects/claude-code-personalities".to_string()),
        project_dir: Some("/home/user/projects/claude-code-personalities".to_string()),
    }
}

/// Run the interactive configuration TUI
pub async fn run_config_ui(prefs: PersonalityPreferences) -> Result<PersonalityPreferences> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = ConfigApp::new(prefs);

    // Run event loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result?;

    Ok(app.prefs)
}

/// Main event loop
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut ConfigApp,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if app.should_quit {
            break;
        }

        // Handle keyboard events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.move_cursor_up();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.move_cursor_down();
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            app.cycle_selection(-1);
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            app.cycle_selection(1);
                        }
                        KeyCode::Char(' ') => {
                            app.toggle_current();
                        }
                        KeyCode::Enter => {
                            if app.cursor == app.options.len() {
                                app.should_quit = true;
                            } else {
                                app.toggle_current();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

/// Render the UI
fn ui(f: &mut Frame, app: &ConfigApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Preview section (compact)
            Constraint::Min(10),   // Options list
            Constraint::Length(3), // Help text
        ])
        .split(f.area());

    // Render preview
    render_preview(f, chunks[0], app);

    // Render options list
    render_options(f, chunks[1], app);

    // Render help text
    render_help(f, chunks[2]);
}

/// Render the statusline preview
fn render_preview(f: &mut Frame, area: Rect, app: &ConfigApp) {
    let state = create_preview_state();
    let workspace = create_preview_workspace();
    let statusline = build_statusline(&state, "Sonnet", &app.prefs, Some(&workspace), None);

    // Fancy block with decorative title
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled("✨ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "Statusline Preview",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ✨", Style::default().fg(Color::Yellow)),
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::horizontal(2)); // Horizontal padding only

    // Convert ANSI codes to ratatui Text
    let mut text = statusline
        .as_bytes()
        .to_vec()
        .into_text()
        .unwrap_or_else(|_| {
            // Fallback: if ANSI parsing fails, show raw text
            Text::raw(statusline.clone())
        });

    // Add empty line before to vertically center the single-line statusline
    text.lines.insert(0, Line::from(""));

    let preview_widget = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(preview_widget, area);
}

/// Render the options list with tree-style hierarchy
fn render_options(f: &mut Frame, area: Rect, app: &ConfigApp) {
    let mut items: Vec<ListItem> = app
        .options
        .iter()
        .enumerate()
        .map(|(idx, opt)| {
            // Determine if this child's parent is enabled
            let is_child_disabled = opt.depth > 0
                && opt
                    .parent
                    .map(|p| !app.is_parent_enabled(p))
                    .unwrap_or(false);

            // Tree prefix for children
            let tree_prefix = if opt.depth > 0 {
                if opt.is_last_child {
                    " └─ "
                } else {
                    " ├─ "
                }
            } else {
                ""
            };

            // Checkbox with checkmark or empty
            let checkbox = if opt.enabled { "[✓]" } else { "[ ]" };
            let check_color = if is_child_disabled {
                Color::DarkGray
            } else if opt.enabled {
                Color::Green
            } else {
                Color::DarkGray
            };

            // Text color based on disabled state
            let text_color = if is_child_disabled {
                Color::DarkGray
            } else {
                Color::White
            };

            // Build the content line
            let mut spans = Vec::new();

            // Tree connector (dimmed)
            if !tree_prefix.is_empty() {
                spans.push(Span::styled(
                    tree_prefix,
                    Style::default().fg(Color::DarkGray),
                ));
            }

            // Checkbox
            spans.push(Span::styled(checkbox, Style::default().fg(check_color)));
            spans.push(Span::raw(" "));

            // Option name
            spans.push(Span::styled(opt.name, Style::default().fg(text_color)));

            // For Select type options, show the current value with cycling arrows
            if let OptionType::Select {
                choices,
                current_index,
            } = &opt.option_type
            {
                let current_value = choices[*current_index];
                let is_selected = idx == app.cursor;

                spans.push(Span::raw("  ")); // spacing

                if is_selected {
                    // Show arrows when selected to indicate cycling
                    spans.push(Span::styled("◀ ", Style::default().fg(Color::Cyan)));
                    spans.push(Span::styled(
                        current_value,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::styled(" ▶", Style::default().fg(Color::Cyan)));
                } else {
                    // Just show the value when not selected
                    spans.push(Span::styled(
                        current_value,
                        Style::default().fg(Color::DarkGray),
                    ));
                }
            }

            let content = Line::from(spans);

            // Highlight selected row
            let style = if idx == app.cursor {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    // Add separator before Done button
    let separator_line = Line::from(Span::styled(
        "─────────────────────────────",
        Style::default().fg(Color::DarkGray),
    ));
    items.push(ListItem::new(separator_line));

    // Add "Done" button with pill-style design
    let is_done_selected = app.cursor == app.options.len();

    let done_content = if is_done_selected {
        // Selected state: highlighted pill button
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                " ✓ Save & Exit ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        // Unselected state: subtle outline button
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("[ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Save & Exit", Style::default().fg(Color::White)),
            Span::styled(" ]", Style::default().fg(Color::DarkGray)),
        ])
    };

    items.push(ListItem::new(done_content));

    let list = List::new(items).block(
        Block::default()
            .title("Configure Display Options")
            .borders(Borders::ALL)
            .padding(Padding::new(2, 2, 1, 1)) // left, right, top, bottom
            .style(Style::default().fg(Color::White)),
    );

    f.render_widget(list, area);
}

/// Render help text
fn render_help(f: &mut Frame, area: Rect) {
    let help_text =
        Paragraph::new("↑↓/jk Navigate • ←→/hl Cycle • Space Toggle • Enter/q/Ctrl+C Save & Exit")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));

    f.render_widget(help_text, area);
}
