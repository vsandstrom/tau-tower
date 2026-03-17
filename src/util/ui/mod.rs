use std::path::Path;
use std::net::IpAddr;
use inline_colorization::{ color_reset, color_bright_red, color_bright_yellow, color_cyan};

pub fn server_started_info(ip: IpAddr, port: u16, endpoint: &str) {
  println!(
    "\
    {color_bright_yellow}Broadcasting on:{color_reset}\n\t{color_cyan}http://{ip}:{port}{endpoint}{color_reset}", 
  );
}

pub fn config_file_created_info(path: &Path) {
  let path = path.display();
  println!(
    "\
    \n{color_bright_yellow}A config file has been written to:{color_reset}\n\t\
    {color_bright_red}{path}{color_reset}\n", 
  );
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::{net::{IpAddr, Ipv4Addr}, path::PathBuf, str::FromStr};
  
  #[test] 
  fn print_server_started() {
    server_started_info(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080, "/endpoint");
  }
   
  #[test] 
  fn print_config_created() {
    config_file_created_info(&PathBuf::from_str("./path/to/file").unwrap());
  }
}
