use std::{env, str::FromStr};

pub fn get_env<T: FromStr>(key: &str, default: Option<T>) -> T {
  return match env::var(key) {
    Ok(val) => val.parse::<T>().ok().unwrap(),
    Err(err) => {
      if default.is_some() {
        return default.unwrap();
      };

      panic!("Required environment variable missing: {}", err);
    }
  };
}
