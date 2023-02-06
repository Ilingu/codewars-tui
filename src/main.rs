pub mod types;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, str::FromStr};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders, Widget},
    Frame, Terminal,
};
use types::{CodewarsCLI, Difficulty, InputMode, Langage, SortBy, Tags};

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
            mode: InputMode::Normal,
            search_result: vec![],
            search_field: String::new(),
            sortby_field: SortBy::Newest,
            langage_field: Langage::All,
            difficulty_field: Difficulty::Empty,
            tag_field: Tags::Empty,
        }
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
            match state.mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
                InputMode::Search => todo!(),
                InputMode::SortBy => todo!(),
                InputMode::Langage => todo!(),
                InputMode::Difficulty => todo!(),
                InputMode::Tags => todo!(),
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, state: &mut CodewarsCLI) {
    let parent_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(f.size());

    let new_section_block = Block::default()
        .title("Search Katas")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(new_section_block, parent_chunk[0]);
    // new_section(f, state, parent_chunk[0]);

    let list_section_block = Block::default()
        .title("List of kata")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(list_section_block, parent_chunk[1]);
    // list_section(f, state, parent_chunk[1])
}
