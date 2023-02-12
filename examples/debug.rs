//  DEBUG.rs
//    by Lut99
// 
//  Created:
//    12 Feb 2023, 15:05:00
//  Last edited:
//    12 Feb 2023, 15:07:11
//  Auto updated?
//    Yes
// 
//  Description:
//!   Shows an example of debugging in debug-mode.
// 

use humanlog::{DebugMode, HumanLogger};
use log::{debug, error, info, warn, trace};


/***** ENTRYPOINT *****/
fn main() {
    // Enable with much more debugging
    if let Err(err) = HumanLogger::terminal(DebugMode::Debug).init() { eprintln!("WARNING: Failed to initialize logger: {} (no logging enabled for this session)", err); }

    // Write some more messages!
    trace!("A trace message");
    debug!("A debug message");
    info!("An info message");
    warn!("A warning message");
    error!("An error message");
}
