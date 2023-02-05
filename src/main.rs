use std::io;
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Widget},
    Terminal,
};

/* How it'll work
- when opening it'll fetch from "https://www.codewars.com/kata/search" for the default kata
- parser for html to struct
- UI: on the left some settings for the search (search, sort by, langage, status, progress...) on update re fetch the kata
- rendering all the kata as a list on the right (90% of the screen)
- when user clicks on a kata in the list, close the setting panel and open a detailled view of the kata with a [download] button at the end
- when user clicks on the [download] button, fetch the kata instruction, sample tests, and sample solution at (https://www.codewars.com/kata/<kata-id>/train/<langage>) and then dwonload it to the user specified folder                                                                                                                  //
 */

fn main() -> Result<(), io::Error> {
    // setup terminal
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default().title("Block").borders(Borders::ALL);
        f.render_widget(block, size);
    })?;

    Ok(())
}
