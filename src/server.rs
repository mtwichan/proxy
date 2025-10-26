use std::net::{TcpListener};
use std::io::{Read, Write};
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

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
                            println!("Recieved msg: {}", String::from_utf8_lossy(&buffer[..bytes_read]));

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

                            let headers_method_content: Vec<&str> = headers_method.split_whitespace().collect();
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
                                    let response = format!("{http_version} 200 OK \r\n") + &format!("Content-Length: {}\r\n", content.len()) + &format!("\r\n") + &content;
                                    stream.write(response.as_bytes())?;
                                    continue;
                                },
                                _ => {
                                    stream.write(format!("HTTP/1.1 400 Bad Request\r\n\r\nInvalid request: invalid header {}", method).as_bytes())?;
                                    continue;
                                }
                            }                            
                        }
                        Err(e) => {eprint!("Read error: {}", e)}
                    }
                }

            }
            Err(e) => eprintln!("Connection failed: {}", e)
        }
    }
    Ok(())
}