# humanlog-rs
A simple logger for the [log](https://https://docs.rs/log/latest/log/) crate that aims to have a simple user mode and a comprehensive debug mode.


## Installation
To use this crate in your project, add the following lines to `[dependencies]` in your `Cargo.toml` file:
```toml
humanlog = { git = "https://github.com/Lut99/humanlog-rs" }
log = "0.4"
```
This will use the most cutting-edge version (i.e., the `master` branch).

You can also commit to a specific version instead by using the `tag`-selector:
```toml
humanlog = { git = "https://github.com/Lut99/humanlog-rs", tag = "v0.1.0" }
```

### Updating
If you want to update this crate after you've already added your dependencies, run:
```bash
cargo update --package humanlog
```
in your project to pull the most recent version. You can also omit the `--package humanlog` to update all packages.

Don't forget to move to a new tag if you've commited yourself to a specific one.


### Documentation
You can generate the Rustdocs by cloning the repository, and the running `cargo doc`:
```bash
git clone https://github.com/Lut99/humanlog-rs && cd humanlog-rs
cargo doc
```

You can then open `target/doc/humanlog/index.html` in your favourite browser.


## Usage
The following will setup the logger to work with the default settings:
```rust
use humanlog::{DebugMode, HumanLogger};
use log::{debug, info};

fn main() {
    // This will enable it to the verbose debugging mode
    if let Err(err) = HumanLogger::new(DebugMode::Debug).init() {
        eprintln!("WARNING: Failed to setup logger: {err} (no logging enabled for this session)");
    }
    info!("Successfully setup HumanLogger!");
    debug!("Time to crime...");
}
```

You can select between three modes by choosing different values of the `DebugMode`-enum:
- `DebugMode::HumanFriendly`: Only shows `warn`- and `error`-level logging messages, suitable for command-line tools used by humans.
- `DebugMode::Debug`: Shows `debug`-, `info`-, `warn`- and `error`-level logging messages, where each message is more like a log-line stating the time of logging, the module where it originated from, etc.
- `DebugMode::Full`: The same as `DebugMode::Debug`, except that `trace`-level logging messages are also logged, and information given per message is even more precise.

Typically, this crate is used in combination with command-line arguments to let the user switch between the various modes of logging. A common mode of usage is providing the user with the `HumanFriendly` mode by default, and providing them with flags `--debug` and `--trace` to enable those modes, respectively.
```rust
// We use [clap](https://docs.rs/clap/latest/clap/) to parse command-line arguments
// Enable the `derive` feature
use clap::Parser;
use humanlog::{DebugMode, HumanLogger};
use log::{debug, info};

/// Defines the command-line arguments for this executable.
#[derive(Parser)]
struct Arguments {
    /// Will change to `DebugMode::Debug`
    #[clap(long, global=true)]
    debug: bool,
    /// Will change to `DebugMode::Full`
    #[clap(long, global=true)]
    trace: bool,
}

fn main() {
    // Parse the arguments
    let args = Arguments::parse();

    // Enable the correct debugging mode based on the values
    if let Err(err) = HumanLogger::terminal(DebugMode::from_flags(args.debug, args.trace)).init() {
        eprintln!("WARNING: Failed to setup logger: {err} (no logging enabled for this session)");
    }
    info!("Successfully setup HumanLogger!");
    debug!("Time to crime...");
}
```

Another way might be to accept a `--verbose` option, where `0` is `HumanFriendly` (default), `1` is `Debug` and `2` is `Full`:
```rust
// We use [clap](https://docs.rs/clap/latest/clap/) to parse command-line arguments
// Enable the `derive` feature
use clap::Parser;
use humanlog::{DebugMode, HumanLogger};
use log::{debug, info};

/// Defines the command-line arguments for this executable.
#[derive(Parser)]
struct Arguments {
    /// Defines the level to parse.
    #[clap(short, long, default_value="0")]
    verbosity : u32,
}

fn main() {
    // Parse the arguments
    let args = Arguments::parse();

    // Enable the correct debugging mode based on the values
    if let Err(err) = HumanLogger::terminal(DebugMode::from_num(args.verbosity)).init() {
        eprintln!("WARNING: Failed to setup logger: {err} (no logging enabled for this session)");
    }
    info!("Successfully setup HumanLogger!");
    debug!("Time to crime...");
}
```


### `LogWriter`s
By default, the `HumanLogger` logs `error` and `warn` messages to stdout, and the rest to stderr. However, you can change this behaviour by defining one or more `LogWriter`s that define output channels for the logger.

To create a `LogWriter`, use the `LogWriter::new()`-method. This takes in the output `Write`r to write to, a `ColourChoice` which determines if ANSI-colours are used, and a filter that is used to determine which log levels to send to this writer. Note that multiple writers can have the same log level, to log to multiple writers.

For example, the following snippets will write everything to stdout:
```rust
use humanlog::{ColourChoice, DebugMode, HumanLogger, LogWriter};
use log::Level;

// Will only ever write to stdout, regardless of the log type
let logger: LogWriter = LogWriter::new(std::io::stdout(), ColourChoice::Auto, vec![ Level::Error, Level::Warn, Level::Info, Level::Debug, Level::Trace ], "stdout");
if let Err(err) = HumanLogger::new(vec![ logger ], DebugMode::Debug).init() {
    eprintln!("WARNING: Failed to initialize logger: {err} (no logging enabled for this session)");
}
```

For more information, you can consult the [documentation](#documentation) or check some examples in the [`examples`](/examples) directory of this repository.


## Contribution
Feel free to open up an [issue](https://github.com/Lut99/humanlog-rs/issues) or a [pull request](https://github.com/Lut99/humanlog-rs/pulls) if you encounter bugs, have any suggestions or feedback. I'll look at them as soon as I can.


## License
This project is licensed under GPLv3. You can find more information in the [`LICENSE`](/LICENSE) file.
