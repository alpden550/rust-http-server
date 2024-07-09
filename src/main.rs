use flate2::{write::GzEncoder, Compression};
use std::{
    env, fs,
    fs::File,
    io::{Read, Result, Write},
    net::{TcpListener, TcpStream},
    str, thread,
};

const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n\r\n";
const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 Not Found\r\n\r\n";
const CREATED_RESPONSE: &str = "HTTP/1.1 201 Created\r\n\r\n";

fn get_directory_from_args() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Not provided directory")
    }

    return args[2].to_string();
}

fn generate_filename(path: &str) -> String {
    let dir = get_directory_from_args();
    let file_name = path.replace("/files/", "");
    let file_path = format!("{dir}{file_name}").replace("//", "/");

    file_path
}

fn get_header(request: &str, pattern: &str) -> String {
    let mut header: String = String::from("");
    let mut lines = request.lines();

    loop {
        let line = lines.next().unwrap();
        if line == "" {
            break;
        }
        if line.starts_with(pattern) {
            let headers = line.split_once(" ").unwrap();
            header = String::from(headers.1);
        }
    }

    header
}

fn get_compression(request: &str) -> String {
    let compressions = get_header(request, "Accept-Encoding:");
    let mut encoding = String::from("");

    for compression in compressions.split(", ") {
        if compression == "gzip" {
            encoding.push_str("Content-Encoding: ");
            encoding.push_str("gzip");
        }
    }

    encoding
}

fn compress_body(input: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(vec![], Compression::default());
    encoder.write_all(input.as_bytes()).unwrap();
    let compressed_input = encoder.finish().unwrap();

    compressed_input
}

fn build_content_response(request: &str, content: &str, content_type: &str) -> Vec<u8> {
    let compression = get_compression(request);

    let mut response = String::from("HTTP/1.1 200 OK");
    response.push_str("\r\n");
    response.push_str(format!("Content-Type: {}", content_type).as_str());
    if compression != "" {
        let body = compress_body(content);

        response.push_str("\r\n");
        response.push_str(compression.as_str());
        response.push_str("\r\n");
        response.push_str(format!("Content-Length: {}\r\n\r\n", body.len()).as_str());

        let mut raw_response = response.clone().into_bytes();
        raw_response.extend_from_slice(&body);
        return raw_response;
    }

    response.push_str("\r\n");
    response.push_str(format!("Content-Length: {}\r\n\r\n{}", content.len(), content).as_str());

    response.into_bytes()
}

fn handle_echo(path: &str, request: &str, mut stream: TcpStream) -> Result<()> {
    let echo = path.replace("/echo/", "");
    let response = build_content_response(request, echo.as_str(), "text/plain");

    stream.write_all(&response)?;
    Ok(())
}

fn handle_user_agent(request: &str, mut stream: TcpStream) -> Result<()> {
    let user_agent = get_header(request, "User-Agent:");

    if user_agent == "" {
        stream.write_all(NOT_FOUND_RESPONSE.as_bytes())?;
    } else {
        let response = build_content_response(request, user_agent.as_str(), "text/plain");
        stream.write_all(&response)?;
    }

    Ok(())
}

fn handle_file(path: &str, request: &str, mut stream: TcpStream) -> Result<()> {
    let file_path = generate_filename(path);

    match fs::read_to_string(file_path) {
        Ok(file) => {
            let response =
                build_content_response(request, file.as_str(), "application/octet-stream");
            stream.write_all(&response)?;
        }
        Err(_) => {
            stream.write_all("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes())?;
        }
    }

    Ok(())
}

fn handle_file_post(path: &str, request: &str, mut stream: TcpStream) -> Result<()> {
    let file_path = generate_filename(path);

    let body = request.rsplit("\r\n\r\n").collect::<Vec<&str>>()[0];
    let mut file = File::create(file_path)?;
    file.write_all(body.as_bytes())?;
    stream.write_all(CREATED_RESPONSE.as_bytes())?;

    Ok(())
}

fn handle_gets(path: &str, request: &str, mut stream: TcpStream) -> Result<()> {
    if path == "/" {
        stream.write_all(OK_RESPONSE.as_bytes())?;
    } else if path.starts_with("/echo/") {
        handle_echo(path, request, stream)?;
    } else if path == "/user-agent" {
        handle_user_agent(request, stream)?;
    } else if path.starts_with("/files/") {
        handle_file(path, request, stream)?;
    } else {
        stream.write_all(NOT_FOUND_RESPONSE.as_bytes())?;
    }

    Ok(())
}

fn handle_posts(path: &str, request: &str, stream: TcpStream) -> Result<()> {
    if path.starts_with("/files/") {
        handle_file_post(path, request, stream)?;
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buffer: [u8; 1024] = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    let request = str::from_utf8(&buffer[..bytes_read]).unwrap();

    let mut lines = request.lines();
    if let Some(first_line) = lines.next() {
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() > 2 {
            let (method, path) = (parts[0], parts[1]);

            match method {
                "GET" => {
                    handle_gets(path, request, stream)?;
                }
                "POST" => {
                    handle_posts(path, request, stream)?;
                }
                _ => {
                    stream.write_all(NOT_FOUND_RESPONSE.as_bytes())?;
                }
            }
        } else {
            stream.write_all(NOT_FOUND_RESPONSE.as_bytes())?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_connection(stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}
