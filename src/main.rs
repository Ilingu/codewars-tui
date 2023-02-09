pub mod types;
pub mod utils;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, vec};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};
use types::{CodewarsCLI, InputMode, DIFFICULTY, LANGAGE, SORT_BY, TAGS};

use crate::utils::gen_rand_colors;

/* How it'll work
- when opening it'll fetch from "https://www.codewars.com/kata/search" for the default kata
- parser for html to struct
- UI: on the left some settings for the search (search, sort by, langage, status, progress...) on update re fetch the kata
- rendering all the kata as a list on the right (90% of the screen)
- when user clicks on a kata in the list, close the setting panel and open a detailled view of the kata with a [download] button at the end
- when user clicks on the [download] button, fetch the kata instruction, sample tests, and sample solution at (https://www.codewars.com/kata/<kata-id>/train/<langage>) and then dwonload it to the user specified folder                                                                                                                  //
 */

const APP_KEYS_DESC: &str = r#"
- Actions: 
S:              Search Kata
F:              Filter Result
D:              Download selected Kata

- Moves:
Tab:            Go to next field
Shift+Tab:      Go to previous filed
Esc:            Exit search mode
"#;

impl CodewarsCLI {
    pub fn new() -> CodewarsCLI {
        CodewarsCLI {
            input_mode: InputMode::Normal,
            search_result: vec![],
            is_dropdown: false,
            help_mode: false,
            search_field: String::new(),
            sortby_field: 0,
            langage_field: 0,
            difficulty_field: 0,
            tag_field: 0,
        }
    }

    pub fn change_state(&mut self, new_state: InputMode) {
        self.input_mode = new_state;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut state = CodewarsCLI::new();
    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut state);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    if let Err(e) = result {
        println!("{}", e.to_string());
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut CodewarsCLI,
) -> Result<(), std::io::Error> {
    loop {
        terminal.draw(|f| ui(f, state))?;

        if let Event::Key(key) = event::read()? {
            match state.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('s') => {
                        state.change_state(InputMode::Search);
                    }
                    _ => {}
                },

                InputMode::Search => match key.code {
                    KeyCode::Char(c) => {
                        state.search_field.push(c);
                    }
                    KeyCode::Backspace => {
                        state.search_field.pop();
                    }
                    KeyCode::Esc => {
                        state.change_state(InputMode::Normal);
                    }
                    KeyCode::Tab => {
                        state.change_state(InputMode::SortBy);
                    }
                    _ => {}
                },

                InputMode::SortBy => match key.code {
                    KeyCode::Esc => {
                        state.change_state(InputMode::Normal);
                    }
                    KeyCode::Tab => {
                        state.change_state(InputMode::Langage);
                    }
                    KeyCode::BackTab => {
                        state.change_state(InputMode::Search);
                    }
                    _ => {}
                },

                InputMode::Langage => match key.code {
                    KeyCode::Esc => {
                        state.change_state(InputMode::Normal);
                    }
                    KeyCode::Tab => {
                        state.change_state(InputMode::Difficulty);
                    }
                    KeyCode::BackTab => {
                        state.change_state(InputMode::SortBy);
                    }
                    _ => {}
                },

                InputMode::Difficulty => match key.code {
                    KeyCode::Esc => {
                        state.change_state(InputMode::Normal);
                    }
                    KeyCode::Tab => {
                        state.change_state(InputMode::Tags);
                    }
                    KeyCode::BackTab => {
                        state.change_state(InputMode::Langage);
                    }
                    _ => {}
                },

                InputMode::Tags => match key.code {
                    KeyCode::Esc => {
                        state.change_state(InputMode::Normal);
                    }

                    KeyCode::BackTab => {
                        state.change_state(InputMode::Difficulty);
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, state: &mut CodewarsCLI) {
    let parent_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(f.size());

    let search_section = Block::default()
        .title("Search Katas")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(search_section, parent_chunk[0]);
    draw_search_section(f, state, parent_chunk[0]);

    let list_section_block = Block::default()
        .title("List of katas")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(list_section_block, parent_chunk[1]);
    // list_section(f, state, parent_chunk[1])
}

fn draw_welcome_text() -> Paragraph<'static> {
    let colors = [gen_rand_colors(), gen_rand_colors(), gen_rand_colors()];

    let text = vec![
        Spans::from(vec![
            Span::styled(
                "Welcome",
                Style::default().fg(colors[0]).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                "to",
                Style::default().fg(colors[1]).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                "CodewarsCLI",
                Style::default().fg(colors[2]).add_modifier(Modifier::BOLD),
            ),
        ]),
        Spans::from("A tool to download katas locally"),
        Spans::from(APP_KEYS_DESC),
    ];

    return Paragraph::new(text).alignment(Alignment::Center);
}

fn draw_search_section<B: Backend>(f: &mut Frame<B>, state: &mut CodewarsCLI, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(2),
                Constraint::Min(4),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(area);

    f.render_widget(draw_welcome_text(), chunks[0]);

    let help = Paragraph::new(APP_KEYS_DESC);
    f.render_widget(help, chunks[1]);

    let search = Paragraph::new(state.search_field.to_owned())
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Search Kata"),
        )
        .style(match state.input_mode {
            InputMode::Search => Style::default().fg(Color::LightYellow),
            _ => Style::default(),
        });
    f.render_widget(search, chunks[2]);

    let sortby = Paragraph::new(SORT_BY[state.sortby_field].to_owned())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Sort By"),
        )
        .style(match state.input_mode {
            InputMode::SortBy => Style::default().fg(Color::LightYellow),
            _ => Style::default(),
        });
    f.render_widget(sortby, chunks[3]);

    let language = Paragraph::new(if state.langage_field == 0 {
        Span::styled(
            LANGAGE[state.langage_field].to_owned(),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )
    } else {
        Span::from(LANGAGE[state.langage_field].to_owned())
    })
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Language"),
    )
    .style(match state.input_mode {
        InputMode::Langage => Style::default().fg(Color::LightYellow),
        _ => Style::default(),
    });
    f.render_widget(language, chunks[4]);

    let difficulty = Paragraph::new(if state.difficulty_field == 0 {
        Span::styled(
            DIFFICULTY[state.difficulty_field].to_owned(),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )
    } else {
        Span::from(DIFFICULTY[state.difficulty_field].to_owned())
    })
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Difficulty"),
    )
    .style(match state.input_mode {
        InputMode::Difficulty => Style::default().fg(Color::LightYellow),
        _ => Style::default(),
    });
    f.render_widget(difficulty, chunks[5]);

    let tags = Paragraph::new(if state.tag_field == 0 {
        Span::styled(
            TAGS[state.tag_field].to_owned(),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )
    } else {
        Span::from(TAGS[state.tag_field].to_owned())
    })
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Tags"),
    )
    .style(match state.input_mode {
        InputMode::Tags => Style::default().fg(Color::LightYellow),
        _ => Style::default(),
    });
    f.render_widget(tags, chunks[6]);
}
