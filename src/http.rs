use core::str::from_utf8;
use collections::vec::*;
use collections::String;
use collections::string::ToString;
use collections::BTreeMap;

pub enum HttpMessage {
	Request(HttpRequestMessage),
	Response(HttpResponseMessage)
}

impl HttpMessage {
	pub fn get_body(&self) -> &Vec<u8> {
		match *self {
			HttpMessage::Request(ref r) => &r.body,
			HttpMessage::Response(ref r) => &r.body
		}		
	}

	pub fn get_body_mut(&mut self) -> &mut Vec<u8> {
		match *self {
			HttpMessage::Request(ref mut r) => &mut r.body,
			HttpMessage::Response(ref mut r) => &mut r.body
		}		
	}
}

impl HttpHeaders for HttpMessage {
	fn get_raw_headers(&self) -> &BTreeMap<String, String> {
		match *self {
			HttpMessage::Request(ref r) => &r.headers,
			HttpMessage::Response(ref r) => &r.headers
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HttpMethod {
	Get,
	Post,
	Head,
	Put,
	Delete,
	Options,
	Notify,
	MSearch,
}

impl HttpMethod {
	pub fn to_string(&self) -> String {
		match *self {
			HttpMethod::Get => "GET",
			HttpMethod::Post => "POST",
			HttpMethod::Head => "HEAD",
			HttpMethod::Put => "PUT",
			HttpMethod::Delete => "DELETE",
			HttpMethod::Options => "OPTIONS",
			HttpMethod::Notify => "NOTIFY",
			HttpMethod::MSearch => "M-SEARCH"
		}.to_string()
	}
}

#[derive(Debug, Eq, PartialEq)]
pub enum HttpContentType {
    Unknown,
    UrlEncodedForm,
}

#[derive(Debug, Eq, PartialEq)]
pub struct HttpRequestMessage {
    pub method: HttpMethod,
    pub http_version: String,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpRequestMessage {
    pub fn empty() -> HttpRequestMessage {
        HttpRequestMessage {
            method: HttpMethod::Get,
            http_version: String::new(),
            url: String::new(),
            headers: BTreeMap::new(),
            body: Vec::new()
        }
    }

	pub fn new_get(url: &str, host: &str) -> HttpRequestMessage {
		let mut headers = BTreeMap::new();
		headers.insert("Host".to_string(), host.to_string());

		HttpRequestMessage {
			method: HttpMethod::Get,
			http_version: "1.1".to_string(),
			url: url.to_string(),
			headers: headers,
			body: Vec::new()
		}
	}

	pub fn to_bytes(&self) -> Vec<u8> {
		let mut ret = Vec::new();
		
		fn output_line(r: &mut Vec<u8>, s: &str) {
			let nl = b"\r\n";
			for b in s.bytes() {
				r.push(b);
			}
			for b in nl {
				r.push(*b);
			}
		}

		output_line(&mut ret, format!("{} {} HTTP/{}", self.method.to_string(), self.url, self.http_version).as_str());		

		for (key, val) in &self.headers {
			output_line(&mut ret, format!("{}: {}", key, val).as_str());
		}

		output_line(&mut ret, "");

		ret.extend_from_slice(&self.body);

		ret
	}	
}

impl HttpHeaders for HttpRequestMessage {
    fn get_raw_headers(&self) -> &BTreeMap<String, String> {
        &self.headers
    }
}

pub trait HttpHeaders {
    fn get_raw_headers(&self) -> &BTreeMap<String, String>;
	//fn get_mut_raw_headers(&mut self) -> &mut BTreeMap<String, String>;

    fn get_raw_header(&self, key: &str) -> Option<&String> {
        let h = self.get_raw_headers();
        h.get(key)
    }

    fn content_length(&self) -> Option<u32> {
        let c = self.get_raw_header("Content-Length");
        if c.is_some() {
            let c = c.unwrap().parse::<u32>();
            if c.is_ok() {
                return Some(c.unwrap());
            }
        }

        None
    }

    fn content_type(&self) -> HttpContentType {
        let c = self.get_raw_header("Content-Type");
        if c.is_some() {
            if c.unwrap().starts_with("application/x-www-form-urlencoded") {
                return HttpContentType::UrlEncodedForm;
            }
        }

        HttpContentType::Unknown
    }
}

pub struct BodyFormParser;
impl BodyFormParser {
    pub fn parse(req: &HttpRequestMessage) -> BTreeMap<String, String> {
        let body = from_utf8(&req.body);
        if body.is_ok() {
            return parse_urlencoded_form(body.unwrap());
        }

        BTreeMap::new()
    }
}

pub fn parse_urlencoded_form(body: &str) -> BTreeMap<String, String> {
    let mut h: BTreeMap<String, String> = BTreeMap::new();

    for f in body.split("&") {
        let kv: Vec<&str> = f.split("=").collect();
        if kv.len() == 2 {
            let k = kv.get(0).unwrap();
            let v = kv.get(1).unwrap();

            let k = percent_decode_str(k);
            let v = percent_decode_str(v);

            h.insert(k, v);
        }
    }

    h
}

/// Percent-decode the given bytes, and push the result to `output`.
pub fn percent_decode_to(input: &[u8], output: &mut Vec<u8>) {
    let mut i = 0;
    while i < input.len() {
        let c = input[i];
        if c == b'%' && i + 2 < input.len() {
            if let (Some(h), Some(l)) = (from_hex(input[i + 1]), from_hex(input[i + 2])) {
                output.push(h * 0x10 + l);
                i += 3;
                continue
            }
        }

        output.push(c);
        i += 1;
    }
}


#[inline]
pub fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0' ... b'9' => Some(byte - b'0'),  // 0..9
        b'A' ... b'F' => Some(byte + 10 - b'A'),  // A..F
        b'a' ... b'f' => Some(byte + 10 - b'a'),  // a..f
        _ => None
    }
}

/// Percent-decode the given bytes.
#[inline]
pub fn percent_decode(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    percent_decode_to(input, &mut output);
    output
}

pub fn percent_decode_str(input: &str) -> String {
    let b: Vec<u8> = input.bytes().collect();
    let dec = percent_decode(&replace_plus(&b));
    String::from_utf8_lossy(&dec).to_string()
}

#[inline]
fn replace_plus(input: &[u8]) -> Vec<u8> {
    input.iter().map(|&b| if b == b'+' { b' ' } else { b }).collect()
}

/// Percent-decode the given bytes, and decode the result as UTF-8.
///
/// This is “lossy”: invalid UTF-8 percent-encoded byte sequences
/// will be replaced � U+FFFD, the replacement character.
#[inline]
pub fn lossy_utf8_percent_decode(input: &[u8]) -> String {
    String::from_utf8_lossy(&percent_decode(input)).to_string()
}

#[cfg(test)]
#[test]
fn test_form_parser() {
    let f = "ssid=test&submit=Connect";
    let p = parse_urlencoded_form(&f);
    println!("p: {:?}", p);

    let f = "ssid=rock+%26+roll&submit=Connect";
    let p = parse_urlencoded_form(&f);
    println!("p: {:?}", p);

    let f = "ssid=%26%23269%3B%9E%26%23263%3B%26%23273%3B%9A%26%23269%3B&submit=Connect";
    let p = parse_urlencoded_form(&f);
    println!("p: {:?}", p);	

}

#[derive(Debug)]
pub struct HttpResponseMessage {
    pub response_code: u16,
    pub response_status: String,
    pub http_version: String,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>
}

impl HttpHeaders for HttpResponseMessage {
	fn get_raw_headers(&self) -> &BTreeMap<String, String> {
		&self.headers
	}
}

impl HttpResponseMessage {
	pub fn empty() -> HttpResponseMessage {
		HttpResponseMessage {
			response_code: 0,
			response_status: "".to_string(),
			http_version: "".to_string(),
			headers: BTreeMap::new(),
			body: Vec::new()
		}
	}

	pub fn to_bytes(&self) -> Vec<u8> {
		let mut ret = Vec::new();
        
        fn output_line(r: &mut Vec<u8>, s: &str) {
            let nl = b"\r\n";
            for b in s.bytes() {
                r.push(b);
            }
            for b in nl {
                r.push(*b);
            }
        }

        output_line(&mut ret, &format!("HTTP/{} {} {}", self.http_version, self.response_code, self.response_status));

        for (key, val) in &self.headers {
            output_line(&mut ret, &format!("{}: {}", key, val));
        }

        output_line(&mut ret, "");

        ret.extend_from_slice(&self.body);

        ret
    }

    pub fn html_utf8(body: &str) -> HttpResponseMessage {
        let mut headers = BTreeMap::new();
        headers.insert(String::from("Content-Type"), String::from("text/html; charset=UTF-U8"));

        HttpResponseMessage {
            response_code: 200,
            response_status: String::from("OK"),
            http_version: String::from("1.1"),
            headers: headers,
            body: body.bytes().collect()
        }
    }

	pub fn text_utf8(body: &str) -> HttpResponseMessage {
		let mut headers = BTreeMap::new();
		headers.insert("Content-Type".to_string(), "text/plain; charset=UTF-8".to_string());

		HttpResponseMessage {
			response_code: 200,
			response_status: "OK".to_string(),
			http_version: "1.1".to_string(),
			headers: headers,
			body: body.bytes().collect()
		}
	}

	pub fn json_utf8(body: &str) -> HttpResponseMessage {
		let mut headers = BTreeMap::new();
		headers.insert("Content-Type".to_string(), "application/json; charset=UTF-8".to_string());

		HttpResponseMessage {
			response_code: 200,
			response_status: "OK".to_string(),
			http_version: "1.1".to_string(),
			headers: headers,
			body: body.bytes().collect()
		}
	}	
}
