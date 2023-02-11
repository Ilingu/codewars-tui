pub mod custom_widgets;
pub mod types;
pub mod utils;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use custom_widgets::StatefulList;
use std::{error::Error, fmt::format, vec};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use types::{CodewarsCLI, InputMode, DIFFICULTY, LANGAGE, SORT_BY, TAGS};

use crate::utils::gen_rand_colors;
use urlencoding::encode;

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

const CODEWARS_ENDPOINT: &str = "https://www.codewars.com/kata/search";

const TERMINAL_REF_SIZE: (u16, u16) = (147, 34);

impl CodewarsCLI<'_> {
    pub fn new() -> CodewarsCLI<'static> {
        CodewarsCLI {
            input_mode: InputMode::Normal,
            terminal_size: (0, 0),
            dropdown: (false, StatefulList::with_items(vec![], 0)),
            search_result: vec![],
            search_field: String::new(),
            sortby_field: 0,
            langage_field: 0,
            difficulty_field: 0,
            tag_field: 0,
        }
    }

    pub fn change_state(&mut self, new_state: InputMode) {
        self.input_mode = new_state;

        // hide dropdown if necessary (normally impossible but never have faith in users)
        match self.input_mode {
            InputMode::Normal | InputMode::Search => self.hide_dropdown(),
            _ => {}
        };
    }

    pub fn show_dropdown(&mut self) {
        let selected: usize = match self.input_mode {
            InputMode::SortBy => self.sortby_field,
            InputMode::Langage => self.langage_field,
            InputMode::Difficulty => self.difficulty_field,
            InputMode::Tags => self.tag_field,
            _ => 0,
        };

        let datas = match self.input_mode {
            InputMode::SortBy => Vec::from(SORT_BY),
            InputMode::Langage => Vec::from(LANGAGE),
            InputMode::Difficulty => Vec::from(DIFFICULTY),
            InputMode::Tags => Vec::from(TAGS),
            _ => vec![],
        }
        .iter()
        .enumerate()
        .map(|(i, d)| (*d, i))
        .collect::<Vec<(&str, usize)>>();

        self.dropdown = (true, StatefulList::with_items(datas, selected));
    }

    pub fn hide_dropdown(&mut self) {
        self.dropdown = (false, StatefulList::with_items(vec![], 0))
    }

    pub fn submit_search(&self) {
        // query args
        let query = format!("?q={}", encode(self.search_field.as_str()));

        // sortby args
        let sortby_value = match SORT_BY[self.sortby_field] {
            "Oldest" => "published_at%20asc",
            "Popularity" => "popularity%20desc",
            "Positive Feedback" => "satisfaction_percent%20desc",
            "Most Completed" => "total_completed%20desc",
            "Least Completed" => "total_completed%20asc",
            "Recently Published" => "published_at%20desc",
            "Hardest" => "rank_id%20desc",
            "Easiest" => "rank_id%20asc",
            "Name" => "name%20asc",
            "Low Satisfaction" => "satisfaction_percent%20asc",
            _ => "",
        }
        .to_string();
        let sortby = if sortby_value.len() <= 0 {
            String::new()
        } else {
            format!("&order_by={sortby_value}")
        };

        // language path
        let language = match LANGAGE[self.langage_field] {
            "All" => String::new(),
            "C++" => "cpp".to_string(),
            "Objective-C" => "objc".to_string(),
            "C#" => "csharp".to_string(),
            "F#" => "fsharp".to_string(),
            "λ Calculus" => "lambdacalc".to_string(),
            "RISC-V" => "riscv".to_string(),
            l => l.to_lowercase().trim().replace(" ", "-"),
        };

        // difficulty args
        let difficulty = if self.difficulty_field == 0 {
            String::new()
        } else {
            format!("&r%5B%5D=-{}", self.difficulty_field)
        };

        // tags args
        let tags = if self.tag_field == 0 {
            String::new()
        } else {
            format!("&tags={}", encode(TAGS[self.tag_field]))
        };

        // fetching
        let url = format!("{CODEWARS_ENDPOINT}/{language}{query}{sortby}{difficulty}{tags}");
        println!("{url}");
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
    state.terminal_size = size()?;
    loop {
        terminal.draw(|f| ui(f, state))?;
        // println!("{:?}", state.terminal_size);

        match event::read()? {
            Event::Resize(w, h) => state.terminal_size = (w, h),
            Event::Paste(data) => match state.input_mode {
                InputMode::Search => {
                    state.search_field.push_str(data.as_str());
                }
                _ => {}
            },
            Event::Mouse(mouse_ev) => {
                if mouse_ev.kind == MouseEventKind::Down(event::MouseButton::Left) {
                    let delta_gap = (
                        (state.terminal_size.0 as f32 - TERMINAL_REF_SIZE.0 as f32) * 0.3, // *0.3 = -70% (because this section have 30% of all screen, see ui())
                        state.terminal_size.1 as i16 - TERMINAL_REF_SIZE.1 as i16,
                    );

                    if mouse_ev.column as i16 >= 2 && mouse_ev.column as f32 <= delta_gap.0 + 42.0 {
                        if mouse_ev.row as i16 >= delta_gap.1 + 16
                            && mouse_ev.row as i16 <= delta_gap.1 + 19
                        {
                            state.change_state(InputMode::Search)
                        }
                        if mouse_ev.row as i16 >= delta_gap.1 + 20
                            && mouse_ev.row as i16 <= delta_gap.1 + 22
                        {
                            state.change_state(InputMode::SortBy)
                        }
                        if mouse_ev.row as i16 >= delta_gap.1 + 23
                            && mouse_ev.row as i16 <= delta_gap.1 + 25
                        {
                            state.change_state(InputMode::Langage)
                        }
                        if mouse_ev.row as i16 >= delta_gap.1 + 26
                            && mouse_ev.row as i16 <= delta_gap.1 + 28
                        {
                            state.change_state(InputMode::Difficulty)
                        }
                        if mouse_ev.row as i16 >= delta_gap.1 + 29
                            && mouse_ev.row as i16 <= delta_gap.1 + 32
                        {
                            state.change_state(InputMode::Tags)
                        }
                    }
                }
            }
            Event::Key(key) => {
                if state.dropdown.0 {
                    match key.code {
                        KeyCode::Up => state.dropdown.1.previous(),
                        KeyCode::Down => state.dropdown.1.next(),
                        KeyCode::Enter => {
                            match state.input_mode {
                                InputMode::SortBy => state.sortby_field = state.dropdown.1.state,
                                InputMode::Langage => state.langage_field = state.dropdown.1.state,
                                InputMode::Difficulty => {
                                    state.difficulty_field = state.dropdown.1.state
                                }
                                InputMode::Tags => state.tag_field = state.dropdown.1.state,
                                _ => {}
                            };

                            state.hide_dropdown();
                            state.submit_search();
                        }
                        KeyCode::Esc => state.hide_dropdown(),
                        _ => {}
                    }
                } else {
                    match state.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('s') => state.change_state(InputMode::Search),
                            _ => {}
                        },

                        InputMode::Search => match key.code {
                            KeyCode::Char(c) => state.search_field.push(c),
                            KeyCode::Enter => state.submit_search(),
                            KeyCode::Backspace => {
                                state.search_field.pop();
                            }
                            KeyCode::Tab | KeyCode::Down => state.change_state(InputMode::SortBy),
                            KeyCode::Esc => state.change_state(InputMode::Normal),
                            _ => {}
                        },

                        InputMode::SortBy => match key.code {
                            KeyCode::Enter => state.show_dropdown(),
                            KeyCode::Tab | KeyCode::Down => state.change_state(InputMode::Langage),
                            KeyCode::BackTab | KeyCode::Up => state.change_state(InputMode::Search),
                            KeyCode::Esc => state.change_state(InputMode::Normal),
                            _ => {}
                        },

                        InputMode::Langage => match key.code {
                            KeyCode::Enter => state.show_dropdown(),
                            KeyCode::Tab | KeyCode::Down => {
                                state.change_state(InputMode::Difficulty)
                            }
                            KeyCode::BackTab | KeyCode::Up => state.change_state(InputMode::SortBy),
                            KeyCode::Esc => state.change_state(InputMode::Normal),
                            _ => {}
                        },

                        InputMode::Difficulty => match key.code {
                            KeyCode::Enter => state.show_dropdown(),
                            KeyCode::Tab | KeyCode::Down => state.change_state(InputMode::Tags),
                            KeyCode::BackTab | KeyCode::Up => {
                                state.change_state(InputMode::Langage)
                            }
                            KeyCode::Esc => state.change_state(InputMode::Normal),
                            _ => {}
                        },

                        InputMode::Tags => match key.code {
                            KeyCode::Enter => state.show_dropdown(),
                            KeyCode::BackTab | KeyCode::Up => {
                                state.change_state(InputMode::Difficulty)
                            }
                            KeyCode::Esc => state.change_state(InputMode::Normal),
                            _ => {}
                        },
                    }
                }
            }
            _ => {}
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

fn welcome_text() -> Paragraph<'static> {
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

fn dropdown(state: &mut CodewarsCLI) -> List<'static> {
    let title = match state.input_mode {
        InputMode::SortBy => "Sort by",
        InputMode::Langage => "Select Programming Languages",
        InputMode::Difficulty => "Select Difficulty",
        InputMode::Tags => "Select Tags",
        _ => "",
    };

    let items = state
        .dropdown
        .1
        .items
        .iter()
        .map(|(content, i)| {
            let is_active = i == &state.dropdown.1.state;

            ListItem::new(Spans::from(Span::styled(
                if is_active {
                    ">> ".to_string() + content
                } else {
                    content.to_string()
                },
                Style::default().add_modifier(Modifier::ITALIC),
            )))
            .style(if is_active {
                Style::default()
                    .fg(Color::Rgb(255, 195, 18))
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::default()
            })
        })
        .collect::<Vec<ListItem>>();

    const ITEMS_IN_VIEW_REF: u16 = 26; // for a terminal with 34 rows we can display 26 items of the list
    let items_in_view =
        (((ITEMS_IN_VIEW_REF * state.terminal_size.1) / TERMINAL_REF_SIZE.1) - 1) as usize;
    let items_ranges = if state.dropdown.1.state > items_in_view {
        (state.dropdown.1.state - items_in_view)..=state.dropdown.1.state
    } else {
        0..=items.len() - 1
    };

    return List::new(items[items_ranges].to_owned())
        .block(Block::default().title(title).borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
}

fn draw_search_section<B: Backend>(f: &mut Frame<B>, state: &mut CodewarsCLI, area: Rect) {
    let contraints = if state.dropdown.0 {
        vec![Constraint::Length(2), Constraint::Min(4)]
    } else {
        vec![
            Constraint::Length(2),
            Constraint::Min(4),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(contraints.as_ref())
        .split(area);

    f.render_widget(welcome_text(), chunks[0]);

    if state.dropdown.0 {
        f.render_widget(dropdown(state), chunks[1]);
        return;
    }

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
