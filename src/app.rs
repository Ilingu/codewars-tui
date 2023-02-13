use crossterm::{
    event::{self, Event, KeyCode, MouseEventKind},
    terminal::size,
};
use scraper::{Html, Selector};
use tui::{backend::Backend, Terminal};
use urlencoding::encode;

use crate::{
    types::{CodewarsCLI, InputMode, KataPreview, DIFFICULTY, LANGAGE, SORT_BY, TAGS},
    ui::ui,
    utils::{fetch_html, StatefulList, TextMethods},
    TERMINAL_REF_SIZE,
};

const CODEWARS_ENDPOINT: &str = "https://www.codewars.com/kata/search";

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

    pub async fn submit_search(&mut self) {
        let url = self.build_url();
        let resp = fetch_html(url).await;

        if let Ok(html_doc) = resp {
            let document = Html::parse_document(html_doc.as_str());

            let kata_selector = Selector::parse("main .list-item-kata").unwrap();
            let name_selector = Selector::parse("a").unwrap(); // only the first item
            let tags_selector = Selector::parse(".keyword-tag").unwrap();
            let languages_selector = Selector::parse("div > div:nth-child(2) li > a").unwrap();
            let author_selector = Selector::parse("div > div:nth-child(1) a:nth-child(5)").unwrap();
            let total_completed_selector =
                Selector::parse("div > div:nth-child(1) span:nth-child(4)").unwrap();
            let rank_selector = Selector::parse("div > div:nth-child(1) span").unwrap(); // only the first item
            let stars_selector =
                Selector::parse("div > div:nth-child(1) span:nth-child(1) > a:nth-child(2)")
                    .unwrap();
            let satisfaction_selector =
                Selector::parse("div > div:nth-child(1) span:nth-child(3)").unwrap();

            let mut katas: Vec<KataPreview> = vec![];
            for element in document.select(&kata_selector) {
                let mut kata = KataPreview::default();

                kata.id = element.value().id().unwrap_or_default().to_string();
                kata.url = format!("https://www.codewars.com/kata/{}", kata.id);
                kata.name = match element.select(&name_selector).next() {
                    Some(elem) => elem.text().to_string(),
                    None => String::new(),
                };

                for tag_elem in element.select(&tags_selector) {
                    kata.tags.push(tag_elem.text().to_string());
                }

                for language_elem in element.select(&languages_selector) {
                    kata.languages.push(
                        language_elem
                            .value()
                            .attr("data-language")
                            .unwrap_or_default()
                            .to_string(),
                    )
                }

                kata.author = match element.select(&author_selector).next() {
                    Some(elem) => elem.text().to_string(),
                    None => String::new(),
                };

                kata.total_completed = match element.select(&total_completed_selector).next() {
                    Some(elem) => elem.text().to_string().parse::<usize>().unwrap_or_default(),
                    None => 0,
                };

                kata.rank = match element.select(&rank_selector).next() {
                    Some(elem) => elem.text().to_string(),
                    None => String::new(),
                };

                kata.total_completed = match element.select(&stars_selector).next() {
                    Some(elem) => elem.text().to_string().parse::<usize>().unwrap_or_default(),
                    None => 0,
                };

                kata.satisfaction = match element.select(&satisfaction_selector).next() {
                    Some(elem) => elem
                        .text()
                        .to_string()
                        .split(" of ")
                        .nth(0)
                        .unwrap_or_default()
                        .to_string(),
                    None => String::new(),
                };

                katas.push(kata);
            }

            self.search_result = katas;
        }
    }

    fn build_url(&self) -> String {
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

        return format!("{CODEWARS_ENDPOINT}/{language}{query}{sortby}{difficulty}{tags}");
    }
}

pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut CodewarsCLI<'_>,
) -> Result<(), std::io::Error> {
    state.terminal_size = size()?;
    loop {
        terminal.draw(|f| ui(f, state))?;

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
                            state.submit_search().await;
                        }
                        KeyCode::Esc => state.hide_dropdown(),
                        _ => {}
                    }
                } else {
                    match state.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('S') => state.submit_search().await,
                            KeyCode::Tab => state.change_state(InputMode::Search),
                            _ => {}
                        },

                        InputMode::Search => match key.code {
                            KeyCode::Char(c) => state.search_field.push(c),
                            KeyCode::Enter => state.submit_search().await,
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
