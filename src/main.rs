pub mod app;
pub mod types;
pub mod ui;
pub mod utils;

use app::run_app;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use tui::{backend::CrosstermBackend, Terminal};
use types::CodewarsCLI;

/* How it'll work
- when opening it'll fetch from "https://www.codewars.com/kata/search" for the default kata
- parser for html to struct
- UI: on the left some settings for the search (search, sort by, langage, status, progress...) on update re fetch the kata
- rendering all the kata as a list on the right (90% of the screen)
- when user clicks on a kata in the list, close the setting panel and open a detailled view of the kata with a [download] button at the end
- when user clicks on the [download] button, fetch the kata instruction, sample tests, and sample solution at (https://www.codewars.com/kata/<kata-id>/train/<langage>) and then dwonload it to the user specified folder                                                                                                                  //
 */

const TERMINAL_REF_SIZE: (u16, u16) = (147, 34);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut state = CodewarsCLI::new();
    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut state).await;

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
