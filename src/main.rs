use std::time::Duration;
use std::net::TcpStream;
use std::env;
use std::fs::File;
use std::net::TcpListener;
use std::io::BufReader;
use std::io::prelude::*;
use std::sync::mpsc::{channel, Sender, Receiver};

fn main() {
    let binding_addr = get_bind_addr();
    let target_list = read_target_list_to_connect_to("config1.txt".to_string());

    println!("Server {:?}", binding_addr);
    println!("Target list: {:?}", target_list);

    let listener = TcpListener::bind(binding_addr.to_string()).unwrap();

    println!("Server is started");
    println!("You can try to connect to the server using telnet");

    let mut tx_list = Vec::<Sender<String>>::new();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("Connection established!");
        let (tx, rx) = channel();
        let x = tx_list;

        std::thread::spawn(move || {
            handle_connection(&stream, &rx, &tx_list);
        });
    }
}

fn handle_connection(mut stream: &TcpStream, mut rx: &Receiver<String>, tx_list: &Vec<Sender<String>>) {
    let mut buffer = [0; 1024];
    println!("We are inside the handle_connection function");
    let timeout_duration = Duration::from_millis(200);

    stream.set_read_timeout(Some(timeout_duration));
    
    loop {
        
        match stream.read(&mut buffer) {
            |Ok(size) => {
                if size == 0 {
                    println!("Client disconnected");
                    break;
                }

                let data = rx.recv_timeout(timeout_duration);
                match data {
                    |Ok(s) => {
                        stream.write(s.as_bytes()).unwrap();
                    }
                    |Err(_) => {
                        println!("Channel closed!?");
                    }
                }

                let mut answer = String::new();
                answer.push_str("> ");

                let my_str = String::from_utf8_lossy(&buffer[..]);
                println!("Request: {}", my_str);
                answer.push_str(&my_str);

                stream.write(answer.as_bytes()).unwrap();
            }
            |Err(e) => {
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

// // use mini_redis::{Connection, Frame};
// use mini_redis::{client, Result};
// use tokio::net::{TcpListener, TcpStream};
// use tokio::io::{self, BufReader, AsyncBufReadExt, AsyncWriteExt};
// use tokio::fs::File;
// use tokio::sync::broadcast;


// #[tokio::main]
// async fn main() {
//     // Bind the listener to the address
//     let target_binding = read_ip_port_to_bind_to("config1.txt".to_string()).await.unwrap();
//     println!("{}", target_binding);

//     let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

//     // let target_list = read_target_list_to_connect_to("config1.txt".to_string()).await.unwrap();

//     // perform_connect(target_list, &target_binding).await.unwrap();

//     let (tx, _rx) = broadcast::channel(10);

//     loop {
//         // The second item contains the ip and port of the new connection.
//         let (mut socket, addr) = listener.accept().await.unwrap();

//         let tx = tx.clone();
//         let mut rx = tx.subscribe();

//         // A new task is spawned for each inbound socket.  The socket is
//         // moved to the new task and processed there.
//         tokio::spawn(async move {
//             let (reader, mut writer) = socket.split();
            
//             let mut reader = BufReader::new(reader);
//             let mut line = String::new();

//             loop {
//                 tokio::select! {
//                     result = reader.read_line(&mut line) => {
//                         if result.unwrap() == 0 {
//                             break;
//                         }

//                         tx.send((line.clone(), addr)).unwrap();
//                         line.clear();
//                     }   

//                     result = rx.recv() => {
//                         let (msg, other_addr) = result.unwrap();

//                         if addr != other_addr {
//                             writer.write_all(msg.as_bytes()).await.unwrap();
//                         }
//                     }
//                 }
//             }

//         });
//     }
// }

// async fn read_ip_port_to_bind_to(filename: String) -> io::Result<String> {
//     let f = File::open(filename).await?; 
//     let mut reader = BufReader::new(f);
//     let mut buffer = String::new();

//     reader.read_line(&mut buffer).await?;
//     println!("{}", buffer);

//     Ok(buffer)
// }

// async fn read_target_list_to_connect_to(filename: String) -> io::Result<Vec<String>> {
//     let f = File::open(filename).await?; 
//     let mut reader = BufReader::new(f);
//     let mut buffer = String::new();
//     let mut list = Vec::<String>::new();

//     let mut lines = reader.lines();

//     while let Some(line) = lines.next_line().await? {
//         // And print out the line length and content
//         println!("{} {}", line.len(), line);
//         list.push(line)
//     }

//     println!("{:?}", list);
//     Ok(list)
// }

// async fn perform_connect(servers: Vec<String>, target: &str) -> Result<()> {
//     for server in servers {
//         if server != target {
//             let mut client = client::connect(server).await?;
//         } 
//     }

//     Ok(())
// }

