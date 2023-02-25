use serde::Deserialize;

use crate::ui::StatefulList;

pub enum InputMode {
    Normal,
    Search,
    SortBy,
    Langage,
    Difficulty,
    Tags,
    KataList,
}

#[derive(PartialEq)]
pub enum DownloadModalInput {
    Disabled,
    Langage,
    Path,
    Submit,
}

// for endpoint: &r%5B%5D=-8&r%5B%5D=-6 (decoded: "&r[]=-8&r[]=-6", here for kyu 8 and 6) // thus it's just the "state.difficulty_field"
pub const DIFFICULTY: [&str; 9] = [
    "Select Ranks", // do nothing
    "1 kyu",
    "2 kyu",
    "3 kyu",
    "4 kyu",
    "5 kyu",
    "6 kyu",
    "7 kyu",
    "8 kyu",
];

// for endpoint: &order_by=popularity%20desc OR &order_by=popularity%20asc ...
pub const SORT_BY: [&str; 11] = [
    "Newest",             // default, put nothing
    "Oldest",             // published_at;asc
    "Popularity",         // popularity;desc
    "Positive Feedback",  // satisfaction_percent;desc
    "Most Completed",     //total_completed;desc
    "Least Completed",    //total_completed;asc
    "Recently Published", // published_at;desc
    "Hardest",            //rank_id;desc
    "Easiest",            // rank_id;asc
    "Name",               // name;asc
    "Low Satisfaction",   // satisfaction_percent;asc
];

// for endpoint: "/kata/search/<langage>?q=...", most are just the same as the one below in lower case, some are more complex: C++ is cpp, Objective-C is objc ...
pub const LANGAGE: [&str; 60] = [
    "All", // do nothing
    "My Languages",
    "Agda",
    "BF",
    "C",
    "CFML",
    "Clojure",
    "COBOL",
    "CoffeeScript",
    "CommonLisp",
    "Coq",
    "C++",
    "Crystal",
    "C#",
    "D",
    "Dart",
    "Elixir",
    "Elm",
    "Erlang",
    "Factor",
    "Forth",
    "Fortran",
    "F#",
    "Go",
    "Groovy",
    "Haskell",
    "Haxe",
    "Idris",
    "Java",
    "JavaScript",
    "Julia",
    "Kotlin",
    "λ Calculus",
    "Lean",
    "Lua",
    "NASM",
    "Nim",
    "Objective-C",
    "OCaml",
    "Pascal",
    "Perl",
    "PHP",
    "PowerShell",
    "Prolog",
    "PureScript",
    "Python",
    "R",
    "Racket",
    "Raku",
    "Reason",
    "RISC-V",
    "Ruby",
    "Rust",
    "Scala",
    "Shell",
    "Solidity",
    "SQL",
    "Swift",
    "TypeScript",
    "VB",
];

// for url endpoint: &tags=Binary%20Search%20Trees%2CAlgorithms (for exemple, PS: "%2C" is ",")
pub const TAGS: [&str; 109] = [
    "Select Tags", // do nothing
    "ASCII Art",
    "Algebra",
    "Algorithms",
    "Angular",
    "Arrays",
    "Artificial Intelligence",
    "Asynchronous",
    "Backend",
    "Big Integers",
    "Binary",
    "Binary Search Trees",
    "Binary Trees",
    "Bits",
    "Cellular Automata",
    "Ciphers",
    "Combinatorics",
    "Compilers",
    "Concurrency",
    "Cryptography",
    "Data Frames",
    "Data Science",
    "Data Structures",
    "Databases",
    "Date Time",
    "Debugging",
    "Decorator",
    "Design Patterns",
    "Discrete Mathematics",
    "Domain Specific Languages",
    "Dynamic Programming",
    "Esoteric Languages",
    "Event Handling",
    "Express",
    "Filtering",
    "Flask",
    "Functional Programming",
    "Fundamentals",
    "Game Solvers",
    "Games",
    "Genetic Algorithms",
    "Geometry",
    "Graph Theory",
    "Graphics",
    "Graphs",
    "Heaps",
    "Image Processing",
    "Interpreters",
    "Iterators",
    "JSON",
    "Language Features",
    "Linear Algebra",
    "Linked Lists",
    "Lists",
    "Logic",
    "Logic Programming",
    "Machine Learning",
    "Macros",
    "Mathematics",
    "Matrix",
    "Memoization",
    "Metaprogramming",
    "Monads",
    "MongoDB",
    "Networks",
    "Neural Networks",
    "NumPy",
    "Number Theory",
    "Object-oriented Programming",
    "Parsing",
    "Performance",
    "Permutations",
    "Physics",
    "Priority Queues",
    "Probability",
    "Promises",
    "Puzzles",
    "Queues",
    "React",
    "Reactive Programming",
    "Recursion",
    "Refactoring",
    "Reflection",
    "Regular Expressions",
    "Restricted",
    "Reverse Engineering",
    "Riddles",
    "RxJS",
    "SQL",
    "Scheduling",
    "Searching",
    "Security",
    "Set Theory",
    "Sets",
    "Simulation",
    "Singleton",
    "Sorting",
    "Stacks",
    "State Machines",
    "Statistics",
    "Streams",
    "Strings",
    "Theorem Proving",
    "Threads",
    "Trees",
    "Tutorials",
    "Unicode",
    "Web Scraping",
    "Web3",
];

// Full katas implementation coming from API (for detailled view, if I have the will to program this non-essential part)
pub struct Kata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub languages: Vec<String>,
    pub tags: Vec<String>,
    pub category: String,
    pub desc: String,
    pub rank: String,
    pub author: String,
    pub published_at: String,
    pub total_attempts: usize,
    pub total_completed: usize,
    pub total_stars: usize,
    pub created_at: String,
}

// Minified katas from search result (https://www.codewars.com/kata/search)
pub struct KataPreview {
    pub id: String,
    pub name: String,
    pub url: String,
    pub tags: Vec<String>,
    pub languages: Vec<String>,
    pub author: String,
    pub total_completed: usize,
    pub rank: String,
}

pub struct CodewarsCLI {
    // client/framework state
    pub terminal_size: (u16, u16),
    // app state
    pub input_mode: InputMode,
    pub search_result: StatefulList<(KataPreview, usize)>,
    pub field_dropdown: (bool, StatefulList<(String, usize)>),

    pub download_modal: (DownloadModalInput, usize),
    pub download_path: (String, StatefulList<String>), // (value; autocompletions suggestions)
    pub download_langage: (bool, StatefulList<(String, usize)>),
    pub download_error_field: Vec<String>,
    // fields state
    pub search_field: String,
    pub sortby_field: usize,
    pub langage_field: usize,
    pub difficulty_field: usize,
    pub tag_field: usize,
}

#[derive(Deserialize)]
pub struct KataAPI {
    pub id: String,          // ID of the kata.
    pub name: String,        // Name of the kata.
    pub slug: String,        // Slug of the kata.
    pub url: String,         // URL of the kata.
    pub category: String,    // Category of the kata.
    pub description: String, // Description of the kata in Markdown.
    pub tags: Vec<String>,   // Array of tags associated with the kata.
    pub languages: Vec<String>, // Array of language names the kata is available in.
                             // this struct is imcomplete, see https://dev.codewars.com/#get-code-challenge
}
