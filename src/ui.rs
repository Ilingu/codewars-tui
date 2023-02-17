use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::{
    types::{CodewarsCLI, InputMode, KataPreview, DIFFICULTY, LANGAGE, SORT_BY, TAGS},
    utils::{gen_rand_colors, rank_color},
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
    draw_list_section(f, state, parent_chunk[1])
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
        .field_dropdown
        .1
        .items
        .iter()
        .map(|(content, i)| {
            let is_active = i == &state.field_dropdown.1.state;

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
    let items_ranges = if state.field_dropdown.1.state > items_in_view {
        (state.field_dropdown.1.state - items_in_view)..=state.field_dropdown.1.state
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
        f.render_widget(dropdown(state), chunks[1]);
        return;
    }

    let help = Paragraph::new(APP_KEYS_DESC);
    f.render_widget(help, chunks[1]);

    let search = Paragraph::new(Spans::from(vec![
        Span::raw(state.search_field.to_owned()),
        match state.input_mode {
            InputMode::Search => Span::styled(
                "|",
                Style::default()
                    .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK)
                    .fg(Color::White),
            ),
            _ => Span::from(""),
        },
    ]))
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
    let items_ranges = if state.search_result.state > ITEMS_IN_VIEW_REF {
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

fn draw_kata(kata: &KataPreview, is_active: bool) -> Paragraph<'static> {
    // const BG: tui::style::Color = Color::Rgb(89, 48, 66);
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
            Span::raw(kata.total_completed.to_string()),
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
            Span::raw(kata.author.to_owned()),
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
                        kata.rank.to_owned(),
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(rank_color(kata.rank.as_str(), Color::White)),
                    ),
                ]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(if is_active {
                    Style::default().fg(rank_color(kata.rank.as_str(), Color::LightGreen))
                } else {
                    Style::default().fg(Color::DarkGray)
                }),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });
}
