use std::error::Error;

use reqwest::Url;
use scraper::element_ref::Text;
use tui::style::Color;

use rand::Rng;

/// generate a random integer between a and b included
pub fn rand_int(a: isize, b: isize) -> isize {
    let mut rng = rand::thread_rng();
    return rng.gen_range(a..=b);
}

pub fn gen_rand_colors() -> Color {
    Color::Rgb(
        rand_int(0, 255) as u8,
        rand_int(0, 255) as u8,
        rand_int(0, 255) as u8,
    )
}

fn is_valid_url(s: &str) -> bool {
    Url::parse(s).is_ok()
}

pub async fn fetch_html(url: String) -> Result<String, Box<dyn Error>> {
    if !is_valid_url(url.as_str()) {
        return Err("invalid url".into());
    }

    let resp = reqwest::get(url).await?.text().await?;
    Ok(resp)
}

// scraper::element_ref::Text hijack to add some methods
pub type ScrapperText<'a> = Text<'a>;
pub trait TextMethods {
    fn to_string(self) -> String;
}

impl TextMethods for ScrapperText<'_> {
    fn to_string(self) -> String {
        self.collect::<Vec<&str>>().join("")
    }
}

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
        if self.state == self.items.len() - 1 {
            self.state = 0
        } else {
            self.state += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.state == 0 {
            self.state = self.items.len() - 1
        } else {
            self.state -= 1;
        }
    }
}
