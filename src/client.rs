use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread::{sleep};
use std::time;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    stream.write(&b"Hello abc 123".repeat(1005))?;

    let ms = time::Duration::from_millis(5000);
    sleep(ms);
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;  
    println!("Bytes read: {}", bytes_read);
    println!("Data: {:?}", &buffer[..bytes_read]);
    println!("Response from server: {}", String::from_utf8_lossy(&buffer[..bytes_read]));
    Ok(())
}


