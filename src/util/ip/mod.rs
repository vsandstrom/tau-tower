use crate::config::TauConfigError;
use is_ip::is_ip;
use regex_lite::Regex;

pub fn validate_ip(ip: String) -> Result<String, TauConfigError> {
  if !is_ip(&ip) {
    return Err(TauConfigError::InvalidIp(ip));
  }
  Ok(ip)
}

pub fn parse_port(p: &str) -> Result<u16, TauConfigError> {
  p.parse::<u16>()
    .map_err(|e| TauConfigError::Input(format!("Unable to parse as number: {e}")))
}

pub fn validate_port(port: u16) -> Result<u16, TauConfigError> {
  if !(1..=0xFFFF).contains(&port) {
    return Err(TauConfigError::InvalidPort(port.to_string()));
  }
  Ok(port)
}

pub fn validate_endpoint(endpoint: &str) -> Result<String, TauConfigError> {
  #[allow(clippy::expect_used)]
  if Regex::new(r"^/?[a-zA-Z0-9._]+$")
    .expect("regex is malformed and could not be built")
    .is_match(endpoint) {
      return Ok(endpoint.to_string());
  } 
  Err(TauConfigError::InvalidEndpoint(endpoint.to_string()))
}

pub fn filter_mount_endpoint(endpoint: &str) -> anyhow::Result<String> {
  if let Ok(endpoint) = validate_endpoint(endpoint) {
    // if missing beginning '/', add it
    return Ok(if &endpoint[..1] == "/" { 
      endpoint 
    } else { 
      format!("/{endpoint}") 
    });
  }

  anyhow::bail!("endpoint is badly formatted - check your config : {endpoint}")
}
