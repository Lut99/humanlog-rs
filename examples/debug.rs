//  DEBUG.rs
//    by Lut99
// 
//  Created:
//    12 Feb 2023, 15:05:00
//  Last edited:
//    03 Mar 2023, 18:10:10
//  Auto updated?
//    Yes
// 
//  Description:
//!   Shows an example of debugging in debug-mode.
// 

use humanlog::{DebugMode, HumanLogger};
use log::{debug, error, info, trace, warn};


/**** ENTRYPOINT *****/
fn main() {
    // Setup the logger to write to the terminal with a server-level verbosity and formatting
    if let Err(err) = HumanLogger::terminal(DebugMode::Debug).init() {
        eprintln!("WARNING: Failed to initialize logger: {} (no logging enabled for this session)", err);
    }
        
    // Write some messages!
    error!("This is an error!");
    warn!("This is a warning!");
    info!("This is an info message!");
    debug!("This is a debug message!");
    trace!("This is a trace message!");
}
