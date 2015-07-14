use core::str::from_utf8;
use http::*;
use core::prelude::*;
use collections::vec::*;
use collections::String;
use collections::string::ToString;

pub struct HttpParser {
	buffer: Vec<u8>,
	pos: usize,
	line_num: u16,
	headers_parsed: bool,

	msg: HttpMessage
}

#[derive(Debug)]
pub enum HttpParserState {
	MoreDataRequired,
	Complete
}

#[derive(Debug)]
pub enum HttpParserError {
	InvalidString,
	HeaderError,
	LineParseError(String)
}

impl HttpParser {
	pub fn new(msg: HttpMessage) -> HttpParser {
		HttpParser {
			buffer: Vec::new(),
			pos: 0,
			line_num: 0,
			headers_parsed: false,
			msg: msg
		}
	}

	pub fn new_request() -> HttpParser {
		HttpParser::new(HttpMessage::Request(HttpRequestMessage::empty()))
	}

	pub fn is_first_line_parsed(&self) -> bool {
		self.line_num > 0
	}

	pub fn are_headers_parsed(&self) -> bool {
		self.headers_parsed
	}

	pub fn read_how_many_bytes(&self) -> u32 {
		if self.is_first_line_parsed() && self.are_headers_parsed() {
			match self.msg {
				HttpMessage::Request(ref req) => {
					if req.method == HttpMethod::Get || req.method == HttpMethod::Head {
		            	return 0;
		            }
				}
				HttpMessage::Response(_) => {}
			}            

            let cl = self.msg.content_length();
            if cl.is_some() {
            	return cl.unwrap() - self.msg.get_body().len() as u32;
            }
        }

        return 1;
	}

	pub fn parse_bytes(&mut self, data: &[u8]) -> Result<HttpParserState, HttpParserError> {
		if data.len() == 0 { return Ok(HttpParserState::MoreDataRequired); }

		self.buffer.push_all(data);

		if self.headers_parsed == false {
			let p = self.pos;
			for i in p..self.buffer.len(){
				//println!("i = {}", i);
				let f = self.buffer[i];
				if f == '\r' as u8 {
					if i + 1 < self.buffer.len() {
						let f2 = self.buffer[i + 1];
						if f2 == '\n' as u8 {
							// line found
							let line = &self.buffer[self.pos..i];

							self.pos = i + 2;							

							if line.len() == 0 {
								self.line_num += 1;
								self.headers_parsed = true;
								break;
							}

							if self.line_num == 0 {
								match self.msg {
									HttpMessage::Request(ref mut m) => try!(HttpParser::parse_first_request_line(m, line)),
									_ => { }
								}
								
							} else {
								try!(HttpParser::parse_header_line(&mut self.msg, line));
							}

							self.line_num += 1;
						}
					}
				}
			}
		}

		if self.headers_parsed {
			{
				let s = &self.buffer[(self.pos)..];
				let body = self.msg.get_body_mut();
				body.push_all(s);
			}
			self.buffer.clear();
			self.pos = 0;
		}


		Ok(HttpParserState::MoreDataRequired)
	}

	fn parse_first_request_line(msg: &mut HttpRequestMessage, line: &[u8]) -> Result<(), HttpParserError> {
		let str = from_utf8(line);
		if !str.is_ok() { return Err(HttpParserError::InvalidString); }
		let str = str.unwrap();

		let mut middle = str;

		let http_methods = [("GET", HttpMethod::Get), ("HEAD", HttpMethod::Head), ("POST", HttpMethod::Post),
		                    ("NOTIFY", HttpMethod::Notify), ("M-SEARCH", HttpMethod::MSearch)
		                   ];
		let method = {
			let mut matched = false;
			for m in &http_methods {
				if str.starts_with(m.0) {
					msg.method = m.1;
					middle = &str[(m.0.len() + 1)..];

					matched = true;
					break;
				}
			}

			matched
		};

		if method == false {
			return Err(HttpParserError::LineParseError(str.to_string()));
		}

		if str.ends_with("HTTP/1.1") {
			msg.http_version = "1.1".to_string();
		} else if str.ends_with("HTTP/1.0") {
			msg.http_version = "1.0".to_string();
		}

		let l = middle.rfind("HTTP/1");
		if l.is_none() { return Err(HttpParserError::LineParseError(str.to_string())); }

		let url = &middle[..(l.unwrap() - 1)];
		msg.url = url.to_string();

		return Ok(());
	}

	fn parse_header_line(msg: &mut HttpMessage, line: &[u8]) -> Result<(), HttpParserError> {
		let str = from_utf8(line);
		if str.is_ok() {
			let str = str.unwrap();
			
			let sep = str.find(": ");
			if sep.is_none() {
				return Err(HttpParserError::HeaderError);
			}
			let sep = sep.unwrap();

			let key = &str[0..sep];
			let val = &str[sep + 2..];

			let headers = match *msg {
				HttpMessage::Request(ref mut r) => &mut r.headers,
				HttpMessage::Response(ref mut r) => &mut r.headers
			};
			headers.insert(key.to_string(), val.to_string());

		} else {
			return Err(HttpParserError::InvalidString);
		}

		return Ok(());
	}

	pub fn get_message(&self) -> &HttpMessage {
		&self.msg
	}

	pub fn get_request(&self) -> Option<&HttpRequestMessage> {
		match self.msg {
			HttpMessage::Request(ref r) => Some(r),
			HttpMessage::Response(_) => None	
		}
	}
}










#[cfg(test)]
mod tests {
	

	use super::*;


	use core::prelude::*;
	use std::prelude::*;
	use collections::vec::Vec;

	#[test]
	pub fn test_request_parsing() {
		let msg = "GET /index.html HTTP/1.1\r\nHost: www.example.com\r\n\r\nbody";

		let mut parser = HttpParser::new_request();
		let bytes = &msg.bytes();
		let bytes: Vec<u8> = bytes.clone().collect();
		parser.parse_bytes(&bytes).unwrap();
		/*
		for b in msg.bytes() {
			parser.parse_bytes(&[b]);
		}
		*/

		let req = parser.get_request();
		println!("parsed: {:?}", req);
	}
	

}

