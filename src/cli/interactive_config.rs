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

use crate::config::{PersonalityPreferences, StatuslineSection};
use crate::state::SessionState;
use crate::statusline::{WorkspaceInfo, build_statusline_with_positions};
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
    /// Move section position in statusline (left/right arrows)
    Move { section: StatuslineSection },
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
            // Personality section with Move child
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
                name: "Move",
                pref_key: "Move Personality",
                depth: 1,
                parent: Some("Personality"),
                is_last_child: true,
                enabled: true, // Move is always "enabled" (just a control)
                option_type: OptionType::Move {
                    section: StatuslineSection::Personality,
                },
            },
            // Activity section with Move, Icon, Label, and Context children
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
                name: "Move",
                pref_key: "Move Activity",
                depth: 1,
                parent: Some("Activity"),
                is_last_child: false,
                enabled: true,
                option_type: OptionType::Move {
                    section: StatuslineSection::Activity,
                },
            },
            ConfigOption {
                name: "Icon",
                pref_key: "Activity Icon",
                depth: 1,
                parent: Some("Activity"),
                is_last_child: false,
                enabled: prefs.show_activity_icon,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Label",
                pref_key: "Activity Label",
                depth: 1,
                parent: Some("Activity"),
                is_last_child: false,
                enabled: prefs.show_activity_label,
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
            // Git section with Move, Icon, Branch, and Status children
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
                name: "Move",
                pref_key: "Move Git",
                depth: 1,
                parent: Some("Git"),
                is_last_child: false,
                enabled: true,
                option_type: OptionType::Move {
                    section: StatuslineSection::Git,
                },
            },
            ConfigOption {
                name: "Icon",
                pref_key: "Git Icon",
                depth: 1,
                parent: Some("Git"),
                is_last_child: false,
                enabled: prefs.show_git_icon,
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
            // Directory section with Move, Icon and Label children
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
                name: "Move",
                pref_key: "Move Directory",
                depth: 1,
                parent: Some("Current Directory"),
                is_last_child: false,
                enabled: true,
                option_type: OptionType::Move {
                    section: StatuslineSection::Directory,
                },
            },
            ConfigOption {
                name: "Icon",
                pref_key: "Directory Icon",
                depth: 1,
                parent: Some("Current Directory"),
                is_last_child: false,
                enabled: prefs.show_directory_icon,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Label",
                pref_key: "Directory Label",
                depth: 1,
                parent: Some("Current Directory"),
                is_last_child: true,
                enabled: prefs.show_directory_label,
                option_type: OptionType::Toggle,
            },
            // Model section with Move, Icon and Label children
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
                name: "Move",
                pref_key: "Move Model",
                depth: 1,
                parent: Some("Model"),
                is_last_child: false,
                enabled: true,
                option_type: OptionType::Move {
                    section: StatuslineSection::Model,
                },
            },
            ConfigOption {
                name: "Icon",
                pref_key: "Model Icon",
                depth: 1,
                parent: Some("Model"),
                is_last_child: false,
                enabled: prefs.show_model_icon,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Label",
                pref_key: "Model Label",
                depth: 1,
                parent: Some("Model"),
                is_last_child: true,
                enabled: prefs.show_model_label,
                option_type: OptionType::Toggle,
            },
            // Update Available with Move child
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
                name: "Move",
                pref_key: "Move Update",
                depth: 1,
                parent: Some("Update Available"),
                is_last_child: true,
                enabled: true,
                option_type: OptionType::Move {
                    section: StatuslineSection::UpdateAvailable,
                },
            },
            // Colors (standalone, not reorderable)
            ConfigOption {
                name: "Colors",
                pref_key: "Colors",
                depth: 0,
                parent: None,
                is_last_child: false,
                enabled: prefs.use_colors,
                option_type: OptionType::Toggle,
            },
            // Separators (standalone, not reorderable)
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
            // Debug Info with Move child
            ConfigOption {
                name: "Debug Info",
                pref_key: "Debug Info",
                depth: 0,
                parent: None,
                is_last_child: false,
                enabled: prefs.display.show_debug_info,
                option_type: OptionType::Toggle,
            },
            ConfigOption {
                name: "Move",
                pref_key: "Move Debug",
                depth: 1,
                parent: Some("Debug Info"),
                is_last_child: true,
                enabled: true,
                option_type: OptionType::Move {
                    section: StatuslineSection::DebugInfo,
                },
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

    /// Move a section left or right in the statusline order
    fn move_section(&mut self, direction: i32) {
        if self.cursor >= self.options.len() {
            return;
        }

        let opt = &self.options[self.cursor];
        if let OptionType::Move { section } = &opt.option_type {
            // Find current position of the section
            if let Some(idx) = self.prefs.section_order.iter().position(|s| s == section) {
                let len = self.prefs.section_order.len();
                let new_idx = (idx as i32 + direction).clamp(0, len as i32 - 1) as usize;

                // Only swap if actually moving
                if new_idx != idx {
                    self.prefs.section_order.swap(idx, new_idx);
                    // Rebuild options to reflect the change (preview will update)
                    self.options = Self::build_options(&self.prefs);
                }
            }
        }
    }

    /// Handle left/right key press - routes to move_section or cycle_selection
    fn handle_horizontal_key(&mut self, direction: i32) {
        if self.cursor >= self.options.len() {
            return;
        }

        match &self.options[self.cursor].option_type {
            OptionType::Move { .. } => self.move_section(direction),
            OptionType::Select { .. } => self.cycle_selection(direction),
            OptionType::Toggle => {} // No horizontal action for toggles
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
    /// All options are now always interactive to prevent stuck states.
    /// Children of disabled parents are visually dimmed but can still be toggled
    /// (which auto-enables their parent).
    fn is_option_interactive(&self, _idx: usize) -> bool {
        true
    }

    /// Toggle the option at the current cursor position
    fn toggle_current(&mut self) {
        if self.cursor < self.options.len() {
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

            // Determine if we're enabling or disabling
            let is_enabling = !selections.contains(&pref_key);

            // Toggle the selected one
            if is_enabling {
                selections.push(pref_key);
            } else {
                selections.retain(|&x| x != pref_key);
            }

            // Update preferences
            self.prefs.update_from_selections(&selections);

            // Smart auto-enable/disable based on action
            if is_enabling {
                // Enabling a child? Auto-enable its parent too
                self.auto_enable_parent_if_needed(pref_key);
            } else {
                // Disabling something? Auto-disable empty parents
                self.auto_disable_empty_parents();
            }

            // Refresh options list
            self.options = Self::build_options(&self.prefs);
        } else if self.cursor == self.options.len() {
            // "Done" option selected
            self.should_quit = true;
        }
    }

    /// Auto-disable parent sections when all their children are disabled
    fn auto_disable_empty_parents(&mut self) {
        // Activity: disable if Icon=false AND Label=false AND Context=false
        if !self.prefs.show_activity_icon
            && !self.prefs.show_activity_label
            && !self.prefs.show_context
        {
            self.prefs.show_activity = false;
        }

        // Git: disable if Icon=false AND Branch=false AND Status=false
        if !self.prefs.show_git_icon && !self.prefs.show_git_branch && !self.prefs.show_git_status {
            self.prefs.show_git = false;
        }

        // Directory: disable if Icon=false AND Label=false
        if !self.prefs.show_directory_icon && !self.prefs.show_directory_label {
            self.prefs.show_current_dir = false;
        }

        // Model: disable if Icon=false AND Label=false
        if !self.prefs.show_model_icon && !self.prefs.show_model_label {
            self.prefs.show_model = false;
        }
    }

    /// Auto-enable parent section when enabling a child option
    fn auto_enable_parent_if_needed(&mut self, pref_key: &str) {
        match pref_key {
            "Activity Icon" | "Activity Label" | "Activity Context" => {
                self.prefs.show_activity = true;
            }
            "Git Icon" | "Git Branch" | "Git Status" => {
                self.prefs.show_git = true;
            }
            "Directory Icon" | "Directory Label" => {
                self.prefs.show_current_dir = true;
            }
            "Model Icon" | "Model Label" => {
                self.prefs.show_model = true;
            }
            _ => {}
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

/// Map a config option to its corresponding statusline section
fn get_section_for_option(opt: &ConfigOption) -> Option<StatuslineSection> {
    match opt.pref_key {
        "Personality" | "Move Personality" => Some(StatuslineSection::Personality),
        "Activity" | "Move Activity" | "Activity Icon" | "Activity Label" | "Activity Context" => {
            Some(StatuslineSection::Activity)
        }
        "Git" | "Move Git" | "Git Icon" | "Git Branch" | "Git Status" => {
            Some(StatuslineSection::Git)
        }
        "Current Directory" | "Move Directory" | "Directory Icon" | "Directory Label" => {
            Some(StatuslineSection::Directory)
        }
        "Model" | "Move Model" | "Model Icon" | "Model Label" => Some(StatuslineSection::Model),
        "Update Available" | "Move Update" => Some(StatuslineSection::UpdateAvailable),
        "Debug Info" | "Move Debug" => Some(StatuslineSection::DebugInfo),
        // Colors, Separators don't map to a specific section
        _ => None,
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
                            app.handle_horizontal_key(-1);
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            app.handle_horizontal_key(1);
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
            Constraint::Length(6), // Preview section (statusline + indicator)
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

/// Render the statusline preview with section indicator
fn render_preview(f: &mut Frame, area: Rect, app: &ConfigApp) {
    let state = create_preview_state();
    let workspace = create_preview_workspace();
    let (statusline, positions) =
        build_statusline_with_positions(&state, "Sonnet", &app.prefs, Some(&workspace), None);

    // Get the section for the currently selected option
    let highlighted_section = if app.cursor < app.options.len() {
        get_section_for_option(&app.options[app.cursor])
    } else {
        None
    };

    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            "Preview",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]))
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

    // Build indicator line if we have a highlighted section
    let indicator_line = if let Some(section) = highlighted_section {
        if let Some(&(start, width)) = positions.positions.get(&section) {
            // Create indicator line: spaces up to start, then underline chars for width
            let mut indicator = String::new();
            for _ in 0..start {
                indicator.push(' ');
            }
            for _ in 0..width {
                indicator.push('─'); // Box drawing horizontal
            }
            Line::from(Span::styled(indicator, Style::default().fg(Color::Cyan)))
        } else {
            Line::from("") // Section not in current statusline (e.g., disabled)
        }
    } else {
        Line::from("") // No section highlighted (Colors/Separators/Done)
    };

    // Build the text: empty line + statusline + indicator
    text.lines.insert(0, Line::from(""));
    text.lines.push(indicator_line);

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

            // Handle different option types
            match &opt.option_type {
                OptionType::Move { section } => {
                    // No checkbox for Move - just show position indicator
                    let is_selected = idx == app.cursor;

                    // Find current position in section_order
                    let position = app
                        .prefs
                        .section_order
                        .iter()
                        .position(|s| s == section)
                        .map(|p| p + 1) // 1-indexed for display
                        .unwrap_or(0);
                    let total = app.prefs.section_order.len();

                    // Option name
                    spans.push(Span::styled(opt.name, Style::default().fg(text_color)));
                    spans.push(Span::raw("  ")); // spacing

                    // Position indicator with arrows when selected
                    let pos_str = format!("{position}/{total}");
                    if is_selected {
                        spans.push(Span::styled("◀ ", Style::default().fg(Color::Cyan)));
                        spans.push(Span::styled(
                            pos_str,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ));
                        spans.push(Span::styled(" ▶", Style::default().fg(Color::Cyan)));
                    } else {
                        spans.push(Span::styled(
                            format!("  {pos_str}  "),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                }
                OptionType::Select {
                    choices,
                    current_index,
                } => {
                    // Checkbox
                    spans.push(Span::styled(checkbox, Style::default().fg(check_color)));
                    spans.push(Span::raw(" "));

                    // Option name
                    spans.push(Span::styled(opt.name, Style::default().fg(text_color)));

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
                OptionType::Toggle => {
                    // Checkbox
                    spans.push(Span::styled(checkbox, Style::default().fg(check_color)));
                    spans.push(Span::raw(" "));

                    // Option name
                    spans.push(Span::styled(opt.name, Style::default().fg(text_color)));
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
