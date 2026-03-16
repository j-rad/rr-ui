// src/cli/tui.rs
use anyhow::Result;
use colored::*;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

pub struct TuiApp {
    state: ListState,
    items: Vec<MenuItem>,
}

#[derive(Clone)]
struct MenuItem {
    number: usize,
    label: String,
    description: String,
}

impl TuiApp {
    pub fn new() -> Self {
        let items = vec![
            MenuItem {
                number: 0,
                label: "Exit".to_string(),
                description: "Exit the management panel".to_string(),
            },
            MenuItem {
                number: 1,
                label: "Start Service".to_string(),
                description: "Start rr-ui service".to_string(),
            },
            MenuItem {
                number: 2,
                label: "Stop Service".to_string(),
                description: "Stop rr-ui service".to_string(),
            },
            MenuItem {
                number: 3,
                label: "Restart Service".to_string(),
                description: "Restart rr-ui service".to_string(),
            },
            MenuItem {
                number: 4,
                label: "Service Status".to_string(),
                description: "Show service status and metrics".to_string(),
            },
            MenuItem {
                number: 5,
                label: "Security & SSL".to_string(),
                description: "Manage SSL certificates".to_string(),
            },
            MenuItem {
                number: 6,
                label: "Enable BBR".to_string(),
                description: "Enable TCP BBR congestion control".to_string(),
            },
            MenuItem {
                number: 7,
                label: "Network Speedtest".to_string(),
                description: "Test network bandwidth".to_string(),
            },
            MenuItem {
                number: 8,
                label: "Sync Geo Assets".to_string(),
                description: "Download latest GeoIP/GeoSite".to_string(),
            },
            MenuItem {
                number: 9,
                label: "Reset Admin Credentials".to_string(),
                description: "Reset username/password to admin".to_string(),
            },
            MenuItem {
                number: 10,
                label: "Change Panel Port".to_string(),
                description: "Modify web panel port".to_string(),
            },
            MenuItem {
                number: 11,
                label: "Change Panel Path".to_string(),
                description: "Modify web panel access path".to_string(),
            },
            MenuItem {
                number: 12,
                label: "Reset 2FA".to_string(),
                description: "Disable two-factor authentication".to_string(),
            },
            MenuItem {
                number: 13,
                label: "System Update".to_string(),
                description: "Check and install updates".to_string(),
            },
            MenuItem {
                number: 14,
                label: "Enable Autostart".to_string(),
                description: "Enable service on boot".to_string(),
            },
            MenuItem {
                number: 15,
                label: "Disable Autostart".to_string(),
                description: "Disable service on boot".to_string(),
            },
            MenuItem {
                number: 16,
                label: "View Logs".to_string(),
                description: "Display service logs".to_string(),
            },
        ];

        let mut state = ListState::default();
        state.select(Some(0));

        Self { state, items }
    }

    pub fn run(&mut self) -> Result<Option<usize>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn run_app(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<Option<usize>> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                    KeyCode::Down | KeyCode::Char('j') => self.next(),
                    KeyCode::Up | KeyCode::Char('k') => self.previous(),
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if let Some(i) = self.state.selected() {
                            return Ok(Some(self.items[i].number));
                        }
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        // Allow multi-digit input? No, single digit only for now.
                        // But we have > 9 items. Key handling for '1' then '0' is harder.
                        // Simplified: Only support up to 9 via hotkey or just arrows.
                        // Let's rely on Arrows/Enter for >9.
                        let num = c.to_digit(10).unwrap() as usize;
                        if num == 1 {
                            // crude check for 10-16 ... implementing multi-digit is too complex for this snippet.
                            // Just stick to arrows.
                        }
                        if let Some(pos) = self.items.iter().position(|item| item.number == num) {
                            self.state.select(Some(pos));
                            // return Ok(Some(num)); // Don't auto-select on number press, just highlight?
                            // Or standard behavior: Press number -> Selects it.
                            // But 1 vs 10...
                            // Let's comment out direct number selection or accept single digits only.
                            // self.state.select(Some(pos));
                            // return Ok(Some(num));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        // Header
        let header = Paragraph::new(vec![Line::from(Span::styled(
            "RR-UI Management Panel",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Cyan)),
        );

        f.render_widget(header, chunks[0]);

        // Menu items
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| {
                let content = Line::from(vec![
                    Span::styled(
                        format!(" {} ", item.number),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(&item.label, Style::default().fg(Color::White)),
                    Span::raw(" - "),
                    Span::styled(&item.description, Style::default().fg(Color::DarkGray)),
                ]);
                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Menu"))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, chunks[1], &mut self.state);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::raw("Navigate: "),
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw(" | Select: "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" | Exit: "),
            Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
        ]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        f.render_widget(footer, chunks[2]);
    }
}

// Simple fallback menu for environments without terminal support
pub fn simple_menu() -> Result<Option<usize>> {
    use std::io::{self, Write};

    loop {
        println!();
        println!(
            "{}",
            "╔════════════════════════════════════════════════╗".cyan()
        );
        println!(
            "{}",
            "║     RR-UI Management Panel (x-ui.sh Style)    ║"
                .cyan()
                .bold()
        );
        println!(
            "{}",
            "╚════════════════════════════════════════════════╝".cyan()
        );
        println!();
        println!("  {}  {}", "0.".yellow().bold(), "Exit");
        println!("  {}  {}", "1.".yellow().bold(), "Start Service");
        println!("  {}  {}", "2.".yellow().bold(), "Stop Service");
        println!("  {}  {}", "3.".yellow().bold(), "Restart Service");
        println!("  {}  {}", "4.".yellow().bold(), "Service Status");
        println!("  {}  {}", "5.".yellow().bold(), "Security & SSL");
        println!("  {}  {}", "6.".yellow().bold(), "Enable BBR");
        println!("  {}  {}", "7.".yellow().bold(), "Network Speedtest");
        println!("  {}  {}", "8.".yellow().bold(), "Sync Geo Assets");
        println!("  {}  {}", "9.".yellow().bold(), "Reset Admin Credentials");
        println!("  {}  {}", "10.".yellow().bold(), "Change Panel Port");
        println!("  {}  {}", "11.".yellow().bold(), "Change Panel Path");
        println!("  {}  {}", "12.".yellow().bold(), "Reset 2FA");
        println!("  {}  {}", "13.".yellow().bold(), "System Update");
        println!("  {}  {}", "14.".yellow().bold(), "Enable Autostart");
        println!("  {}  {}", "15.".yellow().bold(), "Disable Autostart");
        println!("  {}  {}", "16.".yellow().bold(), "View Logs");
        println!();
        print!("{}", "Enter choice [0-16]: ".green().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<usize>() {
            Ok(n) if n <= 16 => return Ok(Some(n)),
            _ => println!(
                "{}",
                "Invalid choice. Please enter a number between 0-16.".red()
            ),
        }
    }
}
