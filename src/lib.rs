//  LIB.rs
//    by Lut99
// 
//  Created:
//    12 Feb 2023, 13:39:26
//  Last edited:
//    03 Mar 2023, 17:52:22
//  Auto updated?
//    Yes
// 
//  Description:
//!   A simple implementation for the
//!   [log](https://https//docs.rs/log/latest/log/) crate that aims to
//!   have a pretty, user-friendly mode, and a comprehensive, dev-friendly
//!   _debug_ mode.
// 

use std::any::Any;
use std::io::{Stderr, Stdin, Stdout, Write};
use std::ops::DerefMut as _;

use atty::Stream;
use chrono::Local;
use console::{style, Style};
use log::{Level, LevelFilter, Log, SetLoggerError};
use parking_lot::{Mutex, MutexGuard};


/***** HELPER MACROS *****/
/// Writes something to the given LogWriter.
macro_rules! log_write {
    ($enabled:ident, $writer:ident, $($t:tt)*) => {
        if let Err(err) = write!($writer.writer, $($t)*) {
            eprintln!("{}: Failed to write to writer '{}': {} (will not attempt again)", style("WARNING").yellow().bold(), $writer.label, err);
            *$enabled = false;
            continue;
        }
    };
}

/// Writes something to the given LogWriter but with a newline
macro_rules! log_writeln {
    ($enabled:ident, $writer:ident, $($t:tt)*) => {
        if let Err(err) = writeln!($writer.writer, $($t)*) {
            eprintln!("{}: Failed to write to writer '{}': {} (will not attempt again)", style("WARNING").yellow().bold(), $writer.label, err);
            *$enabled = false;
            continue;
        }
    };
}





/***** AUXILLARY *****/
/// Defines the mode to print it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DebugMode {
    /// No debugging, only warning and errors
    Friendly,
    /// Debugs also info and debug
    Debug,
    /// Debugs that + trace
    Full,
}



/// Enum that can be used to choose whether colour should be enabled.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ColourChoice {
    /// Colours are always written.
    Yes,
    /// Colours are never written.
    No,
    /// Evaluates to 'yes' if the given object is a TTY, or 'no' otherwise.
    Auto,
}
impl ColourChoice {
    /// Resolves this ColourChoice based on the given writer.
    /// 
    /// # Arguments
    /// - `writer`: The writer to resolve the ColourChoice with.
    /// 
    /// # Returns
    /// True if self was (`ColourChoice::Yes` || (`ColourChoice::Auto` && `writer` is TTY)), or else false.
    fn resolve(&self, writer: &(impl 'static + Write)) -> bool {
        match self {
            // Easy cases first
            ColourChoice::Yes => true,
            ColourChoice::No  => false,

            // Then the hard one
            ColourChoice::Auto => if (writer as &dyn Any).downcast_ref::<Stdin>().is_some() {
                atty::is(Stream::Stdin)
            } else if (writer as &dyn Any).downcast_ref::<Stderr>().is_some() {
                atty::is(Stream::Stderr)
            } else if (writer as &dyn Any).downcast_ref::<Stdout>().is_some() {
                atty::is(Stream::Stdout)
            } else {
                false
            }
        }
    }
}



/// Wrapper around a Write-capable type that filters the types of messages that are written to it.
pub struct LogWriter {
    /// The debug label of this writer.
    label  : String,
    /// The writer to write to.
    writer : Box<dyn Send + Sync + Write>,
    /// Whether to write to this writer with ANSI.
    colour : bool,
    /// The set of filters to allow.
    filter : Vec<Level>,
}
impl LogWriter {
    /// Default constructor for the LogWriter that initializes it for stdout.
    /// 
    /// By default, will use automatic colour selection and only logs trace, debug and info-messages.
    /// 
    /// # Returns
    /// A new LogWriter instance that can be used to log to stdout.
    #[inline]
    pub fn stdout() -> Self { Self::new("stdout", std::io::stdout(), ColourChoice::Auto, vec![ Level::Trace, Level::Debug, Level::Info ]) }

    /// Default constructor for the LogWriter that initializes it for stderr.
    /// 
    /// By default, will use automatic colour selection and only logs warning and error message.
    /// 
    /// # Returns
    /// A new LogWriter instance that can be used to log to stderr.
    #[inline]
    pub fn stderr() -> Self { Self::new("stderr", std::io::stderr(), ColourChoice::Auto, vec![ Level::Warn, Level::Error ]) }

    /// Constructor for the LogWriter that wraps it around the given `Write`r.
    /// 
    /// # Arguments
    /// - `label`: Some description of the writer for debugging purposes.
    /// - `writer`: The handle or other object that implement `Write` and we will write to.
    /// - `colour_choice`: Whether to enable ANSI colours for this file or not.
    /// - `filter`: The list of Levels that are only allowed to be written to this writer.
    /// 
    /// # Returns
    /// A new LogWriter instance that wraps around the given `writer`.
    #[inline]
    pub fn new(label: impl Into<String>, writer: impl 'static + Send + Sync + Write, colour: ColourChoice, filter: impl Into<Vec<Level>>) -> Self {
        // Resolve the colour first
        let colour: bool = colour.resolve(&writer);

        // Return ourselves with that colour
        Self {
            label  : label.into(),
            writer : Box::new(writer),
            colour,
            filter : filter.into(),
        }
    }
}





/***** LIBRARY *****/
/// Defines a logger that has a pretty, user-friendly mode, and a comprehensive, dev-friendly  _debug_ mode.
pub struct HumanLogger {
    /// Some writers that we write to.
    writers : Vec<Mutex<(bool, LogWriter)>>,
    /// Whether we are in debug mode or not.
    debug   : DebugMode,
}

impl HumanLogger {
    /// Default constructor for the HumanLogger that prepares it for logging to the terminal.
    /// 
    /// Logs to both stdout and stderr (warning and errors to the latter, the rest to the first), and uses automatic colour selection.
    /// 
    /// For more customization options, use `HumanLogger::new()` and `LogWriter::new()`.
    /// 
    /// # Arguments
    /// - `debug`: Whether to enable debug mode or not.
    /// 
    /// # Returns
    /// A new HumanLogger that will log to stdout and stderr.
    #[inline]
    pub fn terminal(debug: DebugMode) -> Self { Self::new(vec![ LogWriter::stdout(), LogWriter::stderr() ], debug) }

