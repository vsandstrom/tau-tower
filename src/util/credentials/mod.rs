
pub struct Credentials {
  pub username: String,
  pub password: String,
}

impl Credentials {
  pub fn validate(&self, username: &str, password: &str) -> bool {
    username == self.username && password == self.password 
  }
}

