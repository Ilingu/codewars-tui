use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::{error::Error, fs::File, path::Path, process::Command};

use headless_chrome::Browser;

use reqwest::Url;
use scraper::element_ref::Text;
use tui::style::Color;

use rand::Rng;
use users::get_current_username;

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

pub fn trim_specials_chars(string: &str) -> String {
    let mut out = String::new();
    for ch in string.chars() {
        if ch.is_alphabetic() {
            out.push(ch);
        } else if ch == ' ' {
            out.push('-');
        }
    }
    return out;
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

pub fn ls_dir(path: &str) -> Result<Vec<String>, String> {
    if cfg!(target_os = "windows") {
        // let cmd_res = Command::new("dir").arg("/d").current_dir(path).output();
        return Err("not supported".to_string());
    }

    let cmd_res = Command::new("dir").current_dir(path).output();
    return match cmd_res {
        Ok(out) => {
            let out_str = String::from_utf8(out.stdout);
            match out_str {
                Ok(mut output) => {
                    output = output.trim().replace("\t", " ").replace("\n", " ");
                    Ok(output
                        .split(" ")
                        .filter(|x| !x.eq(&""))
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>())
                }
                Err(why) => Err(why.to_string()),
            }
        }
        Err(err) => Err(err.to_string()),
    };
}

pub fn get_uname() -> String {
    return get_current_username()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string();
}

pub fn log_print(log: String) {
    let uname = get_uname();

    let path_str = format!("/home/{uname}/.cache/codewars_cli");
    let path = Path::new(path_str.as_str());
    if let Err(_) = fs::create_dir_all(path) {
        return;
    }

    let log_file = format!("{path_str}/dev_logs.log");
    let log_file_path = Path::new(log_file.as_str());

    let mut file = match OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(log_file_path)
    {
        Ok(f) => f,
        Err(_) => return,
    };

    if let Err(_) = writeln!(file, "{log}") {}
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

// IO
pub fn write_file(path_str: String, value: String) -> Result<(), String> {
    let path = Path::new(&path_str);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => return Err(format!("couldn't create {}: {}", display, why)),
        Ok(file) => file,
    };

    // Write the `LOREM_IPSUM` string to `file`, returns `io::Result<()>`
    match file.write_all(value.as_bytes()) {
        Err(why) => return Err(format!("couldn't write to {}: {}", display, why)),
        Ok(_) => Ok(()),
    }
}

// Fetch codewars sample code & instruction for puzzles
pub async fn fetch_codewars_download_info(
    kata_id: &str,
    langage: Option<&str>,
) -> Result<(String, Vec<String>, Vec<String>), Box<dyn Error>> {
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

    let solution_field_elems = tab.wait_for_elements("#code div.CodeMirror-code > div > pre");
    let solution_field_lines = match solution_field_elems {
        Ok(lines) => lines
            .iter()
            .map(|line| line.get_inner_text().unwrap_or_default())
            .collect::<Vec<String>>(),
        Err(_) => return Err("failed to get the code sample".into()),
    };

    let tests_field_elems = tab.wait_for_elements("#fixture div.CodeMirror-code > div > pre");
    let tests_field_lines = match tests_field_elems {
        Ok(lines) => lines
            .iter()
            .map(|line| line.get_inner_text().unwrap_or_default())
            .collect::<Vec<String>>(),
        Err(_) => return Err("failed to get the code sample".into()),
    };

    Ok((instruction, solution_field_lines, tests_field_lines))
}

// yet a another utils func

pub fn language_to_extension(language: &str) -> Option<&str> {
    match language {
        "agda" => Some(".agda"),
        "bf" => Some(".bf"),
        "c" => Some(".c"),
        "cfml" => Some(".cfm"),
        "clojure" => Some(".clj"),
        "cobol" => Some(".cob"),
        "coffeescript" => Some(".coffee"),
        "commonlisp" => Some(".lisp"),
        "coq" => Some(".v"),
        "cpp" => Some(".cpp"),
        "crystal" => Some(".cr"),
        "csharp" => Some(".cs"),
        "d" => Some(".d"),
        "dart" => Some(".dart"),
        "elixir" => Some(".ex"),
        "elm" => Some(".elm"),
        "erlang" => Some(".erl"),
        "factor" => Some(".factor"),
        "forth" => Some(".forth"),
        "fortran" => Some(".f90"),
        "fsharp" => Some(".fs"),
        "go" => Some(".go"),
        "groovy" => Some(".groovy"),
        "haskell" => Some(".hs"),
        "haxe" => Some(".hx"),
        "idris" => Some(".idr"),
        "java" => Some(".java"),
        "javascript" => Some(".js"),
        "julia" => Some(".jl"),
        "kotlin" => Some(".kt"),
        "lambdacalc" => Some(".lc"),
        "lean" => Some(".lean"),
        "lua" => Some(".lua"),
        "nasm" => Some(".asm"),
        "nim" => Some(".nim"),
        "objc" => Some(".m"),
        "ocaml" => Some(".ml"),
        "pascal" => Some(".pas"),
        "perl" => Some(".pl"),
        "php" => Some(".php"),
        "powershell" => Some(".ps1"),
        "prolog" => Some(".pl"),
        "purescript" => Some(".purs"),
        "python" => Some(".py"),
        "r" => Some(".r"),
        "racket" => Some(".rkt"),
        "raku" => Some(".raku"),
        "reason" => Some(".re"),
        "riscv" => Some(".s"),
        "ruby" => Some(".rb"),
        "rust" => Some(".rs"),
        "scala" => Some(".scala"),
        "shell" => Some(".sh"),
        "solidity" => Some(".sol"),
        "sql" => Some(".sql"),
        "swift" => Some(".swift"),
        "typescript" => Some(".ts"),
        "vb" => Some(".vb"),
        _ => None,
    }
}
