use core::str::from_utf8;
use http::*;
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
    Complete,
}

#[derive(Debug)]
pub enum HttpParserError {
    InvalidString,
    HeaderError,
    LineParseError(String),
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

	pub fn new_response() -> HttpParser {
		HttpParser::new(HttpMessage::Response(HttpResponseMessage::empty()))
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
            	let len = cl.unwrap();
            	let r = len as i32 - self.msg.get_body().len() as i32;
            	if r <= 0 { return 0; }
            	return r as u32;
            }
        }

        return 1;
    }

	pub fn parse_bytes(&mut self, data: &[u8]) -> Result<HttpParserState, HttpParserError> {
		if data.len() == 0 { return Ok(HttpParserState::MoreDataRequired); }

		self.buffer.extend_from_slice(data);

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
									HttpMessage::Response(ref mut m) => try!(HttpParser::parse_first_response_line(m, line))
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
				body.extend_from_slice(s);
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
		} else if str.ends_with("HTTP/1.0") || str.ends_with("HTTP/1") {
			msg.http_version = "1.0".to_string();
		}

		let l = middle.rfind("HTTP/1");
		if l.is_none() { return Err(HttpParserError::LineParseError(str.to_string())); }

        let url = &middle[..(l.unwrap() - 1)];
        msg.url = url.to_string();

        return Ok(());
    }

	fn parse_first_response_line(msg: &mut HttpResponseMessage, line: &[u8]) -> Result<(), HttpParserError> {
		let str = from_utf8(line);
		if !str.is_ok() { return Err(HttpParserError::InvalidString); }
		let str = str.unwrap();

		let split: Vec<&str> = str.splitn(3, " ").collect();
		if split.len() != 3 { return Err(HttpParserError::LineParseError(str.to_string())); }

		match split.get(0) {
			Some(&"HTTP/1.1") => msg.http_version = "1.1".to_string(),
			Some(&"HTTP/1.0") | Some(&"HTTP/1") => msg.http_version = "1.0".to_string(),
			_ => { return Err(HttpParserError::LineParseError(str.to_string())); }
		}

		match split.get(1) {
			Some(code) => {
				if let Ok(code_num) = code.parse::<u16>() {
					msg.response_code = code_num;
				} else {
					return Err(HttpParserError::LineParseError(str.to_string()));
				}
			}
			_ => { return Err(HttpParserError::LineParseError(str.to_string())); }
		}

		match split.get(2) {
			Some(status) => {
				msg.response_status = status.to_string();
			}
			_ => { return Err(HttpParserError::LineParseError(str.to_string())); }
		}

		Ok(())
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

            Ok(())
        } else {
            return Err(HttpParserError::InvalidString);
        }
    }

	pub fn get_message(&self) -> &HttpMessage {
		&self.msg
	}

	pub fn get_request(&self) -> Option<&HttpRequestMessage> {
		if let HttpMessage::Request(ref r) = self.msg {
			return Some(r);
		}
		None
	}

	pub fn into_request(self) -> Option<HttpRequestMessage> {
		if let HttpMessage::Request(r) = self.msg {
			return Some(r);
		}
		None
	}

	pub fn get_response(&self) -> Option<&HttpResponseMessage> {
		if let HttpMessage::Response(ref r) = self.msg {
			return Some(r);
		}
		None
	}

	pub fn into_response(self) -> Option<HttpResponseMessage> {
		if let HttpMessage::Response(r) = self.msg {
			return Some(r);
		}
		None
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    use collections::vec::Vec;
	use super::super::{HttpRequestMessage};

	use std::io::prelude::*;
	use std::net::TcpStream;

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

		let req = parser.get_request().unwrap();
		println!("parsed: {:?}", req);
	}


	#[test]
	pub fn test_response_parsing() {
		let msg = b"HTTP/1.1 200 OK\r\n\
Date: Mon, 23 May 2005 22:38:34 GMT\r\n\
Server: Apache/1.3.3.7 (Unix) (Red-Hat/Linux)\r\n\
Last-Modified: Wed, 08 Jan 2003 23:11:55 GMT\r\n\
ETag: \"3f80f-1b6-3e1cb03b\"\r\n\
Content-Type: text/html; charset=UTF-8\r\n\
Content-Length: 138\r\n\
Accept-Ranges: bytes\r\n\
Connection: close\r\n\
\r\n\
<html>\r\n\
<head>\r\n\
  <title>An Example Page</title>\r\n\
</head>\r\n\
<body>\r\n\
  Hello World, this is a very simple HTML document.\r\n\
</body>\r\n\
</html>";

		let mut parser = HttpParser::new_response();
		parser.parse_bytes(msg).unwrap();

		let resp = parser.get_response().unwrap();
		println!("parsed: {:?}", resp);


	}


	#[test]
	pub fn test_http_client() {		
	    let mut stream = TcpStream::connect("clients3.google.com:80").unwrap();

	    let request = HttpRequestMessage::new_get("/generate_204", "clients3.google.com");

	    let _ = stream.write(&request.to_bytes());
	    let mut response_parser = HttpParser::new_response();
	    loop {
	    	let mut buf = [0; 4096];
	    	let r = stream.read(&mut buf);
	    	if !r.is_ok() {
	    		panic!("err")
	    	}
	    	let read_bytes = r.unwrap();
	    	if read_bytes == 0 {
	    		break;
	    	}

	    	let parsed = response_parser.parse_bytes(&buf[..read_bytes]);
	    	if !parsed.is_ok() {
	    		panic!("parser borked");
	    	}

	    	if response_parser.read_how_many_bytes() == 0 {
	    		break;
	    	}
	    }

	    println!("response: {:?}", response_parser.get_response().unwrap());
	}
}
