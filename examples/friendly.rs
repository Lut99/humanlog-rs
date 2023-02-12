//  EXAMPLE.rs
//    by Lut99
// 
//  Created:
//    12 Feb 2023, 14:45:53
//  Last edited:
//    12 Feb 2023, 15:07:09
//  Auto updated?
//    Yes
// 
//  Description:
//!   Shows an example where we debug in friendly mode.
// 

use humanlog::{DebugMode, HumanLogger};
use log::{debug, error, info, warn, trace};


/**** ENTRYPOINT *****/
fn main() {
    // Setup the most default logger in consice mode
    if let Err(err) = HumanLogger::terminal(DebugMode::Friendly).init() { eprintln!("WARNING: Failed to initialize logger: {} (no logging enabled for this session)", err); }

    // Write some messages!
    trace!("A trace message");
    debug!("A debug message");
    info!("An info message");
    warn!("A warning message");
    error!("An error message");
}
