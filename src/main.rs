use colored::*;
use git2::Repository;
use std::env;

const TERM_BG: Color = Color::Black;
const POWERLINE_FG: Color = Color::Black;
const POWERLINE_SEP: &str = "";

fn main() {
    // color detection can be unreliable (e.g. for PowerShell prompts) so just force it on
    colored::control::set_override(true);

    let exit_code = env::args().nth(1).map_or(
        // exit code argument is optional so treat its absence as success
        Some(0),
        |value| value.parse::<i32>().ok(),
    );

    // TODO handle invalid working directory?
    let full_path = env::current_dir().unwrap();
    let path_str = {
        let stripped = (|| full_path.strip_prefix(dirs::home_dir()?).ok())();
        let path = match stripped {
            Some(path) => format!("~/{}", path.display()),
            None => full_path.display().to_string(),
        };
        if cfg!(windows) {
            path.replace("\\", "/")
        } else {
            path
        }
        .trim_end_matches('/')
        .to_string()
    };

    let current_ref = (|| {
        let repo = Repository::discover(&full_path).ok()?;
        let head = repo.head().ok()?;
        head.shorthand().map(|shorthand| shorthand.to_string())
    })();

    let all_entries = [
        match exit_code {
            Some(0) => None,
            _ => Some(Entry {
                text: "✖".to_string(),
                fg: Color::Red,
                bg: Color::BrightWhite,
            }),
        },
        Some(Entry {
            text: path_str,
            fg: POWERLINE_FG,
            bg: Color::Blue,
        }),
        current_ref.map(|current_ref| Entry {
            text: format!(" {}", current_ref),
            fg: POWERLINE_FG,
            bg: Color::Yellow,
        }),
    ];
    let entries = all_entries
        .iter()
        .filter_map(|x| x.as_ref())
        .collect::<Vec<_>>();

    let mut prompt = String::from("\n");
    for (i, entry) in entries.iter().enumerate() {
        if i == 0 {
            prompt.push_str(&POWERLINE_SEP.color(TERM_BG).on_color(entry.bg).to_string())
        }
        let next_bg = entries.get(i + 1).map_or(TERM_BG, |next| next.bg);
        prompt.push_str(&entry.powerline(next_bg));
    }
    prompt.push_str(&format!("\n{} ", "▶".bright_black()));

    println!("{}", prompt);
}

struct Entry {
    text: String,
    fg: Color,
    bg: Color,
}

impl Entry {
    fn powerline(&self, next_bg: Color) -> String {
        let content = format!(" {} ", self.text).color(self.fg).on_color(self.bg);
        let separator = POWERLINE_SEP.color(self.bg).on_color(next_bg);
        format!("{}{}", content, separator)
    }
}
