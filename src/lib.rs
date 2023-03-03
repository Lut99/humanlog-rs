//  LIB.rs
//    by Lut99
// 
//  Created:
//    12 Feb 2023, 13:39:26
//  Last edited:
//    03 Mar 2023, 18:51:51
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
use std::sync::Arc;

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

/// Flushes a given list of log writers.





/***** AUXILLARY *****/
/// Defines the mode to print the log messages in the HumanLogger.
/// 
/// Note that it applies both a change in _what_ is logged, as well as _how_ it is logged (i.e., the formatting changes too).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DebugMode {
    /// No debugging, only warnings (`Level::Warn`) and errors (`Level::Error`).
    /// 
    /// # Examples
    /// ```rust
    /// use humanlog::{DebugMode, HumanLogger};
    /// use log::{debug, error, info, trace, warn};
    /// 
    /// // Setup the logger to write to the terminal with default settings and the prettiest (and least informative) debug mode
    /// if let Err(err) = HumanLogger::terminal(DebugMode::HumanFriendly).init() {
    ///     eprintln!("WARNING: Failed to initialize logger: {} (no logging enabled for this session)", err);
    /// }
    /// 
    /// error!("This is an error!");
    /// warn!("This is a warning!");
    /// info!("This is an info message!");
    /// debug!("This is a debug message!");
    /// trace!("This is a trace message!");
    /// ```
    /// 
    /// This will show:
    /// ```bash
    /// ERROR: This is an error!
    /// WARNING: This is a warning!
    /// ```
    HumanFriendly,
    /// Debugs `Level::Info` and `Level::Debug` in addition to those of `DebugMode::HumanFriendly`.
    /// 
    /// # Examples
    /// ```rust
    /// use humanlog::{DebugMode, HumanLogger};
    /// use log::{debug, error, info, trace, warn};
    /// 
    /// // Setup the logger to write to the terminal with a server-level verbosity and formatting
    /// if let Err(err) = HumanLogger::terminal(DebugMode::Debug).init() {
    ///     eprintln!("WARNING: Failed to initialize logger: {} (no logging enabled for this session)", err);
    /// }
    /// 
    /// error!("This is an error!");
    /// warn!("This is a warning!");
    /// info!("This is an info message!");
    /// debug!("This is a debug message!");
    /// trace!("This is a trace message!");
    /// ```
    /// 
    /// This will show:
    /// ```bash
    /// [2023-03-03T18:10:13Z ERROR debug] This is an error!
    /// [2023-03-03T18:10:13Z WARNING debug] This is a warning!
    /// [2023-03-03T18:10:13Z INFO debug] This is an info message!
    /// [2023-03-03T18:10:13Z DEBUG debug] This is a debug message!
    /// ```
    Debug,
    /// Debugs everything, which is everything `DebugLevel::Debug` does, plus `Level::Trace`.
    /// 
    /// # Examples
    /// ```rust
    /// use humanlog::{DebugMode, HumanLogger};
    /// use log::{debug, error, info, trace, warn};
    /// 
    /// // Setup the logger to write to the terminal with the most verbose and extensive mode available.
    /// if let Err(err) = HumanLogger::terminal(DebugMode::Full).init() {
    ///     eprintln!("WARNING: Failed to initialize logger: {} (no logging enabled for this session)", err);
    /// }
    /// 
    /// error!("This is an error!");
    /// warn!("This is a warning!");
    /// info!("This is an info message!");
    /// debug!("This is a debug message!");
    /// trace!("This is a trace message!");
    /// ```
    /// 
    /// This will show:
    /// ```bash
    /// [2023-03-03T18:11:37.853292702+01:00 ERROR examples/full.rs:27 full] This is an error!
    /// [2023-03-03T18:11:37.853450438+01:00 WARNING examples/full.rs:28 full] This is a warning!
    /// [2023-03-03T18:11:37.853482929+01:00 INFO examples/full.rs:29 full] This is an info message!
    /// [2023-03-03T18:11:37.853495693+01:00 DEBUG examples/full.rs:30 full] This is a debug message!
    /// [2023-03-03T18:11:37.853507184+01:00 TRACE examples/full.rs:31 full] This is a trace message!
    /// ```
    Full,
}



