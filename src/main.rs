use std::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;
use std::net::{Shutdown, TcpStream};
use std::env;
use std::fs::File;
use std::net::TcpListener;
use std::io::BufReader;
use std::io::prelude::*;
use std::collections::HashMap;


#[derive(Clone)]
struct Task {
    id: String,
    name: String,
    detail: String,
    duedate: String,
    owner: String,
    total_vote: i32,
}

struct MapContainer {
    connections: HashMap<String, TcpStream>,
    tasks: HashMap<String, Task>, 
}

impl Clone for MapContainer {
    fn clone(&self) -> MapContainer {
        MapContainer { 
            connections: self.connections.iter().map(
                |(key, value)| (key.clone(), value.try_clone().unwrap())
            ).collect(),
            tasks: self.tasks.clone(), 
        }
    }
}

fn main() {
    let binding_addr = get_bind_addr();
    let target_list = read_target_list_to_connect_to("config1.txt".to_string());

    println!("Server {:?}", binding_addr);
    println!("Target list: {:?}", target_list);

    let listener = TcpListener::bind(binding_addr.to_string()).unwrap();

    println!("Server is started");
    println!("You can try to connect to the server using telnet");

    let connection_map = HashMap::<String, TcpStream>::new();
    let task_map = HashMap::<String, Task>::new();
    // let mut count = 0;

    let maps = Arc::new(Mutex::new(MapContainer {
        connections: connection_map,
        tasks: task_map,
    }));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("{:?}", stream);
        
        // let the_hashmap = (*maps).lock().unwrap();
        let map = maps.clone();

        // let mut client_id = String::new();
        // client_id.push_str("Client_");
        // client_id.push_str(&count.to_string());

        // the_hashmap.insert(client_id, stream.try_clone().unwrap());

        // count += 1;

        println!("Connection established!");
        // let container_clone = maps.clone();

        std::thread::spawn(move || {
            handle_connection(&stream, map);
        });
    }
}


// REGISTER <client_id>
// SEND <client_id> <message>
// BROADCAST <message>
fn command_parser(stream: &TcpStream, arguments: String, container: Arc<Mutex<MapContainer>>){
    let mut locked_container = (*container).lock().unwrap();
    let connections = &mut locked_container.connections;

    let mut words: Vec<&str> = arguments.split(' ').collect();
    println!("{:?}", words);

    let mut payload_list = words.split_off(1); // split into command and payload
    
    match words[0] {
        "EXIT" => {
            std::process::exit(0);
        }

        "TASKLIST" => {

        }

        "LIST" => {
            let client_ids = connections.keys();

            // needs formatting
            println!("{:?}", connections.keys());
        }

        "REGISTER" => {
            let client_id = payload_list[0].trim().to_string();
            println!("{}", client_id);

            connections.insert(client_id, stream.try_clone().unwrap());
            println!("{:?}", connections);
        }

        "SEND" => {
            let message = String::from_iter(payload_list.split_off(1)); 
            //payload list is now a length 1 list containing client_id
            let target_client = payload_list[0];
            println!("{}", target_client);

            let client = connections.get(target_client); 
            println!("{:?}", connections.keys());

            client.unwrap().write_all(message.as_bytes()).unwrap();
        }

        "BROADCAST" => {
            // send data to all clients that are connected
            let payload = String::from_iter(payload_list);

            connections.iter().for_each(|(_id, mut client)| {
                client.write_all(payload.as_bytes()).unwrap();
            });
        }

        "RENAME" => {
            let old_id = payload_list[0].trim().to_string();
            let new_id = payload_list[1].trim().to_string();
            let old_stream = connections.remove(&old_id).unwrap();
            // println!("{}", client_id);
            
            connections.insert(new_id, old_stream);
            println!("{:?}", connections.keys());
        }

        "KICK" => {
            let old_id = payload_list[0].trim().to_string();
            let old_stream = connections.remove(&old_id).unwrap();
            // println!("{}", client_id);
            
            old_stream.shutdown(Shutdown::Both).expect("shutdown call failed");
            println!("{:?}", connections.keys());
        }

        &_ => {
            println!("Invalid command");
        }
    }
}

fn handle_connection(mut stream: &TcpStream, connections: Arc<Mutex<MapContainer>>) {
    let mut buffer = [0; 1024];
    let timeout_duration = Duration::from_millis(200);

    stream.set_read_timeout(Some(timeout_duration)).unwrap();
    
    loop {
        match stream.read(&mut buffer) {
            |Ok(0) => {
                println!("Client disconnected");
                break;
            },
            |Ok(size) => {
                let my_str = String::from_utf8_lossy(&buffer[..size]);
                let mut answer = String::new();
                answer.push_str("> ");
                answer.push_str(&my_str);
                
                let clients = connections.clone();
                command_parser(stream, my_str.to_string(), clients);
                // Spread the incoming text over all clients.
            }
            |Err(_e) => {
                // println!("Error: {}", e);    // This gonna occur if the client doesn't send in data.
            }
        }        
    }

    println!("Connection closed");
}

fn get_bind_addr() -> String {
    let maybe_arg = env::args().nth(1);
    println!("{:?}", maybe_arg);
    match maybe_arg {
        Some(arg) => format!("0.0.0.0:{}", arg),
        None => "0.0.0.0:7007".to_owned(),
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
