use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::{
    types::{
        CodewarsCLI, CursorDirection, DownloadModalInput, InputMode, KataAPI, DIFFICULTY, LANGAGE,
        SORT_BY, TAGS,
    },
    utils::{gen_rand_colors, log_print, rank_color},
    TERMINAL_REF_SIZE,
};

const APP_KEYS_DESC: &str = r#"
- Actions:
q: Quit app (normal mode)
S: Search Kata (normal mode)
L: Focus List of Katas (normal mode)
D: Download selected Kata (list of kata)

- Moves:
Tab:        Go to next field/kata
Shift+Tab:  Go to previous field/kata
Esc:        Exit to normal mode
"#;

// Custom widgets
pub struct StatefulList<T> {
    pub state: usize,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>, initial_state: usize) -> StatefulList<T> {
        StatefulList {
            state: initial_state,
            items,
        }
    }

    pub fn next(&mut self) {
        if self.items.len() <= 0 {
            return;
        }

        if self.state == self.items.len() - 1 {
            self.state = 0
        } else {
            self.state += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.items.len() <= 0 {
            return;
        }

        if self.state == 0 {
            self.state = self.items.len() - 1
        } else {
            self.state -= 1;
        }
    }
}

pub struct InputWidget {
    pub value: String,
    pub cursor_pos: usize,
    pub suggestion: StatefulList<String>,
}

impl InputWidget {
    pub fn default() -> Self {
        Self {
            value: String::new(),
            cursor_pos: 0,
            suggestion: StatefulList::with_items(vec![], 0),
        }
    }

    pub fn push_char(&mut self, ch: char) {
        self.value.insert(self.cursor_pos, ch);
        self.cursor_pos += 1;
    }
    pub fn push_str(&mut self, string: &str) {
        self.value.insert_str(self.cursor_pos, string);
        self.cursor_pos += string.len();
    }
    /// backspace behavior
    pub fn backspace(&mut self) {
        if self.cursor_pos <= 0 {
            return;
        }
        self.value.remove(self.cursor_pos - 1);
        self.cursor_pos -= 1;
    }
    /// 'del' key behavior
    pub fn del(&mut self) {
        if self.cursor_pos == self.value.len() {
            return;
        }
        self.value.remove(self.cursor_pos);
    }

    pub fn set_suggestions(&mut self, suggestions: Vec<String>) {
        self.suggestion = StatefulList::with_items(suggestions, 0)
    }
    pub fn append_suggestions(&mut self, mut suggestions: Vec<String>) {
        self.suggestion.items.append(&mut suggestions);
    }

    pub fn move_cursor(&mut self, direction: CursorDirection) {
        match direction {
            CursorDirection::RIGHT => {
                if self.cursor_pos == self.value.len() {
                    return;
                }
                self.cursor_pos += 1;
            }
            CursorDirection::LEFT => {
                if self.cursor_pos <= 0 {
                    return;
                }
                self.cursor_pos -= 1;
            }
        }
    }

    /// no style, alignment, blocks just the text and cursor and suggestions
    pub fn basic_render(&mut self, is_active: bool) -> Paragraph<'static> {
        let mut text: Vec<Span> = vec![];

        let cursor = if is_active {
            Span::styled(
                "|",
                Style::default()
                    .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK)
                    .fg(Color::White),
            )
        } else {
            Span::from("")
        };

        if self.value.len() <= 0 {
            text.push(cursor);
        } else {
            if self.cursor_pos <= 0 {
                text.push(cursor.clone());
            }

            for (i, ch) in self.value.chars().enumerate() {
                text.push(Span::raw(ch.to_string()));
                if i + 1 == self.cursor_pos {
                    text.push(cursor.clone());
                }
            }
        }

        // suggestions (only if cursor at the end and is_active)
        if is_active && self.cursor_pos == self.value.len() {
            text.push(if self.suggestion.items.len() > 0 {
                Span::styled(
                    self.suggestion.items[self.suggestion.state].to_owned(),
                    Style::default()
                        .add_modifier(Modifier::ITALIC)
                        .fg(Color::DarkGray),
                )
            } else {
                Span::from("")
            });
        }

        return Paragraph::new(Spans::from(text));
    }
}

