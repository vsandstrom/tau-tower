use std::path::PathBuf;

use inline_colorization::*;

pub fn server_started_info(ip: std::net::Ipv4Addr, port: u16, endpoint: &str) {
  println!("{}Broadcasting on:{}\n\t{}http://{}:{}{}{}", 
    color_bright_yellow,
    color_reset,
    color_cyan,
    ip, 
    port, 
    endpoint,
    color_reset
  );
}

pub fn config_file_created_info(path: PathBuf) {
  println!("\
    \n{}A config file has been written to:{}\n\t\
    {}{}{}\n", 
    color_bright_yellow,
    color_reset,
    color_bright_red,
    path.display(),
    color_reset
  )
}

#[cfg(test)]
mod tests {
  use std::{net::Ipv4Addr, str::FromStr};

use super::*;
  
  #[test] 
  fn print_server_started() {
    server_started_info(
      Ipv4Addr::UNSPECIFIED,
      8080, 
      "/endpoint"
    );
  }
  
  #[test] 
  fn print_config_created() {
    config_file_created_info( PathBuf::from_str("./path/to/file").unwrap());
  }
}
