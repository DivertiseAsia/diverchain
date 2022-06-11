// use mini_redis::{Connection, Frame};
use mini_redis::{client, Result};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, BufReader, AsyncBufReadExt, AsyncWriteExt};
use tokio::fs::File;
use tokio::sync::broadcast;


#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let target_binding = read_ip_port_to_bind_to("config1.txt".to_string()).await.unwrap();
    println!("{}", target_binding);

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    // let target_list = read_target_list_to_connect_to("config1.txt".to_string()).await.unwrap();

    // perform_connect(target_list, &target_binding).await.unwrap();

    let (tx, _rx) = broadcast::channel(10);

    loop {
        // The second item contains the ip and port of the new connection.
        let (mut socket, addr) = listener.accept().await.unwrap();

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        // A new task is spawned for each inbound socket.  The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }

                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }   

                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();

                        if addr != other_addr {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }

        });
    }
}

async fn read_ip_port_to_bind_to(filename: String) -> io::Result<String> {
    let f = File::open(filename).await?; 
    let mut reader = BufReader::new(f);
    let mut buffer = String::new();

    reader.read_line(&mut buffer).await?;
    println!("{}", buffer);

    Ok(buffer)
}

async fn read_target_list_to_connect_to(filename: String) -> io::Result<Vec<String>> {
    let f = File::open(filename).await?; 
    let mut reader = BufReader::new(f);
    let mut buffer = String::new();
    let mut list = Vec::<String>::new();

    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        // And print out the line length and content
        println!("{} {}", line.len(), line);
        list.push(line)
    }

    println!("{:?}", list);
    Ok(list)
}

async fn perform_connect(servers: Vec<String>, target: &str) -> Result<()> {
    for server in servers {
        if server != target {
            let mut client = client::connect(server).await?;
        } 
    }

    Ok(())
}

// async fn process(socket: TcpStream) {
//     use mini_redis::Command::{self, Get, Set};
//     use std::collections::HashMap;

//     // A hashmap is used to store data
//     let mut db = HashMap::new();

//     // Connection, provided by `mini-redis`, handles parsing frames from
//     // the socket
//     let mut connection = Connection::new(socket);

//     // Use `read_frame` to receive a command from the connection.
//     while let Some(frame) = connection.read_frame().await.unwrap() {
//         println!("GOT: {:?}", frame);

//         let response = match Command::from_frame(frame).unwrap() {
//             Set(cmd) => {
//                 // The value is stored as `Vec<u8>`
//                 db.insert(cmd.key().to_string(), cmd.value().to_vec());
//                 Frame::Simple("OK".to_string())
//             }
//             Get(cmd) => {
//                 if let Some(value) = db.get(cmd.key()) {
//                     // `Frame::Bulk` expects data to be of type `Bytes`. This
//                     // type will be covered later in the tutorial. For now,
//                     // `&Vec<u8>` is converted to `Bytes` using `into()`.
//                     Frame::Bulk(value.clone().into())
//                 } else {
//                     Frame::Null
//                 }
//             }
//             cmd => panic!("unimplemented {:?}", cmd),
//         };

//         // Write the response to the client
//         connection.write_frame(&response).await.unwrap();
//     }
// }

// use tokio::net::{TcpListener, TcpStream};
// use mini_redis::{Connection, Frame};

// #[tokio::main]
// async fn main() {
//     // Bind the listener to the address
//     let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

//     loop {
//         // The second item contains the IP and port of the new connection.
//         let (socket, _) = listener.accept().await.unwrap();
//         process(socket).await;
//     }
// }

// async fn process(socket: TcpStream) {
//     // The `Connection` lets us read/write redis **frames** instead of
//     // byte streams. The `Connection` type is defined by mini-redis.
//     let mut connection = Connection::new(socket);

//     if let Some(frame) = connection.read_frame().await.unwrap() {
//         println!("GOT: {:?}", frame);

//         // Respond with an error
//         let response = Frame::Error("unimplemented".to_string());
//         connection.write_frame(&response).await.unwrap();
//     }
// }