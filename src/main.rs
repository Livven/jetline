use colored::*;
use git2::{Repository, StatusOptions};
use itertools::Itertools;
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

    // TODO show branch name (i.e. "master") when there are no commits yet
    let git_info = (|| {
        let repo = Repository::discover(&full_path).ok()?;
        let head = repo.head().ok()?;
        let branch_name = if head.is_branch() {
            Some(head.shorthand()?.to_string())
        } else {
            None
        };
        let commit_sha = head.target()?;

        let tracking = (|| {
            let upstream_name = repo.branch_upstream_name(head.name()?).ok()?;
            let upstream = repo.find_reference(upstream_name.as_str()?).ok()?;
            let (ahead, behind) = repo
                .graph_ahead_behind(commit_sha, upstream.target()?)
                .ok()?;
            Some(Tracking { ahead, behind })
        })();

        // TODO distinguish between staged and unstaged changes
        let changes = repo
            .statuses(Some(
                &mut StatusOptions::new()
                    .include_untracked(true)
                    .recurse_untracked_dirs(true),
            ))
            .ok()?
            .len();

        Some(GitInfo {
            sha: commit_sha.to_string(),
            branch: branch_name.map(|name| Branch { name, tracking }),
            changes,
        })
    })();

    let entries = vec![
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
        git_info.map(|git_info| Entry {
            text: {
                vec![
                    Some("".to_string()),
                    Some(match &git_info.branch {
                        Some(branch) => branch.name.to_string(),
                        None => git_info.sha[..7].to_string(),
                    }),
                    git_info.branch.map(|branch| match branch.tracking {
                        Some(Tracking {
                            ahead: 0,
                            behind: 0,
                        }) => "≣".to_string(),
                        Some(tracking) => [("↑", tracking.ahead), ("↓", tracking.behind)]
                            .iter()
                            .filter(|(_, count)| count != &0usize)
                            .map(|(symbol, count)| format!("{}{}", symbol, count))
                            .join(" "),
                        None => "≢".to_string(),
                    }),
                    match git_info.changes {
                        0 => None,
                        changes => Some(format!("±{}", changes)),
                    },
                ]
                .into_iter()
                .filter_map(|x| x)
                .join(" ")
            },
            fg: POWERLINE_FG,
            bg: match git_info.changes {
                0 => Color::Green,
                _ => Color::Yellow,
            },
        }),
    ]
    .into_iter()
    .filter_map(|x| x)
    .collect_vec();

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

struct GitInfo {
    sha: String,
    branch: Option<Branch>,
    changes: usize,
}

struct Branch {
    name: String,
    tracking: Option<Tracking>,
}

struct Tracking {
    ahead: usize,
    behind: usize,
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