    /// Constructor for the HumanLogger that will log to the given set of `Write`rs.
    /// 
    /// # Arguments
    /// - `writers`: A list of writers to write to. You can configure for each of them if they should add ANSI colours to their output or not, and which log levels need to be written to them.
    /// - `debug`: Whether to enable debug mode or not.
    /// 
    /// # Returns
    /// A new HumanLogger instance that can then be installed in the `log`-crate.
    pub fn new(writers: impl IntoIterator<Item = LogWriter>, debug: DebugMode) -> Self {
        Self {
            writers : writers.into_iter().map(|w| Mutex::new((true, w))).collect(),
            debug,
        }
    }



    /// Initializes this logger as the `log`-crate's logger.
    pub fn init(self) -> Result<(), SetLoggerError> {
        // Set the logger
        let debug = self.debug;
        let res   = log::set_boxed_logger(Box::new(self));

        // Set the maximum level based on the debug
        if res.is_ok() {
            log::set_max_level(if debug == DebugMode::Friendly { LevelFilter::Warn } else if debug == DebugMode::Debug { LevelFilter::Debug } else { LevelFilter::Trace });
        }

        // Done
        res
    }
}

impl Log for HumanLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        // Find if any writer likes this message
        for w in &self.writers {
            let lock: MutexGuard<(bool, LogWriter)> = w.lock();

            // Only accept if enabled and there's a filter that matched this one
            if lock.0 && lock.1.filter.contains(&metadata.level()) { return true; }
        }
        false
    }

    fn log(&self, record: &log::Record) {
        // Write it to all writers who like this message
        for w in &self.writers {
            let mut lock: MutexGuard<(bool, LogWriter)> = w.lock();
            let (enabled, writer): &mut (bool, LogWriter) = lock.deref_mut();

            // Skip if the writer does not accept this message for whatever reason
            if !*enabled || !writer.filter.contains(&record.level()) { continue; }

            // Write the time, if debug logging
            if self.debug == DebugMode::Debug {
                log_write!(enabled, writer, "[{} ", Style::new().force_styling(writer.colour).dim().apply_to(Local::now().format("%Y-%m-%dT%H:%M:%SZ")));
            } else if self.debug == DebugMode::Full {
                log_write!(enabled, writer, "[{} ", Style::new().force_styling(writer.colour).dim().apply_to(Local::now().to_rfc3339()));
            }
            // Write the verbosity level
            log_write!(enabled, writer, "{}", match record.level() {
                Level::Trace => Style::new().force_styling(writer.colour).bold().apply_to("TRACE"),
                Level::Debug => Style::new().force_styling(writer.colour).bold().blue().apply_to("DEBUG"),
                Level::Info  => Style::new().force_styling(writer.colour).bold().green().apply_to("INFO"),
                Level::Warn  => Style::new().force_styling(writer.colour).bold().yellow().apply_to("WARNING"),
                Level::Error => Style::new().force_styling(writer.colour).bold().red().apply_to("ERROR"),
            });
            // Write the module
            if self.debug == DebugMode::Debug {
                let target: &str = record.target();
                if let Some(module_path) = record.module_path() {
                    // We only add if they actually differ
                    if module_path != target {
                        log_write!(enabled, writer, " {}", Style::new().force_styling(writer.colour).dim().apply_to(module_path));
                    }
                }
                log_write!(enabled, writer, " {}]", Style::new().force_styling(writer.colour).bold().apply_to(target));
            } else if self.debug == DebugMode::Full {
                if let Some(file) = record.file() {
                    log_write!(enabled, writer, " {}{}",
                        Style::new().force_styling(writer.colour).dim().apply_to(file),
                        if let Some(line) = record.line() {
                            format!("{}", Style::new().force_styling(writer.colour).dim().apply_to(format!(":{}", line)))
                        } else {
                            String::new()
                        },
                    );
                }
                log_write!(enabled, writer, " {}]", Style::new().force_styling(writer.colour).bold().apply_to(record.target()));
            }

            // Now write the message
            log_writeln!(enabled, writer, "{}{}", if self.debug == DebugMode::Friendly { ": " } else { " " }, record.args());
        }
    }

    fn flush(&self) {
        // Flush the writers, if enabled
        for w in &self.writers {
            let mut lock: MutexGuard<(bool, LogWriter)> = w.lock();

            // Skip if not enabled
            if !lock.0 { continue; }

            // Attempt to flush
            if let Err(err) = lock.1.writer.flush() {
                eprintln!("{}: Failed to flush writer '{}': {} (will not attempt again)", style("WARNING").yellow().bold(), lock.1.label, err);
                lock.0 = false;
                continue;
            }
        }
    }
}
