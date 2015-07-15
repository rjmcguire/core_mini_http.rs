#![feature(convert)]


use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};
use std::net::Shutdown;

extern crate core_mini_http;

use core_mini_http::*;
use std::sync::Arc;


struct HttpServer {
    routes: Vec<Box<HttpRoute + Send + Sync + 'static>>
}

impl HttpServer {
    fn handle_client(&self, stream: TcpStream) {
        let mut stream = stream;

        let mut parser = HttpParser::new_request();

        loop {
            let mut buf = [0; 1024];
            let r = stream.read(&mut buf);
            if !r.is_ok() {
                panic!("stream broken");
            }

            let read_bytes = r.unwrap();
            if read_bytes == 0 {
                println!("stream endeth");
                break;
            }

            let parsed = parser.parse_bytes(&buf[..read_bytes]);
            if !parsed.is_ok() {
                panic!("parser borked");
            }

            if parser.read_how_many_bytes() == 0 {
                break;
            }
        }

        println!("{:?}", parser.get_request());

        stream.shutdown(Shutdown::Read);

        let res = http_router(&self.routes, &parser.get_request().unwrap());
        if res.is_ok() {
            let resp = res.unwrap().execute(&parser.get_request().unwrap());
            if resp.is_ok() {
                let resp = resp.unwrap();
                stream.write(&resp.to_bytes());
                stream.flush();
                stream.shutdown(Shutdown::Write);
                return;
            }
        }

        stream.shutdown(Shutdown::Write);
    }
}


fn main() {
    let listener = TcpListener::bind("127.0.0.1:8088").unwrap();
    let server = HttpServer {
        routes: vec![
            Box::new(HttpRouteStaticUrl::new_get("/", |req| {
                HttpResponseMessage::html_utf8("<h1>Hello World!</h1><form method='post' action='/form'><p>ssid: <input type='text' name='ssid' value='' /></p><p><input type='submit' name='submit' value='Connect' /></p></form>")
            })),


            Box::new(
                HttpRouteStaticUrl {
                    urls: vec!["/form".to_string()],
                    methods: vec![HttpMethod::Post],
                    action: Box::new(|req| {
                        let mut msg = "<h1>Response from the FORM!</h1>".to_string();

                        if req.content_type() == HttpContentType::UrlEncodedForm {
                            let p = BodyFormParser::parse(&req);
                            
                            if p.contains_key("ssid") {
                                msg = format!("<p>SSID: <b>{}</b></p>", p.get("ssid").unwrap());
                            }
                        }

                        HttpResponseMessage::html_utf8(&msg)
                    })
                }
            ),

            Box::new(HttpRouteDynamicUrl::new(DynamicUrl::parse_str("/test/:id/").unwrap(), HttpMethod::Get, |req, vars| {
                HttpResponseMessage::html_utf8(format!("<h1>Hello World!</h1><p>ID: <b>{}</b></p>", vars.get("id").unwrap()).as_str())
            })),

        ]
    };
    let server = Arc::new(server);


    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        let server = server.clone();
        match stream {
            Ok(stream) => {
                thread::spawn(move|| {
                    // connection succeeded
                    //handle_client(stream);                    
                    server.handle_client(stream);
                });
            }
            Err(e) => { /* connection failed */ }
        }
    }

    // close the socket server
    drop(listener);
}