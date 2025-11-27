mod client;
mod server;

// HTTP 1.1

// Ex:
// GET HTTP/1.1 200 OK\r\n
// Content-Type: text/html\r\n
// Content-Length: 13\r\n
// \r\n
// Hello, World!

// me:
// 1. split the headers using `\r\n\r\n`. We do this because this is the
// separator that splits the header and body
// 2. now we have the headers and the body
// 3. the first line has method, http version status code and path
// 4. split first line to get all parts
// 5. parse the remaining lines as the headers and push into an array, by name and value

// chunked
// {size_in_hex}\r\n
// {data}\r\n

// 0\r\n
// \r\n
// 1. set header to support transfer-encoding as chunked
// 2. parser now checks for chunked stream and builds body
// 3. parse chunked_body
// 4. find chunksize based on hex, convert to decimal and get all data after set data size
// 5. find first size using \r\n, convert to decimal and parse from first size -> decimal

struct HttpRequest {
    method: String,                 // GET, POST
    path: String,                   // /index.html
    version: String,                // HTTP 1.1
    headers: Vec<(String, String)>, // List of name, value
    body: String,                   // Request body
}

#[derive(Debug)]
enum ParseError {
    MissingRequestLine,
    MissingMethod,
    MissingPath,
    MissingVersion,
    ContentLengthMismatch { expected: usize, actual: usize },
    InvalidChunkSize,
    InvalidChunkData,
}

fn parse_chunked_body(raw_body: &str) -> Result<String, ParseError> {
    let mut result = String::new();
    let mut remaining = raw_body;

    loop {
        let size_end = raw_body.find("\r\n").ok_or(ParseError::InvalidChunkSize)?;
        let size_hex = &remaining[..size_end];

        let chunk_size =
            usize::from_str_radix(size_hex, 16).map_err(|_| ParseError::InvalidChunkSize)?;

        if chunk_size == 0 {
            break;
        }

        remaining = &remaining[size_end + 2..];

        if remaining.len() < chunk_size {
            return Err(ParseError::InvalidChunkData);
        }

        let chunk_data = &remaining[..chunk_size];
        result.push_str(chunk_data);

        remaining = &remaining[chunk_size + 2..]
    }
    Ok(result)
}

fn parse_request(raw: &str) -> Result<HttpRequest, ParseError> {
    // 1. Split headers from body
    let parts: Vec<&str> = raw.splitn(2, "\r\n\r\n").collect();

    let request = parts.get(0).ok_or(ParseError::MissingRequestLine)?;
    let raw_body = parts.get(1).unwrap_or(&"").to_string();

    // 2. Split headers into lines, generator
    let mut lines = request.lines();

    // 3. Parse first line
    let request_line = lines.next().ok_or(ParseError::MissingRequestLine)?;
    let mut request_line_parts = request_line.split_whitespace();

    let method = request_line_parts
        .next()
        .ok_or(ParseError::MissingMethod)?
        .to_string(); // GET
    let path = request_line_parts
        .next()
        .ok_or(ParseError::MissingPath)?
        .to_string(); // "/""
    let version = request_line_parts
        .next()
        .ok_or(ParseError::MissingVersion)?
        .to_string(); // HTTP/1.1

    // 4. Parse remaining headers
    let mut headers = Vec::new();
    let mut is_chunked = false;
    let mut content_length: Option<usize> = None;

    for line in lines {
        if let Some((name, value)) = line.split_once(": ") {
            if name == "Content-Length" {
                content_length = value.parse::<usize>().ok();
            } else if name == "Transfer-Encoding" && value == "chunked" {
                is_chunked = true;
            }
            headers.push((name.to_string(), value.to_string()));
        }
    }

    let body = if is_chunked {
        parse_chunked_body(&raw_body)?
    } else if let Some(length) = content_length {
        if length != raw_body.len() {
            return Err(ParseError::ContentLengthMismatch {
                expected: length,
                actual: raw_body.len(),
            });
        }
        raw_body
    } else {
        raw_body
    };

    Ok(HttpRequest {
        method,
        path,
        version,
        headers,
        body,
    })
}

struct HttpResponse {
    version: String,
    status_code: u16,
    status_text: String,
    headers: Vec<(String, String)>,
    body: String,
}
fn build_response(status_code: u16, status_text: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}",
        status_code,
        status_text,
        body.len(),
        body
    )
}

fn main() {
    println!("=== HTTP/1.1 Parser Demo ===\n");

    // Test 1: Regular request with Content-Length
    println!("--- Content-Length Request ---");
    let raw_post = "POST /data HTTP/1.1\r\nHost: example.com\r\nContent-Length: 5\r\n\r\nHello";

    match parse_request(raw_post) {
        Ok(req) => println!("Body: '{}'\n", req.body),
        Err(e) => println!("Error: {:?}\n", e),
    }

    // Test 2: Chunked request
    println!("--- Chunked Request ---");
    let raw_chunked = "POST /data HTTP/1.1\r\nHost: example.com\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nHello\r\n1\r\n \r\n6\r\nWorld!\r\n0\r\n\r\n";

    match parse_request(raw_chunked) {
        Ok(req) => println!("Body: '{}'\n", req.body),
        Err(e) => println!("Error: {:?}\n", e),
    }

    // Test 3: Build a chunked response
    // println!("--- Build Chunked Response ---");
    // let response = build_chunked_response(200, "OK", &["Hello", " ", "World!"]);
    // println!("{}", response);
}