/// Enum that can be used to choose whether colour should be enabled in the HumanLogger's log messages.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ColourChoice {
    /// ANSI colours are always written, regardless of what we write to.
    Yes,
    /// ANSI colours are never written, regardless of what we write to.
    No,
    /// ANSI colours are written depending on whether we are writing to a TTY (in which case we will), or some other output (in which case we won't).
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
/// 
/// This can be used to customize the output source of the HumanLogger.
/// 
/// # Examples
/// 
/// A logger that writes to stdout only, instead of mixed stdout/stderr (like usual).
/// ```rust
/// use humanlog::{ColourChoice, DebugMode, HumanLogger, LogWriter};
/// use log::{debug, info, error, Level};
/// 
/// let logger: LogWriter = LogWriter::new(std::io::stdout(), ColourChoice::Auto, vec![ Level::Error, Level::Warn, Level::Info, Level::Debug, Level::Trace ], "stdout");
/// if let Err(err) = HumanLogger::new(vec![ logger ], DebugMode::Debug).init() {
///     eprintln!("WARNING: Failed to initialize logger: {} (no logging enabled for this session)", err);
/// }
/// ```
/// 
/// A logger that writes to a file instead of stdout/stderr.
/// ```rust
/// use std::fs::File;
/// use humanlog::{ColourChoice, DebugMode, HumanLogger, LogWriter};
/// use log::{debug, info, error, Level};
/// 
/// // Open a file
/// match File::create("output.log") {
///     Ok(handle) => {
///         // Use that to create a writer that receives everything
///         let logger: LogWriter = LogWriter::new(handle, ColourChoice::No, vec![ Level::Error, Level::Warn, Level::Info, Level::Debug, Level::Trace ], "file");
///         if let Err(err) = HumanLogger::new(vec![ logger ], DebugMode::Debug).init() {
///             eprintln!("WARNING: Failed to initialize logger: {} (no logging enabled for this session)", err);
///         }
///     },
///
///     Err(err) => {
///         eprintln!("WARNING: Failed to initialize logger: Failed to create file 'output.log': {} (no logging enabled for this session)", err);
///     },
/// }
/// ```
/// 
/// A logger that writes to a file _in addition to_ the standard stdout/stderr logging.
/// ```rust
/// use std::fs::File;
/// use humanlog::{ColourChoice, DebugMode, HumanLogger, LogWriter};
/// use log::{debug, info, error, Level};
/// 
/// // Open a file
/// match File::create("output.log") {
///     Ok(handle) => {
///         // Create three LogWriters: one per output stream
///         let stdout_logger: LogWriter = LogWriter::stdout();
///         let stderr_logger: LogWriter = LogWriter::stderr();
///         // Note the repeated levels; the logger will simply log to all LogWriters that want that particular level
///         let file_logger: LogWriter = LogWriter::new(handle, ColourChoice::No, vec![ Level::Error, Level::Warn, Level::Info, Level::Debug, Level::Trace ], "file");
/// 
///         // Finally, we put it all in one logger
///         if let Err(err) = HumanLogger::new(vec![ stdout_logger, stderr_logger, file_logger ], DebugMode::Debug).init() {
///             eprintln!("WARNING: Failed to initialize logger: {} (no logging enabled for this session)", err);
///         }
///     },
/// 
///     Err(err) => {
///         eprintln!("WARNING: Failed to initialize logger: Failed to create file 'output.log': {} (no logging enabled for this session)", err);
///     },
/// }
/// ```
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
    /// 
    /// # Examples
    /// ```rust
    /// use humanlog::{DebugMode, HumanLogger, LogWriter};
    /// use log::{debug, info, error, Level};
    /// 
    /// // Will emulate the default behaviour of writing `Level::Error` and `Level::Warn` to stderr, the rest to stdout.
    /// if let Err(err) = HumanLogger::new(vec![ LogWriter::stdout(), LogWriter::stderr() ], DebugMode::Debug).init() {
    ///     eprintln!("WARNING: Failed to initialize logger: {} (no logging enabled for this session)", err);
    /// }
    /// ```
    #[inline]
    pub fn stdout() -> Self { Self::new(std::io::stdout(), ColourChoice::Auto, vec![ Level::Trace, Level::Debug, Level::Info ], "stdout") }

    /// Default constructor for the LogWriter that initializes it for stderr.
    /// 
    /// By default, will use automatic colour selection and only logs warning and error message.
    /// 
    /// # Returns
    /// A new LogWriter instance that can be used to log to stderr.
    /// 
    /// # Examples
    /// ```rust
    /// use humanlog::{DebugMode, HumanLogger, LogWriter};
    /// use log::{debug, info, error, Level};
    /// 
    /// // Will emulate the default behaviour of writing `Level::Error` and `Level::Warn` to stderr, the rest to stdout.
    /// if let Err(err) = HumanLogger::new(vec![ LogWriter::stdout(), LogWriter::stderr() ], DebugMode::Debug).init() {
    ///     eprintln!("WARNING: Failed to initialize logger: {} (no logging enabled for this session)", err);
    /// }
    /// ```
    #[inline]
    pub fn stderr() -> Self { Self::new(std::io::stderr(), ColourChoice::Auto, vec![ Level::Warn, Level::Error ], "stderr") }

    /// Constructor for the LogWriter that wraps it around the given `Write`r.
    /// 
    /// # Arguments
    /// - `writer`: The handle or other object that implement `Write` and we will write to.
    /// - `colour_choice`: Whether to enable ANSI colours for this file or not.
    /// - `filter`: The list of Levels that are only allowed to be written to this writer.
    /// - `label`: Some description of the writer for debugging purposes.
    /// 
    /// # Returns
    /// A new LogWriter instance that wraps around the given `writer`.
    /// 
    /// # Examples
    /// ```rust
    /// use humanlog::{ColourChoice, DebugMode, HumanLogger, LogWriter};
    /// use log::{debug, info, error, Level};
    /// 
    /// // Will only ever write to stdout, regardless of the log type
    /// let logger: LogWriter = LogWriter::new(std::io::stdout(), ColourChoice::Auto, vec![ Level::Error, Level::Warn, Level::Info, Level::Debug, Level::Trace ], "stdout");
    /// if let Err(err) = HumanLogger::new(vec![ logger ], DebugMode::Debug).init() {
    ///     eprintln!("WARNING: Failed to initialize logger: {} (no logging enabled for this session)", err);
    /// }
    /// ```
    #[inline]
    pub fn new(writer: impl 'static + Send + Sync + Write, colour: ColourChoice, filter: impl Into<Vec<Level>>, label: impl Into<String>) -> Self {
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

/// An inner counterpart of LogWriter that does not carry filter information anymore.
struct InternalLogWriter {
    /// The debug label of this writer.
    label  : String,
    /// The writer to write to.
    writer : Box<dyn Send + Sync + Write>,
    /// Whether to write to this writer with ANSI.
    colour : bool,
}
impl From<LogWriter> for InternalLogWriter {
    #[inline]
    fn from(value: LogWriter) -> Self {
        Self {
            label  : value.label,
            writer : value.writer,
            colour : value.colour,
        }
    }
}





/***** LIBRARY *****/
/// Defines a logger that has a pretty, user-friendly mode, and a comprehensive, dev-friendly  _debug_ mode.
pub struct HumanLogger {
    /// All writers that write `Level::Error`.
    error_writers : Vec<Arc<Mutex<(bool, InternalLogWriter)>>>,
    /// All writers that write `Level::Warn`.
    warn_writers  : Vec<Arc<Mutex<(bool, InternalLogWriter)>>>,
    /// All writers that write `Level::Info`.
    info_writers  : Vec<Arc<Mutex<(bool, InternalLogWriter)>>>,
    /// All writers that write `Level::Debug`.
    debug_writers : Vec<Arc<Mutex<(bool, InternalLogWriter)>>>,
    /// All writers that write `Level::Trace`.
    trace_writers : Vec<Arc<Mutex<(bool, InternalLogWriter)>>>,

    /// Which debug mode to log with.
    debug : DebugMode,
}

impl HumanLogger {
    /// Constructor for the HumanLogger that will log to the given set of `Write`rs.
    /// 
    /// # Arguments
    /// - `writers`: A list of writers to write to. You can configure for each of them if they should add ANSI colours to their output or not, and which log levels need to be written to them.
    /// - `debug`: Whether to enable debug mode or not.
    /// 
    /// # Returns
    /// A new HumanLogger instance that can then be installed in the `log`-crate.
    pub fn new(writers: impl IntoIterator<Item = LogWriter>, debug: DebugMode) -> Self {
        // Sort the given writers into the given lists
        let mut error_writers : Vec<Arc<Mutex<(bool, InternalLogWriter)>>> = vec![];
        let mut warn_writers  : Vec<Arc<Mutex<(bool, InternalLogWriter)>>> = vec![];
        let mut info_writers  : Vec<Arc<Mutex<(bool, InternalLogWriter)>>> = vec![];
        let mut debug_writers : Vec<Arc<Mutex<(bool, InternalLogWriter)>>> = vec![];
        let mut trace_writers : Vec<Arc<Mutex<(bool, InternalLogWriter)>>> = vec![];
        for writer in writers.into_iter() {
            // Create the base arc
            let filters : Vec<Level> = writer.filter;
            let writer  : Arc<Mutex<(bool, InternalLogWriter)>> = Arc::new(Mutex::new((true, writer.into())));

            // Add it to any list it wants
            for filter in filters {
                match filter {
                    Level::Error => error_writers.push(writer.clone()),
                    Level::Warn  => warn_writers.push(writer.clone()),
                    Level::Info  => info_writers.push(writer.clone()),
                    Level::Debug => debug_writers.push(writer.clone()),
                    Level::Trace => trace_writers.push(writer.clone()),
                }
            }
        }

        // We can now store this
        Self {
            error_writers,
            warn_writers,
            info_writers,
            debug_writers,
            trace_writers,

            debug,
        }
    }

    /// Default constructor for the HumanLogger that prepares it for logging to the terminal.
    /// 
    /// Logs to both stdout and stderr (errors and warnings to the latter, the rest to the first), and uses automatic colour selection.
    /// 
    /// For more customization options, use `HumanLogger::new()` with a list of `LogWriter`s.
    /// 
    /// # Arguments
    /// - `mode`: The mode of debugging to use for this session. Decides both which `Level`s to apply, and how to format the resulting messages.
    /// 
    /// # Returns
    /// A new HumanLogger that will log to stdout and stderr.
    #[inline]
    pub fn terminal(mode: DebugMode) -> Self { Self::new(vec![ LogWriter::stdout(), LogWriter::stderr() ], mode) }



    /// Initializes this logger as the `log`-crate's logger.
    /// 
    /// # Errors
    /// Tihs function may error if we failed to setup the logger. This can happen if there already was one or any other reason that `log` crashes.
    /// 
    /// # Examples
    /// ```rust
    /// 
    /// ```
    pub fn init(self) -> Result<(), SetLoggerError> {
        // Set the logger
        let debug = self.debug;
        log::set_boxed_logger(Box::new(self))?;

        // Set the maximum level based on the debug
        log::set_max_level(match debug {
            DebugMode::HumanFriendly => LevelFilter::Warn,
            DebugMode::Debug         => LevelFilter::Debug,
            DebugMode::Full          => LevelFilter::Trace,
        });

        // Done
        Ok(())
    }
}

impl Log for HumanLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        // Match on the level of the message to get the list of writers to write to
        let writers: &[Arc<Mutex<(bool, InternalLogWriter)>>] = match metadata.level() {
            Level::Error => &self.error_writers,
            Level::Warn  => &self.warn_writers,
            Level::Info  => &self.info_writers,
            Level::Debug => &self.debug_writers,
            Level::Trace => &self.trace_writers,
        };

        // Search those writers for _any_ non-enabled one
        writers.iter().any(|w| w.lock().0)
    }

    fn log(&self, record: &log::Record) {
        // Match on the level of the message to get the list of writers to write to
        let writers: &[Arc<Mutex<(bool, InternalLogWriter)>>] = match record.level() {
            Level::Error => &self.error_writers,
            Level::Warn  => &self.warn_writers,
            Level::Info  => &self.info_writers,
            Level::Debug => &self.debug_writers,
            Level::Trace => &self.trace_writers,
        };

        // Write it to all writers who like this message
        for w in writers {
            let mut lock: MutexGuard<(bool, InternalLogWriter)> = w.lock();
            let (enabled, writer): &mut (bool, InternalLogWriter) = lock.deref_mut();

            // Skip if the writer is no longer enabled (because of an error)
            if !*enabled { continue; }

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
            log_writeln!(enabled, writer, "{}{}", if self.debug == DebugMode::HumanFriendly { ": " } else { " " }, record.args());
        }
    }

    fn flush(&self) {
        // Flush all the writers if they are enabled
        for w in &self.error_writers {
            let mut lock: MutexGuard<(bool, InternalLogWriter)> = w.lock();

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
