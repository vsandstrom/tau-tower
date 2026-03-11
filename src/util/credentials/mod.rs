
pub struct Credentials {
  pub username: String,
  pub password: String,
}

impl Credentials {
  pub fn validate(&self, username: Option<&str>, password: Option<&str>) -> bool {
    if (username, password) == (
      Some(&self.username),
      Some(&self.password),
      ) {
      return true
    }
    false
  }
}

