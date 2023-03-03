//  EXAMPLE.rs
//    by Lut99
// 
//  Created:
//    12 Feb 2023, 14:45:53
//  Last edited:
//    03 Mar 2023, 18:08:50
//  Auto updated?
//    Yes
// 
//  Description:
//!   Shows an example where we debug in friendly mode.
// 

use humanlog::{DebugMode, HumanLogger};
use log::{debug, error, info, trace, warn};


/**** ENTRYPOINT *****/
fn main() {
    // Setup the logger to write to the terminal with default settings and the prettiest (and least informative) debug mode
    if let Err(err) = HumanLogger::terminal(DebugMode::HumanFriendly).init() {
        eprintln!("WARNING: Failed to initialize logger: {} (no logging enabled for this session)", err);
    }
        
    // Write some messages!
    error!("This is an error!");
    warn!("This is a warning!");
    info!("This is an info message!");
    debug!("This is a debug message!");
    trace!("This is a trace message!");
}
