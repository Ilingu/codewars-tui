use std::{fs, process::Command};

use crossterm::{
    event::{self, Event, KeyCode, MouseEventKind},
    terminal::size,
};
use scraper::{Html, Selector};
use tui::{backend::Backend, Terminal};
use urlencoding::encode;
use users::get_current_username;

use crate::{
    types::{
        CodewarsCLI, DownloadModalInput, InputMode, KataPreview, DIFFICULTY, LANGAGE, SORT_BY, TAGS,
    },
    ui::{ui, StatefulList},
    utils::{
        fetch_codewars_download_info, fetch_html, language_to_extension, ls_dir, open_url,
        trim_specials_chars, write_file, TextMethods,
    },
    TERMINAL_REF_SIZE,
};

const CODEWARS_ENDPOINT: &str = "https://www.codewars.com/kata/search";

impl CodewarsCLI {
    pub fn new() -> CodewarsCLI {
        CodewarsCLI {
            input_mode: InputMode::Normal,
            terminal_size: (0, 0),
            field_dropdown: (false, StatefulList::with_items(vec![], 0)),
            download_modal: (DownloadModalInput::Disabled, 0),
            download_path: (String::new(), StatefulList::with_items(vec![], 0)),
            download_langage: (false, StatefulList::with_items(vec![], 0)),
            download_error_field: vec![],
            search_result: StatefulList::with_items(vec![], 0),
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
        .map(|(i, d)| (d.to_string(), i))
        .collect::<Vec<(String, usize)>>();

        self.field_dropdown = (true, StatefulList::with_items(datas, selected));
    }

    pub fn hide_dropdown(&mut self) {
        self.field_dropdown = (false, StatefulList::with_items(vec![], 0))
    }

    pub async fn submit_search(&mut self) {
        let url = self.build_url();
        let resp = fetch_html(url).await;

        if let Ok(html_doc) = resp {
            let document = Html::parse_document(html_doc.as_str());

            let kata_selector = Selector::parse("main .list-item-kata").unwrap();
            let tags_selector = Selector::parse(".keyword-tag").unwrap();
            let languages_selector = Selector::parse("div div:nth-child(2) li a").unwrap();
            let author_selector =
                Selector::parse("a[data-tippy-content=\"This kata's Sensei\"]").unwrap();
            let total_completed_selector = Selector::parse(
                "span[data-tippy-content=\"Total times this kata has been completed\"]",
            )
            .unwrap();
            let rank_selector = Selector::parse("span").unwrap(); // only the first item

            let mut katas: Vec<(KataPreview, usize)> = vec![];
            for (i, element) in document.select(&kata_selector).enumerate() {
                let mut kata = KataPreview::default();

                kata.id = element.value().id().unwrap_or_default().to_string();
                kata.url = format!("https://www.codewars.com/kata/{}", kata.id);
                kata.name = element
                    .value()
                    .attr("data-title")
                    .unwrap_or_default()
                    .to_string();

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
                    Some(elem) => elem
                        .text()
                        .to_string()
                        .replace(",", "")
                        .parse::<usize>()
                        .unwrap_or_default(),
                    None => 0,
                };

                kata.rank = match element.select(&rank_selector).next() {
                    Some(elem) => elem.text().to_string(),
                    None => String::new(),
                };

                katas.push((kata, i));
            }

            self.search_result = StatefulList::with_items(katas, 0);
            self.change_state(InputMode::KataList);
        }
    }

    pub fn run_preinstall(language: &str, path: &str) -> Result<String, String> {
        match language {
            "rust" => {
                let cmd_res = Command::new("cargo").arg("init").current_dir(path).spawn();
                match cmd_res {
                    Ok(_) => Ok("src/".to_string()),
                    Err(err) => Err(err.to_string()),
                }
            }
            _ => Err("this language doesn't exist".to_string()),
        }
    }

