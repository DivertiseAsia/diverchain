use std::net::TcpListener;
use std::net::TcpStream;
use std::io::{Read,Write};
use std::thread;
fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";

    let (status_line, contents) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "Hello world")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "What are you looking for?")
    };

    let response = format!(
      "{}\r\nContent-Length: {}\r\n\r\n{}",
      status_line,
      contents.len(),
      contents
    );

    stream.write(response.as_bytes()).unwrap();
}

pub fn start_server() {

    println!("Spinning up HTTP server...\n");
    thread::spawn(move || {
      let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

      for stream in listener.incoming() {
          let stream = stream.unwrap();

          thread::spawn(|| {
              handle_connection(stream);
          });
      }
    }
  );
}
