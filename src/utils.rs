use std::{collections::HashMap, error::Error, process::Command};

use headless_chrome::Browser;

use reqwest::Url;
use scraper::element_ref::Text;
use tui::style::Color;

use rand::Rng;

use crate::types::KataAPI;

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

pub fn rank_color(rank: &str, default: Color) -> Color {
    match rank {
        "1 kyu" | "2 kyu" => Color::Rgb(134, 108, 199),
        "3 kyu" | "4 kyu" => Color::Rgb(60, 126, 187),
        "5 kyu" | "6 kyu" => Color::Rgb(236, 182, 19),
        "8 kyu" | "7 kyu" => Color::Rgb(230, 230, 230),
        _ => default,
    }
}

pub fn open_url(url: &str) -> Result<(), String> {
    let cmd_res = if cfg!(target_os = "windows") {
        Command::new("start").arg(url).output()
    } else {
        Command::new("xdg-open").arg(url).output()
    };

    return match cmd_res {
        Ok(_) => Ok(()),
        Err(err) => Err(err.to_string()),
    };
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

// Fetch codewars sample code & instruction for puzzles
pub async fn fetch_codewars_download_info(
    kata_id: &str,
    langage: Option<&str>,
) -> Result<(String, Vec<String>), Box<dyn Error>> {
    // get instruction
    let resp = reqwest::get(format!(
        "https://www.codewars.com/api/v1/code-challenges/{}",
        kata_id
    ))
    .await?
    .json::<KataAPI>()
    .await?;

    let instruction = resp.description; // instruction in markdown

    // get sample code
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;
    tab.navigate_to(&format!(
        "https://www.codewars.com/kata/{}/train{}",
        kata_id,
        match langage {
            Some(l) => "/".to_string() + l,
            None => String::new(),
        }
    ))?;

    let solution_field_elems = tab.wait_for_elements("#code > div.text-editor.js-editor.has-shadow > div.CodeMirror.cm-s-codewars > div.CodeMirror-scroll > div.CodeMirror-sizer > div > div > div > div.CodeMirror-code > div > pre");
    let solution_field_lines = match solution_field_elems {
        Ok(lines) => lines
            .iter()
            .map(|line| line.get_inner_text().unwrap_or_default())
            .collect::<Vec<String>>(),
        Err(_) => return Err("failed to get solution boilerplate".into()),
    };

    Ok((instruction, solution_field_lines))
}
