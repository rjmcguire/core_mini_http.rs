use core::str::from_utf8;
use core::prelude::*;
use collections::vec::*;
use collections::String;
use collections::string::ToString;
use collections::BTreeMap;


#[derive(Debug)]
pub struct Url {
	pub scheme: String,
	pub port: Option<u16>,
	pub host: String,
	pub full_path: String
}

impl Url {
	pub fn parse(url: &str) -> Option<Url> {
		let scheme_sep = url.find("://");
		if scheme_sep.is_none() { return None; }
		let scheme_sep = scheme_sep.unwrap();
		let scheme = &url[0..scheme_sep];

		let mut port = None;
		let mut path = "";
		let mut host = "";
		let mut url_rest = &url[(scheme_sep+3)..];
		
		let host_end = url_rest.find("/");
		if host_end.is_none() {
			host = &url_rest;
			url_rest = &url_rest[0..0];
		} else {
			let host_end = host_end.unwrap();
			host = &url_rest[..host_end];
			url_rest = &url_rest[host_end..];
		}


		let port_sep = host.find(":");
		if port_sep.is_some() {
			let port_sep = port_sep.unwrap();
			let potential_port = &host[(port_sep + 1)..];
			let potential_port = potential_port.parse::<u16>();
			if potential_port.is_ok() {
				port = Some(potential_port.unwrap());
				host = &host[..port_sep];
			}
		}


		path = &url_rest;


		if port == None {
			port = match scheme {
				"http" => { Some(80) },
				"https" => { Some(443) },
				_ => { None }
			};
		}


		Some(Url {
			scheme: scheme.to_string(),
			port: port,
			host: host.to_string(),
			full_path: path.to_string()
		})
	}
}


#[cfg(test)]
mod tests {
	

	use super::*;


	use core::prelude::*;
	use std::prelude::*;
	use collections::vec::Vec;
	use collections::string::ToString;

	#[test]
	pub fn test_url_parser() {
		let url_str = "http://clients3.google.com/generate_204";
		let url = Url::parse(url_str);
		println!("url: {:?}", url);

		let url_str = "http://clients3.google.com";
		let url = Url::parse(url_str);
		println!("url: {:?}", url);

		let url_str = "http://clients3.google.com:8080/";
		let url = Url::parse(url_str);
		println!("url: {:?}", url);
	}

}