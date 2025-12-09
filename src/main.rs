use clap::{ArgAction, Parser};
use std::cmp::Ordering;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Parser, Debug)]
#[command(author, version, about = "A NuShell-inspired ls with color.")]
struct Cli {
    /// Path to list
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Include dotfiles (like ls -a)
    #[arg(short = 'a', long = "all", action = ArgAction::SetTrue, default_value_t = false)]
    include_hidden: bool,

    /// Long listing output (accepted for familiarity; same as default output)
    #[arg(short = 'l', long = "long", action = ArgAction::SetTrue, default_value_t = false)]
    _long: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EntryType {
    Dir,
    File,
}

#[derive(Debug)]
struct EntryRow {
    name_plain: String,
    name_colored: String,
    entry_type_plain: String,
    entry_type_colored: String,
    size_plain: String,
    size_colored: String,
    modified_plain: String,
    modified_colored: String,
    is_dir: bool,
}

#[derive(Clone, Copy)]
enum Align {
    Left,
    Right,
}

mod palette {
    pub const RESET: &str = "\x1b[0m";
    pub const BORDER: &str = "\x1b[38;5;99m";
    pub const HEADER: &str = "\x1b[38;5;82m";
    pub const INDEX: &str = "\x1b[38;5;51m";
    pub const TYPE: &str = "\x1b[38;5;78m";
    pub const SIZE: &str = "\x1b[38;5;45m";
    pub const MODIFIED: &str = "\x1b[38;5;114m";
    pub const DIR: &str = "\x1b[38;5;45m";
    pub const FILE: &str = "\x1b[38;5;252m";
    pub const EXEC: &str = "\x1b[38;5;197m";
    pub const DOTFILE: &str = "\x1b[38;5;179m";
    pub const WARN: &str = "\x1b[38;5;214m";

    pub fn paint(text: impl AsRef<str>, color: &str) -> String {
        format!("{}{}{}", color, text.as_ref(), RESET)
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("{} {}", palette::paint("error:", palette::WARN), err);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let path = cli.path;
    let entries = collect_entries(&path, cli.include_hidden)?;
    render_table(entries);
    Ok(())
}

fn collect_entries(path: &PathBuf, include_hidden: bool) -> Result<Vec<EntryRow>, String> {
    let mut rows = Vec::new();
    let dir_reader = fs::read_dir(path).map_err(|err| format!("cannot read {}: {err}", path.display()))?;

    for entry in dir_reader {
        let entry = entry.map_err(|err| format!("cannot read entry: {err}"))?;
        let name = entry.file_name().to_string_lossy().to_string();
        let is_hidden = name.starts_with('.');
        if !include_hidden && is_hidden {
            continue;
        }

        let file_type = entry
            .file_type()
            .map_err(|err| format!("cannot get type for {}: {err}", name))?;
        let metadata = entry
            .metadata()
            .map_err(|err| format!("cannot read metadata for {}: {err}", name))?;

        let entry_type = if file_type.is_dir() {
            EntryType::Dir
        } else {
            EntryType::File
        };
        let is_executable = is_executable(&metadata);

        let size = metadata.len();
        let modified_plain = metadata
            .modified()
            .ok()
            .and_then(|ts| Some(format_relative_time(ts)))
            .unwrap_or_else(|| "unknown".to_string());

        let name_colored = color_name(&name, entry_type, is_executable, is_hidden);
        let type_plain = match entry_type {
            EntryType::Dir => "dir".to_string(),
            EntryType::File => "file".to_string(),
        };

        rows.push(EntryRow {
            name_plain: name.clone(),
            name_colored,
            entry_type_plain: type_plain.clone(),
            entry_type_colored: palette::paint(type_plain, palette::TYPE),
            size_plain: format_size(size),
            size_colored: palette::paint(format_size(size), palette::SIZE),
            modified_colored: palette::paint(modified_plain.clone(), palette::MODIFIED),
            modified_plain,
            is_dir: entry_type == EntryType::Dir,
        });
    }

    rows.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.name_plain.to_lowercase().cmp(&b.name_plain.to_lowercase()),
        }
    });

    Ok(rows)
}

