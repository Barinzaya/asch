use serde::{Deserialize, de::Error as DeError};
use time::{Time};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
	pub mark_traffic: bool,

	#[serde(default, with = "config_time::option")]
	pub time: Option<Time>,
}

mod config_time {
	use super::*;
	use time::format_description::{FormatItem};
	use time::macros::{format_description};

	static FORMATS: &[&[FormatItem]] = &[
		format_description!("[hour repr:12]:[minute] [period case_sensitive:false]"),
		format_description!("[hour repr:24]:[minute]"),
	];

	fn parse<E: DeError>(s: &str) -> Result<Time, E> {
		for format in FORMATS {
			if let Ok(time) = Time::parse(s, format) {
				return Ok(time);
			}
		}

		Err(E::custom(format!("Failed to parse time (\"{}\")!", s)))
	}

	pub mod option {
		use super::*;

		pub fn deserialize<'de, D: serde::Deserializer<'de>>(de: D) -> Result<Option<Time>, D::Error> {
			if let Some(s) = <Option<&'de str>>::deserialize(de)? {
				parse(s).map(Some)
			} else {
				Ok(None)
			}
		}
	}
}