// APP UI
pub fn ui<B: Backend>(f: &mut Frame<B>, state: &mut CodewarsCLI) {
    let parent_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(f.size());

    let search_section = Block::default()
        .title(Span::styled(
            "Search Katas",
            match state.input_mode {
                InputMode::KataList => Style::default(),
                _ => Style::default().fg(Color::LightRed),
            },
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(match state.input_mode {
            InputMode::KataList => Style::default(),
            _ => Style::default().fg(Color::LightRed),
        });
    f.render_widget(search_section, parent_chunk[0]);
    draw_search_section(f, state, parent_chunk[0]);

    let list_section_block = Block::default()
        .title(Span::styled(
            "List of katas",
            match state.input_mode {
                InputMode::KataList => Style::default().fg(Color::LightRed),
                _ => Style::default(),
            },
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(match state.input_mode {
            InputMode::KataList => Style::default().fg(Color::LightRed),
            _ => Style::default(),
        });
    f.render_widget(list_section_block, parent_chunk[1]);
    if state.download_modal.0 != DownloadModalInput::Disabled {
        draw_download_modal(f, state, parent_chunk[1])
    } else {
        draw_list_section(f, state, parent_chunk[1])
    }
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

fn dropdown(
    dropdown_info: &StatefulList<(String, usize)>,
    input_mode: &InputMode,
    terminal_size: &(u16, u16),
    items_in_views: Option<u16>,
) -> List<'static> {
    let title = match input_mode {
        InputMode::SortBy => "Sort by",
        InputMode::Langage => "Select Programming Language",
        InputMode::Difficulty => "Select Difficulty",
        InputMode::Tags => "Select Tags",
        _ => "",
    };

    let items = dropdown_info
        .items
        .iter()
        .map(|(content, i)| {
            let is_active = i == &dropdown_info.state;

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

    let wanted_item_in_view: u16 = match items_in_views {
        Some(iivr) => iivr,
        None => 26,
    }; // for a terminal with 34 rows we can display 26 items of the list
    let items_in_view =
        (((wanted_item_in_view * terminal_size.1) / TERMINAL_REF_SIZE.1) - 1) as usize;
    let items_ranges = if dropdown_info.state > items_in_view {
        (dropdown_info.state - items_in_view)..=dropdown_info.state
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
    let contraints = if state.field_dropdown.0 {
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

    if state.field_dropdown.0 {
        f.render_widget(
            dropdown(
                &state.field_dropdown.1,
                &state.input_mode,
                &state.terminal_size,
                None,
            ),
            chunks[1],
        );
        return;
    }

    let help = Paragraph::new(APP_KEYS_DESC);
    f.render_widget(help, chunks[1]);

    let search = state
        .search_field
        .basic_render(state.input_mode == InputMode::Search)
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

fn draw_list_section<B: Backend>(f: &mut Frame<B>, state: &mut CodewarsCLI, area: Rect) {
    if state.search_result.items.len() <= 0 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Min(5),
                Constraint::Min(5),
                Constraint::Min(5),
                Constraint::Min(5),
                Constraint::Min(5),
                Constraint::Min(5),
            ]
            .as_ref(),
        )
        .split(area);

    const ITEMS_IN_VIEW_REF: usize = 6 - 1; // for a terminal with 34 rows we can display  items of the list
    let items_ranges = if state.search_result.items.len() - 1 <= ITEMS_IN_VIEW_REF {
        0..=(state.search_result.items.len() - 1)
    } else if state.search_result.state > ITEMS_IN_VIEW_REF {
        (state.search_result.state - ITEMS_IN_VIEW_REF)..=state.search_result.state
    } else {
        0..=ITEMS_IN_VIEW_REF
    };

    for (i, (kata, kata_idx)) in (&state.search_result.items[items_ranges])
        .iter()
        .enumerate()
    {
        let is_active = *kata_idx == state.search_result.state;
        f.render_widget(draw_kata(kata, is_active), chunks[i]);
    }
}

fn draw_kata(kata: &KataAPI, is_active: bool) -> Paragraph<'static> {
    const FG_HEAD: tui::style::Color = Color::Rgb(104, 175, 49);

    let mut tags: Vec<Span> = vec![Span::styled(
        "Tags: ",
        Style::default().fg(Color::LightCyan),
    )];
    for tag in kata.tags.to_owned() {
        tags.push(Span::styled(tag, Style::default().bg(Color::DarkGray)));
        tags.push(Span::raw(" "));
    }

    let mut languages: Vec<Span> = vec![Span::styled(
        "Languages: ",
        Style::default().fg(Color::LightCyan),
    )];
    for language in kata.languages.to_owned() {
        languages.push(Span::styled(language, Style::default().bg(Color::DarkGray)));
        languages.push(Span::raw(" "));
    }

    let text = vec![
        Spans::from(vec![
            Span::styled(
                "Total Completed: ",
                Style::default()
                    .add_modifier(Modifier::ITALIC)
                    .fg(Color::LightCyan),
            ),
            Span::raw(kata.totalCompleted.to_string()),
            Span::styled(
                " | ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Author: ",
                Style::default()
                    .add_modifier(Modifier::ITALIC)
                    .fg(Color::LightCyan),
            ),
            Span::raw(kata.createdBy.username.to_owned()),
        ]),
        Spans::from(tags),
        Spans::from(languages),
    ];

    return Paragraph::new(text)
        .block(
            Block::default()
                .title(Spans::from(vec![
                    Span::styled(
                        kata.name.to_owned(),
                        Style::default().add_modifier(Modifier::BOLD).fg(FG_HEAD),
                    ),
                    Span::raw(" - "),
                    Span::styled(
                        kata.rank.name.to_owned(),
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(rank_color(kata.rank.name.as_str(), Color::White)),
                    ),
                ]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(if is_active {
                    Style::default().fg(rank_color(kata.rank.name.as_str(), Color::LightGreen))
                } else {
                    Style::default().fg(Color::DarkGray)
                }),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });
}

fn draw_download_modal<B: Backend>(f: &mut Frame<B>, state: &mut CodewarsCLI, area: Rect) {
    const ITEM_IN_VIEW: u16 = 18;
    let compute_percent = |no_items: usize| -> u16 {
        // why all these fancy number? Just used regression to find a mathematical law

        // -> affine way
        (((no_items as f64) + 1.80519480519481) / 0.298961038961039).round() as u16

        // -> polynomial way, much more precise on the right interval (from 0% to 65%)
        // let a: f64 = 0.00145854145854146;
        // let b: f64 = 0.1993006993007;
        // let c: f64 = -0.72527472527431 - no_items as f64;
        // let delta = b.powi(2) - 4.0 * a * c;

        // let result = ((-b + delta.sqrt()) / (2.0 * a)).round() as u16;
        // return result;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                if state.download_langage.0 {
                    let percent = if state.download_langage.1.items.len() <= ITEM_IN_VIEW as usize {
                        compute_percent(state.download_langage.1.items.len())
                    } else {
                        65
                    };
                    Constraint::Percentage(percent)
                } else {
                    Constraint::Length(3)
                },
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(area);

    let header = Paragraph::new(
        state.search_result.items[state.download_modal.1]
            .0
            .name
            .to_owned(),
    )
    .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    if state.download_langage.0 {
        f.render_widget(
            dropdown(
                &state.download_langage.1,
                &InputMode::Langage,
                &state.terminal_size,
                Some(ITEM_IN_VIEW),
            ),
            chunks[1],
        );
    } else {
        let language = Paragraph::new(
            state.download_langage.1.items[state.download_langage.1.state]
                .0
                .to_owned(),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Kata Langage"),
        )
        .style(match state.download_modal.0 {
            DownloadModalInput::Langage => Style::default().fg(Color::LightYellow),
            _ => Style::default(),
        });
        f.render_widget(language, chunks[1]);
    }

    let path = state
        .download_path
        .basic_render(state.download_modal.0 == DownloadModalInput::Path)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Download Path"),
        )
        .style(match state.download_modal.0 {
            DownloadModalInput::Path => Style::default().fg(Color::LightYellow),
            _ => Style::default(),
        });
    f.render_widget(path, chunks[2]);

    let editor = state
        .editor_field
        .basic_render(state.download_modal.0 == DownloadModalInput::Editor)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Open with (terminal cmd)"),
        )
        .style(match state.download_modal.0 {
            DownloadModalInput::Editor => Style::default().fg(Color::LightYellow),
            _ => Style::default(),
        });
    f.render_widget(editor, chunks[3]);

    let submit = Paragraph::new("Download ✅")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(match state.download_modal.0 {
            DownloadModalInput::Submit => Style::default().fg(Color::LightGreen),
            _ => Style::default(),
        });
    f.render_widget(submit, chunks[4]);
}
