use std::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;
use std::net::TcpStream;
use std::env;
use std::fs::File;
use std::net::TcpListener;
use std::io::BufReader;
use std::io::prelude::*;

fn main() {
    let binding_addr = get_bind_addr();
    let target_list = read_target_list_to_connect_to("config1.txt".to_string());

    println!("Server {:?}", binding_addr);
    println!("Target list: {:?}", target_list);

    let listener = TcpListener::bind(binding_addr.to_string()).unwrap();

    println!("Server is started");
    println!("You can try to connect to the server using telnet");

    let connection_list = Arc::new(Mutex::new(Vec::<TcpStream>::new()));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        (*connection_list).lock().unwrap().push(stream.try_clone().unwrap());

        println!("Connection established!");
        let clients = connection_list.clone();

        std::thread::spawn(move || {
            handle_connection(&stream, &clients);
        });
    }
}

fn handle_connection(mut stream: &TcpStream, connections: &Arc<Mutex<Vec<TcpStream>>>) {
    let mut buffer = [0; 1024];
    println!("We are inside the handle_connection function");
    let timeout_duration = Duration::from_millis(200);

    stream.set_read_timeout(Some(timeout_duration)).unwrap();
    
    loop {
        
        match stream.read(&mut buffer) {
            |Ok(size) => {
                if size == 0 {
                    println!("Client disconnected");
                    break;
                }

                let mut answer = String::new();
                answer.push_str("> ");

                let my_str = String::from_utf8_lossy(&buffer[..]);

                answer.push_str(&my_str);

                // Spread the incoming text over all clients.
                connections.lock().unwrap().iter().for_each(|mut client| {
                    client.write_all(answer.as_bytes()).unwrap();
                });
            }
            |Err(_e) => {
                // println!("Error: {}", e);
            }
        }
        
    }

    println!("Connection closed, ready for the next one");
}

fn get_bind_addr() -> String {
    let maybe_arg = env::args().nth(3);
    match maybe_arg {
        Some(arg) => format!("0.0.0.0:{:?}", arg),
        None => "0.0.0.0:7007".to_owned()
    }
}

fn read_target_list_to_connect_to(filename: String) -> Vec<String> {
    let f = File::open(filename).unwrap(); 
    let reader = BufReader::new(f);
    let mut list = Vec::<String>::new();

    for line in reader.lines() {
        match line {
            Ok(line) => list.push(line),
            Err(e) => println!("{:?}", e),
        }
    }
    list
}
