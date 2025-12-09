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
    pub const MODIFIED_RECENT: &str = "\x1b[38;5;82m";
    pub const MODIFIED_SOON: &str = "\x1b[38;5;148m";
    pub const MODIFIED_HOURS: &str = "\x1b[38;5;184m";
    pub const MODIFIED_DAYS: &str = "\x1b[38;5;208m";
    pub const MODIFIED_WEEKS: &str = "\x1b[38;5;203m";
    pub const MODIFIED_OLD: &str = "\x1b[38;5;244m";
    pub const MODIFIED_FUTURE: &str = "\x1b[38;5;111m";
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
        let (modified_plain, recency) = metadata
            .modified()
            .ok()
            .map(format_relative_time)
            .unwrap_or_else(|| ("unknown".to_string(), Recency::Unknown));

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
            modified_colored: color_modified(&modified_plain, recency),
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
    const UNITS: &[(&str, u64)] = &[
        ("B", 1),
        ("KB", 1024),
        ("MB", 1024 * 1024),
        ("GB", 1024 * 1024 * 1024),
        ("TB", 1024 * 1024 * 1024 * 1024),
    ];

    let mut unit = UNITS[0];
    for candidate in UNITS {
        if size >= candidate.1 {
            unit = *candidate;
        } else {
            break;
        }
    }

    let value = size as f64 / unit.1 as f64;
    let text = if value < 10.0 && unit.0 != "B" {
        format!("{value:.1}")
    } else {
        format!("{value:.0}")
    };

    format!("{text} {}", unit.0)
}

fn format_relative_time(ts: SystemTime) -> (String, Recency) {
    let now = SystemTime::now();
    let (past, duration) = match now.duration_since(ts) {
        Ok(dur) => (true, dur),
        Err(err) => (false, err.duration()),
    };

    let secs = duration.as_secs();
    let recency = if !past {
        Recency::Future
    } else if secs < 5 {
        Recency::JustNow
    } else if secs < 60 {
        Recency::Seconds
    } else if secs < 3_600 {
        Recency::Minutes
    } else if secs < 86_400 {
        Recency::Hours
    } else if secs < 604_800 {
        Recency::Days
    } else if secs < 2_629_746 {
        Recency::Weeks
    } else if secs < 31_557_600 {
        Recency::Months
    } else {
        Recency::Years
    };

    let text = if recency == Recency::JustNow {
        "just now".to_string()
    } else if !past {
        let (value, unit) = match secs {
            s if s < 60 => (s, "second"),
            s if s < 3_600 => (s / 60, "minute"),
            s if s < 86_400 => (s / 3_600, "hour"),
            s if s < 604_800 => (s / 86_400, "day"),
            s => (s / 604_800, "week"),
        };
        let plural = if value == 1 { "" } else { "s" };
        format!("in {value} {unit}{plural}")
    } else {
        let (value, unit) = match secs {
            s if s < 60 => (s, "second"),
            s if s < 3_600 => (s / 60, "minute"),
            s if s < 86_400 => (s / 3_600, "hour"),
            s if s < 604_800 => (s / 86_400, "day"),
            s if s < 2_629_746 => (s / 604_800, "week"),
            s if s < 31_557_600 => (s / 2_629_746, "month"),
            s => (s / 31_557_600, "year"),
        };
        let plural = if value == 1 { "" } else { "s" };
        format!("{value} {unit}{plural} ago")
    };
    (text, recency)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Recency {
    JustNow,
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
    Years,
    Future,
    Unknown,
}

fn color_modified(text: &str, recency: Recency) -> String {
    let color = match recency {
        Recency::JustNow | Recency::Seconds => palette::MODIFIED_RECENT,
        Recency::Minutes => palette::MODIFIED_SOON,
        Recency::Hours => palette::MODIFIED,
        Recency::Days => palette::MODIFIED_HOURS,
        Recency::Weeks => palette::MODIFIED_DAYS,
        Recency::Months => palette::MODIFIED_WEEKS,
        Recency::Years => palette::MODIFIED_OLD,
        Recency::Future => palette::MODIFIED_FUTURE,
        Recency::Unknown => palette::MODIFIED,
    };
    palette::paint(text, color)
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
