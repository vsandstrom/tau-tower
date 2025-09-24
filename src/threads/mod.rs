pub mod ws;
pub mod udp;
pub mod http;

use std::time::Duration;

pub const MTU: usize = 1500;
const TIMEOUT: Duration = Duration::from_millis(50);
const LOG_TIMEOUT: Duration = Duration::from_secs(10);

