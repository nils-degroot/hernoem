use audiotags::Tag;
use clap::Parser;
use env_logger::fmt::{self, Formatter};
use log::{info, warn, Level, Record};
use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
};
use thiserror::Error;
use walkdir::WalkDir;

mod format;

const AUDIO_EXTENSIONS: &[&'_ str] = &["mp3", "m4a", "mp4", "flac"];

lazy_static::lazy_static! {
    static ref ARGS: Args = Args::parse();
}

#[derive(Debug, Parser)]
struct Args {
    /// Directory/File to rename [default: "cwd"]
    #[arg(long, short)]
    from: Option<PathBuf>,
    /// Format to use. Results in a error was some data was missing
    #[arg(default_value_t = default_format())]
    format: String,
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

impl Args {
    fn from(&self) -> Option<&PathBuf> {
        self.from.as_ref()
    }

    fn format(&self) -> &str {
        self.format.as_ref()
    }

    fn verbose(&self) -> &clap_verbosity_flag::Verbosity {
        &self.verbose
    }
}

fn default_format() -> String {
    "%n - %t".to_string()
}

fn format_log(buf: &mut Formatter, record: &Record) -> Result<(), io::Error> {
    let mut level_style = buf.style();
    level_style.set_color(match record.level() {
        Level::Error => fmt::Color::Red,
        Level::Warn => fmt::Color::Yellow,
        Level::Info => fmt::Color::Green,
        Level::Debug => fmt::Color::Blue,
        Level::Trace => fmt::Color::Cyan,
    });

    let mut bracket_style = buf.style();
    bracket_style.set_dimmed(true);

    writeln!(
        buf,
        "{}{}{} {}",
        bracket_style.value("["),
        level_style.value(record.level()),
        bracket_style.value("]"),
        record.args()
    )
}

#[derive(Debug, Error)]
enum RenameError {
    #[error("Could not find a cwd, please specify a path `-f`")]
    CouldNotFindPath,
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .format(|r, a| format_log(r, a))
        .filter_level(ARGS.verbose().log_level_filter())
        .init();

    let path = ARGS
        .from()
        .map(|p| p.to_path_buf())
        .or_else(|| env::current_dir().ok())
        .ok_or(RenameError::CouldNotFindPath)?;

    if path.is_file() {
        rename_file(path)?;
    } else {
        let failure = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .map(|e| rename_file(e.path()))
            .find_map(|e| match e {
                Ok(_) => None,
                Err(e) => Some(e),
            });

        if let Some(reason) = failure {
            Err(reason)?;
        }
    }

    Ok(())
}

fn rename_file<P>(path: P) -> anyhow::Result<()>
where
    P: Into<PathBuf>,
{
    let path: PathBuf = path.into();

    let extension = match path.extension().and_then(|e| e.to_str()) {
        Some(e) => e,
        None => {
            warn!(
                "Ignoring file `{}`, extension missing",
                path.to_str().unwrap_or_default()
            );
            return Ok(());
        }
    };

    if !AUDIO_EXTENSIONS.contains(&extension) {
        warn!(
            "Ignoring file `{}`, extension not supported",
            path.to_str().unwrap_or_default()
        );
        return Ok(());
    }

    let tags = Tag::new().read_from_path(&path)?;
    let resulting_name = format::format_name(&tags.to_anytag(), ARGS.format())?;

    let old_name = match path.file_stem().and_then(|f| f.to_str()) {
        Some(n) => n,
        None => "",
    };

    if old_name == &resulting_name {
        warn!(
            "Ignoring file `{}`, file is already formatter correctly",
            path.to_str().unwrap_or_default()
        );
        return Ok(());
    }

    let mut updated = path.clone();
    updated.set_file_name(format!("{}.{}", resulting_name, extension));

    fs::rename(path.clone(), updated.clone())?;
    info!(
        "Renamed {} -> {}",
        old_name,
        format!("{}.{}", resulting_name, extension)
    );

    Ok(())
}
