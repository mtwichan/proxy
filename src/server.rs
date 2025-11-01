use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io::{Read, Write};
use std::net::TcpListener;

enum ParseState {
    ReadingHeaders,
    ReadingBody { expected: usize },
    Complete,
}
struct HttpRequest {
    buffer: Vec<u8>,
    state: ParseState,
    headers_end_pos: Option<usize>,
    headers: String,
    body: String
}

impl HttpRequest {
    fn new() -> Self {
        HttpRequest {
            buffer: Vec::new(),
            state: ParseState::ReadingHeaders,
            headers_end_pos: None,
            headers: String::new(),
            body: String::new()
        }
    }

    fn add_data(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }

    fn parse_content_length(headers: &str) -> Option<usize> {
        headers
            .lines()
            .find(|line| line.starts_with("Content-Length"))
            .and_then(|line| line.split_once(": "))
            .and_then(|(_, value)| value.parse::<usize>().ok())
    }

    fn reset(&mut self) {
        self.state = ParseState::ReadingHeaders;
        self.buffer.clear();
        self.headers_end_pos = None;
        self.headers = String::new();
        self.body = String::new();
    }

    fn is_ready(&self) -> bool {
        matches!(self.state, ParseState::Complete)
    }

    fn parse_buffer(&mut self) {
        match self.state {
            ParseState::ReadingHeaders => {
                if let Some(pos) = HttpRequest::find_subsequence(&self.buffer, b"\r\n\r\n") {
                    self.headers_end_pos = Some(pos);

                    let headers = &self.buffer[..pos];
                    if let Ok(headers_str) = std::str::from_utf8(headers) {
                        self.headers = headers_str.to_string();

                        if let Some(content_len) = HttpRequest::parse_content_length(&headers_str) {
                            self.state = ParseState::ReadingBody {
                                expected: content_len,
                            }
                        } else {
                            self.state = ParseState::Complete;
                        }
                    }
                }
            }
            ParseState::ReadingBody { expected } => {
                if let Some(headers_end) = self.headers_end_pos {
                    let start_pos = headers_end + 4; // end + \r\n\r\n
                    let end_pos = start_pos + expected;

                    // Body exists since buffer is larger than end position of content
                    if self.buffer.len() >= end_pos {
                        let body = &self.buffer[start_pos..end_pos];
                        if let Ok(body_str) = std::str::from_utf8(body) {
                            self.body = body_str.to_string();
                            println!("Body: {}", body_str);
                        }
                        self.state = ParseState::Complete;                        
                    }
                }
            }            
            ParseState::Complete => {
                // Ready
            }
        }
    }
}
/**
 * Http Parser class
 * - buffer
 * - watches for end of header \n\n\
 * - consistently ingesting buffer
 * - look for content-length -> parse remaining bytes to get body
 * - tracks state of parsing
 * - 
 * **/
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                loop {
                    let mut buffer = [0; 1024];

                    match stream.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(bytes_read) => {
                            println!("Read {} bytes:", bytes_read);
                            println!(
                                "Recieved msg: {}",
                                String::from_utf8_lossy(&buffer[..bytes_read])
                            );

                            let msg = String::from_utf8_lossy(&buffer[..bytes_read]);

                            let header_req = match msg.split("\r\n\r\n").next() {
                                Some(h) => h,
                                None => {
                                    stream.write(b"HTTP/1.1 400 Bad Request\r\n\r\nInvalid request: missing headers")?;
                                    continue;
                                }
                            };

                            let headers_content: Vec<&str> = header_req.split("\r\n").collect();

                            let headers_method = headers_content[0];
                            let headers_host_content = headers_content[1];
                            let headers_user_agent_content = headers_content[2];

                            let headers_method_content: Vec<&str> =
                                headers_method.split_whitespace().collect();
                            if headers_method_content.len() < 3 {
                                stream.write(b"HTTP/1.1 400 Bad Request\r\n\r\nInvalid request: missing headers")?;
                                continue;
                            }

                            let method = headers_method_content[0];
                            let path = headers_method_content[1];
                            let http_version = headers_method_content[2];

                            match method {
                                "GET" => {
                                    let random_state = RandomState::new();
                                    let mut hasher = random_state.build_hasher();
                                    hasher.write_u64(0);
                                    let random_value = hasher.finish();

                                    let content = format!("GET Response: {}", random_value);
                                    let response = format!("{http_version} 200 OK \r\n")
                                        + &format!("Content-Length: {}\r\n", content.len())
                                        + &format!("\r\n")
                                        + &content;
                                    stream.write(response.as_bytes())?;
                                    continue;
                                }
                                _ => {
                                    stream.write(format!("HTTP/1.1 400 Bad Request\r\n\r\nInvalid request: invalid header {}", method).as_bytes())?;
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            eprint!("Read error: {}", e)
                        }
                    }
                }
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    Ok(())
}
