use std::net::{TcpListener};
use std::io::{Read, Write};

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
                            if let Err(e) = stream.write(b"Response") {
                                eprintln!("Write error: {}", e);
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