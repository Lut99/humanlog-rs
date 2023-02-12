//  FULL.rs
//    by Lut99
// 
//  Created:
//    12 Feb 2023, 15:05:35
//  Last edited:
//    12 Feb 2023, 15:07:13
//  Auto updated?
//    Yes
// 
//  Description:
//!   Shows an example where we debug in the most verbose mode.
// 

use humanlog::{DebugMode, HumanLogger};
use log::{debug, error, info, warn, trace};


/***** ENTRYPOINT *****/
fn main() {
    // Enable with full debugging
    if let Err(err) = HumanLogger::terminal(DebugMode::Full).init() { eprintln!("WARNING: Failed to initialize logger: {} (no logging enabled for this session)", err); }

    // Write ALL messages!
    trace!("A trace message");
    debug!("A debug message");
    info!("An info message");
    warn!("A warning message");
    error!("An error message");
}
