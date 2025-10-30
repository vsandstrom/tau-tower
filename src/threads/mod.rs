pub mod ws;
// pub mod udp;
pub mod http;

use std::time::Duration;

const TIMEOUT: Duration = Duration::from_millis(50);
const LOG_TIMEOUT: Duration = Duration::from_secs(10);