fn render_table(rows: Vec<EntryRow>) {
    let index_width = format!("{}", rows.len().saturating_sub(1)).len().max(1);
    let name_width = rows
        .iter()
        .map(|row| row.name_plain.len())
        .max()
        .unwrap_or(4)
        .max("name".len());
    let type_width = rows
        .iter()
        .map(|row| row.entry_type_plain.len())
        .max()
        .unwrap_or(4)
        .max("type".len());
    let size_width = rows
        .iter()
        .map(|row| row.size_plain.len())
        .max()
        .unwrap_or(4)
        .max("size".len());
    let modified_width = rows
        .iter()
        .map(|row| row.modified_plain.len())
        .max()
        .unwrap_or(8)
        .max("modified".len());

    let widths = [index_width, name_width, type_width, size_width, modified_width];

    println!("{}", horizontal_border(&widths, BorderKind::Top));
    let header_cells = vec![
        ("#".to_string(), palette::paint("#", palette::INDEX), Align::Right),
        (
            "name".to_string(),
            palette::paint("name", palette::HEADER),
            Align::Left,
        ),
        (
            "type".to_string(),
            palette::paint("type", palette::HEADER),
            Align::Left,
        ),
        (
            "size".to_string(),
            palette::paint("size", palette::HEADER),
            Align::Right,
        ),
        (
            "modified".to_string(),
            palette::paint("modified", palette::HEADER),
            Align::Left,
        ),
    ];
    println!("{}", render_row(&header_cells, &widths));
    println!("{}", horizontal_border(&widths, BorderKind::Middle));

    for (idx, row) in rows.iter().enumerate() {
        let idx_plain = idx.to_string();
        let idx_colored = palette::paint(idx_plain.clone(), palette::INDEX);
        let data_cells = vec![
            (idx_plain, idx_colored, Align::Right),
            (row.name_plain.clone(), row.name_colored.clone(), Align::Left),
            (
                row.entry_type_plain.clone(),
                row.entry_type_colored.clone(),
                Align::Left,
            ),
            (row.size_plain.clone(), row.size_colored.clone(), Align::Right),
            (
                row.modified_plain.clone(),
                row.modified_colored.clone(),
                Align::Left,
            ),
        ];
        println!(
            "{}",
            render_row(&data_cells, &widths)
        );
    }

    println!("{}", horizontal_border(&widths, BorderKind::Bottom));
}

enum BorderKind {
    Top,
    Middle,
    Bottom,
}

fn horizontal_border(widths: &[usize], kind: BorderKind) -> String {
    let (start, sep, end) = match kind {
        BorderKind::Top => ('┌', '┬', '┐'),
        BorderKind::Middle => ('├', '┼', '┤'),
        BorderKind::Bottom => ('└', '┴', '┘'),
    };

    let mut line = String::new();
    line.push(start);
    for (idx, width) in widths.iter().enumerate() {
        line.push_str(&"─".repeat(width + 2));
        if idx + 1 == widths.len() {
            line.push(end);
        } else {
            line.push(sep);
        }
    }
    palette::paint(line, palette::BORDER)
}

fn render_row(columns: &[(String, String, Align)], widths: &[usize]) -> String {
    let mut line = String::new();
    line.push_str(&palette::paint("│", palette::BORDER));
    for ((plain, colored, align), width) in columns.iter().zip(widths.iter()) {
        let padded = pad_cell(colored, plain, *width, *align);
        line.push(' ');
        line.push_str(&padded);
        line.push(' ');
        line.push_str(&palette::paint("│", palette::BORDER));
    }
    line
}

fn pad_cell(colored: &str, plain: &str, width: usize, align: Align) -> String {
    let pad = width.saturating_sub(plain.len());
    match align {
        Align::Left => format!("{colored}{}", " ".repeat(pad)),
        Align::Right => format!("{}{}", " ".repeat(pad), colored),
    }
}

fn format_size(size: u64) -> String {
    let digits: Vec<char> = size.to_string().chars().rev().collect();
    let mut formatted = String::new();
    for (idx, ch) in digits.iter().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(*ch);
    }
    formatted.chars().rev().collect()
}

fn format_relative_time(ts: SystemTime) -> String {
    let now = SystemTime::now();
    let (past, duration) = match now.duration_since(ts) {
        Ok(dur) => (true, dur),
        Err(err) => (false, err.duration()),
    };

    let secs = duration.as_secs();
    if secs < 5 {
        return "just now".to_string();
    }

    let (value, unit) = if secs < 60 {
        (secs, "second")
    } else if secs < 3_600 {
        (secs / 60, "minute")
    } else if secs < 86_400 {
        (secs / 3_600, "hour")
    } else if secs < 604_800 {
        (secs / 86_400, "day")
    } else {
        (secs / 604_800, "week")
    };

    let plural = if value == 1 { "" } else { "s" };
    if past {
        format!("{value} {unit}{plural} ago")
    } else {
        format!("in {value} {unit}{plural}")
    }
}

fn color_name(name: &str, entry_type: EntryType, is_executable: bool, is_hidden: bool) -> String {
    match entry_type {
        EntryType::Dir => palette::paint(name, palette::DIR),
        EntryType::File => {
            if is_hidden {
                palette::paint(name, palette::DOTFILE)
            } else if is_executable {
                palette::paint(name, palette::EXEC)
            } else if name.ends_with(".md") || name.ends_with(".toml") {
                palette::paint(name, palette::WARN)
            } else {
                palette::paint(name, palette::FILE)
            }
        }
    }
}

#[cfg(unix)]
fn is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_metadata: &fs::Metadata) -> bool {
    false
}