    pub fn run_postinstall(path: &str) -> Result<(), String> {
        match Command::new("codium").arg(path).spawn() {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn autocomplete_path(&mut self) {
        let parts = self.download_path.0.split("/").collect::<Vec<&str>>();
        let parent_dir = parts[0..parts.len() - 1].join("/");
        if let Ok(child_dirs) = ls_dir(&parent_dir) {
            let usearch = match parts.last() {
                Some(data) => data.to_lowercase().trim().to_string(),
                None => return,
            };

            let match_dirs = child_dirs
                .iter()
                .filter(|d| **d == usearch)
                .map(|md| md.to_owned())
                .collect::<Vec<String>>();

            self.download_path.1 = StatefulList::with_items(match_dirs, 0);
        } else {
            self.download_error_field
                .push("Invalid directory".to_string());
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

impl KataPreview {
    pub fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            url: String::new(),
            tags: vec![],
            languages: vec![],
            author: String::new(),
            total_completed: 0,
            rank: String::new(),
        }
    }

    pub async fn download(&self, language: &str, mut udownload_path: &str) -> Result<(), String> {
        let (instruction, sample_code_lines, sample_tests_lines) =
            match fetch_codewars_download_info(self.id.as_str(), Some(language)).await {
                Ok(data) => data,
                Err(err) => {
                    return Err(err.to_string());
                }
            };

        udownload_path = udownload_path.trim_end_matches("/");
        let download_path = format!(
            "{udownload_path}/{}",
            trim_specials_chars(self.name.to_lowercase().trim())
        );

        if let Err(why) = fs::create_dir_all(&download_path) {
            return Err(why.to_string());
        }

        let preinstall = match CodewarsCLI::run_preinstall(language, download_path.as_str()) {
            Ok(path) => path,
            Err(_) => String::new(),
        };

        let language_ext = language_to_extension(language).unwrap_or_default();
        let code_filename = format!("{download_path}/{}solution{}", preinstall, language_ext);
        let tests_filename = format!("{download_path}/{}tests{}", preinstall, language_ext);
        let instruction_filename = format!("{download_path}/instruction.md");

        if let Err(why) = write_file(code_filename, sample_code_lines.join("\n")) {
            return Err(why.to_string());
        }
        if let Err(why) = write_file(instruction_filename, instruction) {
            return Err(why.to_string());
        }
        if let Err(why) = write_file(tests_filename, sample_tests_lines.join("\n")) {
            return Err(why.to_string());
        }

        if let Err(_) = CodewarsCLI::run_postinstall(download_path.as_str()) {}

        Ok(())
    }
}

pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut CodewarsCLI,
) -> Result<(), std::io::Error> {
    let mut first_loop = true;
    state.terminal_size = size()?;

    loop {
        terminal.draw(|f| ui(f, state))?;

        if first_loop {
            state.submit_search().await;
            first_loop = false
        }

        match event::read()? {
            Event::Resize(w, h) => state.terminal_size = (w, h),
            Event::Paste(data) => {
                match state.download_modal.0 {
                    DownloadModalInput::Path => {
                        state.download_path.0.push_str(data.as_str());
                    }
                    _ => {}
                }
                match state.input_mode {
                    InputMode::Search => {
                        state.search_field.push_str(data.as_str());
                    }
                    _ => {}
                };
            }
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
                if state.field_dropdown.0 {
                    match key.code {
                        KeyCode::Up => state.field_dropdown.1.previous(),
                        KeyCode::Down => state.field_dropdown.1.next(),
                        KeyCode::Enter => {
                            match state.input_mode {
                                InputMode::SortBy => {
                                    state.sortby_field = state.field_dropdown.1.state
                                }
                                InputMode::Langage => {
                                    state.langage_field = state.field_dropdown.1.state
                                }
                                InputMode::Difficulty => {
                                    state.difficulty_field = state.field_dropdown.1.state
                                }
                                InputMode::Tags => state.tag_field = state.field_dropdown.1.state,
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
                            KeyCode::Char('S') | KeyCode::Char('s') => state.submit_search().await,
                            KeyCode::Char('L') | KeyCode::Char('l') => {
                                state.change_state(InputMode::KataList)
                            }
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

                        InputMode::KataList => match state.download_modal.0 {
                            DownloadModalInput::Disabled => match key.code {
                                KeyCode::Tab | KeyCode::Down => {
                                    if state.search_result.items.len() > 0 {
                                        state.search_result.next();
                                    }
                                }
                                KeyCode::BackTab | KeyCode::Up => {
                                    if state.search_result.items.len() > 0 {
                                        state.search_result.previous();
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Err(_) = open_url(
                                        &state.search_result.items[state.search_result.state].0.url,
                                    ) {}
                                }
                                KeyCode::Char('D') | KeyCode::Char('d') => {
                                    if state.download_path.0 == String::new() {
                                        let uname = get_current_username()
                                            .unwrap_or_default()
                                            .to_str()
                                            .unwrap_or_default()
                                            .to_string();
                                        state.download_path.0 = format!("/home/{uname}/");
                                    }

                                    state.download_langage = (
                                        false,
                                        StatefulList::with_items(
                                            state.search_result.items[state.search_result.state]
                                                .0
                                                .languages
                                                .iter()
                                                .enumerate()
                                                .map(|(i, s)| (s.to_owned(), i))
                                                .collect::<Vec<(String, usize)>>(),
                                            0,
                                        ),
                                    );
                                    state.download_modal =
                                        (DownloadModalInput::Langage, state.search_result.state);
                                }
                                KeyCode::Esc => state.change_state(InputMode::Normal),
                                _ => {}
                            },
                            DownloadModalInput::Langage => {
                                if state.download_langage.0 {
                                    match key.code {
                                        KeyCode::Tab | KeyCode::Down => {
                                            state.download_langage.1.next()
                                        }
                                        KeyCode::BackTab | KeyCode::Up => {
                                            state.download_langage.1.previous()
                                        }
                                        KeyCode::Enter | KeyCode::Esc => {
                                            state.download_langage.0 = false
                                        }
                                        _ => {}
                                    }
                                } else {
                                    match key.code {
                                        KeyCode::Tab | KeyCode::Down => {
                                            state.download_modal.0 = DownloadModalInput::Path
                                        }
                                        KeyCode::Enter => state.download_langage.0 = true,
                                        KeyCode::Esc => {
                                            state.download_modal.0 = DownloadModalInput::Disabled
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            DownloadModalInput::Path => match key.code {
                                KeyCode::Char(c) => {
                                    state.download_path.0.push(c);
                                    state.autocomplete_path();
                                }
                                KeyCode::Backspace => {
                                    if state.download_path.0.split("/").count() != 4
                                        || state.download_path.0.chars().last().unwrap_or_default()
                                            != '/'
                                    {
                                        state.download_path.0.pop();
                                        state.autocomplete_path();
                                    }
                                }
                                KeyCode::Tab | KeyCode::Down => {
                                    state.download_modal.0 = DownloadModalInput::Submit
                                }
                                KeyCode::BackTab | KeyCode::Up => {
                                    state.download_modal.0 = DownloadModalInput::Langage
                                }
                                KeyCode::Esc => {
                                    state.download_modal.0 = DownloadModalInput::Disabled
                                }
                                _ => {}
                            },
                            DownloadModalInput::Submit => match key.code {
                                KeyCode::BackTab | KeyCode::Up => {
                                    state.download_modal.0 = DownloadModalInput::Path
                                }
                                KeyCode::Enter => {
                                    let kata_to_download =
                                        &state.search_result.items[state.download_modal.1].0;

                                    let download_result = kata_to_download
                                        .download(
                                            state.download_langage.1.items
                                                [state.download_langage.1.state]
                                                .0
                                                .as_str(),
                                            state.download_path.0.as_str(),
                                        )
                                        .await;
                                    match download_result {
                                        Ok(_) => {
                                            state.download_modal =
                                                (DownloadModalInput::Disabled, 0);
                                            state.download_langage =
                                                (false, StatefulList::with_items(vec![], 0))

                                            // TODO: ok message to user
                                        }
                                        Err(_) => {
                                            // TODO: err message to user
                                        }
                                    };
                                }
                                KeyCode::Esc => {
                                    state.download_modal.0 = DownloadModalInput::Disabled
                                }
                                _ => {}
                            },
                        },
                    }
                }
            }
            _ => {}
        }
    }
}
