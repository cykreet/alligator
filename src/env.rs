use std::{env, str::FromStr, time::Duration};

pub const DEFAULT_PORT: u16 = 8080;
pub const DEFAULT_DELIVER_DURATION: Duration = Duration::from_secs(7);
pub const DEFAULT_WEBHOOK_ENDPOINT: &str = "https://discord.com/api/webhooks/";
pub const DEFAULT_EMBED_LIMIT: u8 = 10;

pub fn get_env<T: FromStr>(name: &str, require: bool) -> Option<T> {
	return match env::var(name) {
		Ok(val) => val.parse::<T>().ok(),
		Err(_) => {
			if require {
				panic!("Required environment variable missing: {}", name);
			}

			return None;
		}
	};
}

pub fn get_env_default<T: FromStr>(name: &str, default: T) -> T {
	return match get_env(name, false) {
		Some(val) => val,
		None => default,
	};
}
