use std::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;
use std::fs::File;
use std::env;
use std::io::{BufReader, BufWriter};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::net::{SocketAddr};
// use stdweb::web::{Date};
use nanoid::nanoid;
extern crate timer;
extern crate chrono;
// extern crate stdweb;
mod task;
mod httpserver;
mod crypto;
use crate::task::*;
use std::str::FromStr;

// pub mod schema;

// use schema::todos;

// #[macro_use]
// extern crate diesel;
// extern crate dotenv;

// use diesel::pg::PgConnection;
// use crate::diesel::Connection;
// use std::env;
// use dotenv::dotenv;


fn main() {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    println!("{:?}", since_the_epoch);

    let binding_addr = get_bind_addr();
    let listener = std::net::TcpListener::bind(binding_addr.to_string()).unwrap();

    println!("Server is started");
    println!("You can try to connect to the server using telnet");

    println!("Server {:?}", binding_addr);

    let connection_map = HashMap::<String, std::net::TcpStream>::new();
    let task_map = HashMap::<String, Task>::new();
    let server_map = HashMap::<String, std::net::TcpStream>::new();

    let maps = Arc::new(Mutex::new(MapContainer {
        connections: connection_map,
        tasks: task_map,
        servers: server_map,
    }));

    //Scheduling repeating task
    let mapclone = maps.clone();
    let timer = timer::Timer::new();
    
    httpserver::start_server(maps.clone());

    //read from db upon startup
    read_tasks_from_db(maps.clone());


    // dotenv().ok();
    
    // if let Ok(url) = env::var("DATABASE_URL") {
    //     let conn = PgConnection::establish(&url)
    //         .expect("could not connect to database");
    // } else {
    //     println!("No DATABASE_URL set")
    // }

    let _guard = timer.schedule_repeating(chrono::Duration::seconds(10), move || {
        let mapcloneagain = mapclone.clone();

        println!("Scheduling repeating task: uplink check");

        let uplinking = false;
        // if there exist cpnnectopn then skip
        if !uplinking {
            std::thread::spawn(move || {
                let target_list = read_target_list_to_connect_to("config1.txt".to_string());
                println!("Target list: {:?}", target_list);

                write_tasks_to_db(&mapcloneagain);

                uplink(&target_list, mapcloneagain);
            });
        }
        
    });

    println!("TaskServer: Listening...");
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("Stream {:?}", stream);

        let map = maps.clone();

        println!("Connection established!");

        std::thread::spawn(move || {
            handle_connection(&stream, map);
        });
    }
}

// fn query_todos(conn: &PgConnection) {
//     let rows = todos::table
//         .limit(10)
//         .load::<Todo>(conn)
//         .expect("could not load todos");
//     for row in rows {
//         println!("{:?}", row);
//     }
// }

fn write_tasks_to_db(map: &Arc<Mutex<MapContainer>>) {
    let mut locked_container = (*map).lock().unwrap();
    let tasks_map = &mut locked_container.tasks;

    let file = File::create("db.txt");

    match file {
        | Ok(database_file) => {
            // delete everything in file
            database_file.set_len(0);

            println!("Everything deleted");

            let mut writer = BufWriter::new(database_file);

            for (task_id, task_item) in tasks_map.iter() {
                println!("{}", task_id);
                let serialized_task = serde_json::to_string(&task_item).unwrap();
                let write_string = task_id.to_string() + " " + &serialized_task + "\n";
                
                writer.write_all(write_string.as_bytes()).expect("unable to write data");
            }

            writer.flush().unwrap();
        }   

        | Err(e) => {
            println!("{}", e);
        }
    }
    
}

fn read_tasks_from_db(map: Arc<Mutex<MapContainer>>) {
    let mut locked_container = (*map).lock().unwrap();
    let tasks_map = &mut locked_container.tasks;

    let file = File::open("db.txt");
    
    match file {
        | Ok(database_file) => {

            let reader = BufReader::new(database_file); 

            for line in reader.lines() {
                match line {
                    | Ok(line) => {
                        let split_line: Option<(&str, &str)> = line.split_once(' ');

                            match split_line {
                                | Some((str1, str2)) => {
                                    let task: Task = serde_json::from_str(str2).unwrap();

                                    tasks_map.insert(str1.to_string(), task);
                                }

                                | None => {
                                    println!("Error in formatting of task in database")
                                }
                            }
                    }

                    | Err(e) => println!("{:?}", e),
                }
            }


            for (key, value) in tasks_map.iter() {
                println!("{}: {:?}", key, value);
            }
        }   

        | Err(e) => {
            println!("{}", e);
        }
    }
    
}

fn uplink(target_list: &Vec<String>, map: Arc<Mutex<MapContainer>>) {
    // let uplinking = true;
    for server in target_list {
        println!("connecting to {:?}", server);
        let mut locked_container = (*map).lock().unwrap();
        println!("MUTEX PASSED");
        let server_map = &mut locked_container.servers;

        // let mut server_map = HashMap::<String, std::net::TcpStream>::new();
        let mut found = false; 
        let mut server_keys: Vec<String> = Vec::new();
        
        for (stream_key, stream_obj) in server_map.iter() {
            println!("checking server map {:?}", stream_key);
            // randomly errors
            // chek if connection still ongoing
            let stream_status = &stream_obj.peer_addr(); 

            match stream_status {
                | Ok(stream_ip) => {    
                    println!("{}", &stream_ip.to_string());

                    let list_ip: String = String::from(stream_ip.to_string());
                    let map_ip: String = String::from(server);

                    if list_ip.eq(&map_ip) {
                        found = true; 
                    }
                }, 
                | _error => {
                    server_keys.push(stream_key.to_string());
                    println!("Peer disconnected!: Add to garbage : {:?}", stream_key);
                }
            }
        }

        for key in server_keys.iter() {
            println!("deleting server {:?}", key);
            let _ = server_map.remove(key);
        }

        println!("FOUND: {:?}", found);

        if !found {
            println!("not found? attempt to connect to {:?}", server);  
            let socket = SocketAddr::from_str(server).unwrap();
            // let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
            let time = Duration::from_millis(1000);

            let stream_attempt = std::net::TcpStream::connect_timeout(&socket, time);
            // println!("passed attempt");

            match stream_attempt {
                | Ok(mut stream) => {
                    let mut server_id = nanoid!();

                    loop {
                        if !server_map.contains_key(&server_id) {
                            break;
                        }

                        server_id = nanoid!();
                    }

                    server_map.insert(server_id, stream.try_clone().unwrap());

                    let clonemap = map.clone();

                    // stream.write_all("SERVERREG doggo_server idk".as_bytes());

                    std::thread::spawn(move || {
                        handle_connection(&stream, clonemap);
                    });
                    
                    println!("Successfully connected");
                },
                | _error => {
                    println!("Could not connect to server");
                }
            }
        }
    }
    // let uplinking = false;
}

fn parse(mut incoming_cmd: Vec<&str>) -> String {
    match incoming_cmd[0].trim() {
        "RELAY" => {
            let new_list = incoming_cmd.split_off(2);
            return parse(new_list);
        } 

        _ => {
            let mut returned_string = incoming_cmd.iter().fold("".to_string(), |acc, x| acc + &" ".to_string() + x);
            let without_space = returned_string.split_off(1);

            return without_space.to_string();
        }
    }
}

fn relayer(own_server_ip: String, servers: &HashMap<String, std::net::TcpStream>, arguments: String) {
    for (_serv_id, mut map_stream) in servers.iter() {
        let remote_server_ip = map_stream.peer_addr().unwrap().to_string();

        if !arguments.contains(&remote_server_ip) {
            println!("looping");
            let new_cmd = "RELAY ".to_owned() + &own_server_ip + " " + &arguments;
            stream_handler(map_stream.write_all(new_cmd.as_bytes())); 
        }
    }
}

fn get_client_id(command: &str) -> String {
    let args: Vec<&str> = command.split(' ').collect();
    // println!("{:?}", args);
    match args[0] {
        "SEND" => {
            return args[1].to_string();  
        }

        "RENAME" => {
            return args[1].to_string();  
        }

        "KICK" => {
            return args[1].to_string();  
        }

        _ => {
            return "NOT RELAYABLE".to_string();
        }
    }
}

// fn is_server_not_in_relay(original_cmd) {
//     return original_cmd.contains("")
// }

// assume: incoming_cmd, my_server_id
// fn relay(original_cmd, servers) {
//     for (server_id, stream) in servers.iter() {

//       if original_cmd.contains(server_id) {
//         send("RELAY "+my_server_id+" "+original_cmd)
//       }
//     }
// }

fn relay_parser(mut stream: &std::net::TcpStream, arguments: String, container: Arc<Mutex<MapContainer>>) {
    let mut locked_container = (*container).lock().unwrap();
    let mut words: Vec<&str> = arguments.trim().split(' ').collect();
    let original_cmd = words.clone();
    let mut payload_list = words.split_off(1); // split into command and payload
    let own_server_ip = stream.local_addr().unwrap().to_string();
    

    match words[0].trim() {
        "RELAY" => {
            let mut cloned = locked_container.clone();
            let servers = &locked_container.servers;
            let connections = &locked_container.connections;

            let parsed_command = parse(original_cmd);

            let client_id = get_client_id(&parsed_command);
            

            println!("{}", client_id);

            if connections.contains_key(&client_id) {
                // actually process the command
                println!("{}", parsed_command);
                command_parser(connections.get(&client_id).unwrap(), parsed_command, &mut cloned);

            } else {
                // loop through all servers and tell them to RELAY to the servers they are connected to
                println!("Doesn't contain client_id");

                for (key, value) in servers.iter() {
                    println!("{}, {:?}", key, value);
                }

                for (key, value) in connections.iter() {
                    println!("{}, {:?}", key, value);
                }

                relayer(own_server_ip, servers, arguments);
                
                // for (_serv_id, mut map_stream) in servers.iter() {
                //     let remote_server_ip = map_stream.peer_addr().unwrap().to_string();

                //     if !arguments.contains(&remote_server_ip) {
                //         println!("looping");
                //         let new_cmd = "RELAY ".to_owned() + &own_server_ip + " " + &arguments;
                //         stream_handler(map_stream.write_all(new_cmd.as_bytes())); 
                //     }
                // }
            }
        }

        "SEND" => { 
            let connections = &locked_container.connections;
            let servers = &locked_container.servers;

            println!("{:?}", original_cmd);
            let parsed_command = parse(original_cmd);
            println!("{}", parsed_command);
            let client_id = get_client_id(&parsed_command);
            // println!("{}", client_id);

            if connections.contains_key(&client_id) {
                let message = String::from_iter(payload_list.split_off(1)); 
                //payload list is now a length 1 list containing client_id
                let target_client = payload_list[0];
                println!("{}", target_client);

                let client = connections.get(target_client); 
                println!("{:?}", connections.keys());

                client.unwrap().write_all(message.as_bytes()).unwrap();

            } else {
                // for (_serv_id, mut map_stream) in servers.iter() {
                //     let remote_server_ip = map_stream.peer_addr().unwrap().to_string();

                //     if !arguments.contains(&remote_server_ip) {
                //         println!("looping");
                //         let new_cmd = "RELAY ".to_owned() + &own_server_ip + " " + &arguments;
                //         stream_handler(map_stream.write_all(new_cmd.as_bytes())); 
                //     }
                // }
                relayer(own_server_ip, servers, arguments);
            }

        }

        "TASKADD" => {
            // for TASKADD <task> <client_id> <duedate>? <detail>?
            // DATE - format YYYY-MM-DD
            let tasks = &mut locked_container.tasks;

            let taskname = payload_list[0]; 
            let client = payload_list[1];

            let mut task_id = nanoid!();
            let mut valid = true; 

            let mut detail_text = "None".to_string();
            let mut duedate_text = None;
            
            loop {
                if !tasks.contains_key(&task_id) {
                    break;
                }

                task_id = nanoid!();
            }
            println!("{}", task_id);
            
            if payload_list.len() == 3 {
                let date_detail = payload_list[2];

                if date_detail.chars().count() == 10 {
                    let mut is_detail = true;
                    let mut count = 0;

                    for c in date_detail.chars() {
                        if count == 5 || count == 8 {
                            if c != '-' {
                                is_detail = false;
                                break; 
                            }
                        } else {
                            if !c.is_numeric() {
                                is_detail = false;
                                break;
                            }
                        }

                        count = count + 1;
                    }

                    if is_detail {
                        detail_text = date_detail.to_string();
                        duedate_text = None;

                        // let task = Task {
                        //     id: task_id.to_string(),
                        //     name: taskname.to_string(),
                        //     detail: date_detail.to_string(),
                        //     duedate: "".to_string(),
                        //     owner: client.to_string(),
                        //     total_vote: 0,
                        //     voter_map: HashMap::<String, Option<String>>::new(),
                        // };

                        // tasks.insert(task.id.clone(), task);

                    } else {
                        duedate_text = Some(date_detail.to_string());

                        // let task = Task {
                        //     id: task_id.to_string(),
                        //     name: taskname.to_string(),
                        //     detail: "".to_string(),
                        //     duedate: date_detail.to_string(),
                        //     owner: client.to_string(),
                        //     total_vote: 0,
                        //     voter_map: HashMap::<String, Option<String>>::new(),
                        // };

                        // tasks.insert(task.id.clone(), task);
                    }

                } else {
                    detail_text = date_detail.to_string();
                    duedate_text = None;
                    // let task = Task {
                    //     id: task_id.to_string(),
                    //     name: taskname.to_string(),
                    //     detail: date_detail.to_string(),
                    //     duedate: "".to_string(),
                    //     owner: client.to_string(),
                    //     total_vote: 0,
                    //     voter_map: HashMap::<String, Option<String>>::new(),
                    // };

                    // tasks.insert(task.id.clone(), task);
                }


            }

            else if payload_list.len() == 4 {
                duedate_text = Some(payload_list[2].to_string());
                detail_text = payload_list[3].to_string();
            }

            else {
                valid = false;
                println!("INVALID INPUT!");
            }

            if valid {
                let task = Task {
                    id: Some(task_id.to_string()),
                    content: taskname.to_string(),
                    status: "new".to_string(),
                    detail: detail_text,
                    deadline: duedate_text,
                    creator: client.to_string(),
                    vote: 0,
                    voted: HashMap::<String, Option<String>>::new(),
                    comments: HashMap::<String, String>::new(),
                };
    
                tasks.insert(task_id.clone(), task);
            }
            
            let servers = &locked_container.servers;

            // for (_serv_id, mut map_stream) in servers.iter() {
            //     let remote_server_ip = map_stream.peer_addr().unwrap().to_string();

            //     if !arguments.contains(&remote_server_ip) {
            //         println!("looping");
            //         let new_cmd = "RELAY ".to_owned() + &own_server_ip + " " + &arguments;
            //         stream_handler(map_stream.write_all(new_cmd.as_bytes())); 
            //     }
            // }

            relayer(own_server_ip, servers, arguments);
        }

        "ADDVOTE" => {
            let user_id = payload_list[0].to_string(); 
            let task_name = payload_list[1].to_string();

            let tasks = &mut locked_container.tasks;
            let task_item = tasks.get_mut(&task_name); 
            
            match task_item {
                | Some(task) => {
                    let voters = &mut task.voted; 
                    let voter = voters.get_mut(&user_id); 

                    match voter {
                        | Some(_) => {
                            stream_handler(stream.write_all("Vote already counted\n".as_bytes()));
                        }, 
                        | None => {
                            task.vote = task.vote + 1;
                            voters.insert(user_id.to_string(), None);
                            stream_handler(stream.write_all("Successfully added vote\n".as_bytes()));
                        },
                    }
                }, 
                | None => {stream_handler(stream.write_all("Task deleted successfully\n".as_bytes()));
                },
            }

            let servers = &locked_container.servers;

            relayer(own_server_ip, servers, arguments);
        }

        "WITHDRAWVOTE" => {
            let user_id = payload_list[0].to_string(); 
            let task_name = payload_list[1].to_string();

            let tasks = &mut locked_container.tasks;
            let task_item = tasks.get_mut(&task_name); 
            
            match task_item {
                | Some(task) => {
                    let voters = &mut task.voted; 
                    let voter = voters.get_mut(&user_id); 

                    match voter {
                        | Some(_) => {
                            task.vote = task.vote - 1;
                            voters.insert(user_id.to_string(), None);
                            stream_handler(stream.write_all("Successfully withdrew vote\n".as_bytes()));
                        }, 
                        | None => {
                            stream_handler(stream.write_all("Voter not found\n".as_bytes()));
                        },
                    }
                }, 
                | None => {stream_handler(stream.write_all("Task deleted successfully\n".as_bytes()));
                },
            }

            let servers = &locked_container.servers;

            relayer(own_server_ip, servers, arguments);
        }

        "VALIDATETASK" => {
            let user_id = payload_list[0].to_string(); 
            let task_name = payload_list[1].to_string();
            let signature = payload_list[2].to_string();

            let tasks = &mut locked_container.tasks;
            let task_item = tasks.get_mut(&task_name); 
            
            match task_item {
                | Some(task) => {
                    let voters = &mut task.voted; 
                    let voter = voters.get_mut(&user_id); 

                    match voter {
                        | Some(_) => {
                            stream_handler(stream.write_all("Vote already counted\n".as_bytes()));
                        }, 
                        | None => {
                            voters.insert(user_id.to_string(), Some(signature.to_string()));
                            stream_handler(stream.write_all("Successfully added vote\n".as_bytes()));
                        },
                    }
                }, 
                | None => {stream_handler(stream.write_all("Task deleted successfully\n".as_bytes()));
                },
            }

            let servers = &locked_container.servers;

            relayer(own_server_ip, servers, arguments);
        }

        "TASKDEL" => {
            let tasks = &mut locked_container.tasks;

            if payload_list.len() == 1 {
                let task_id = payload_list[0];

                match tasks.remove(task_id) {
                    | Some(_) => {
                        stream_handler(stream.write_all("Task deleted successfully\n".as_bytes()));
                    },
                    | None => {
                        stream_handler(stream.write_all("Task does not exist\n".as_bytes()));
                    },
                }
            }

            let servers = &locked_container.servers;

            relayer(own_server_ip, servers, arguments);
        }

        _ => {
            command_parser(stream, arguments, &mut locked_container);
        }
    }
}

fn stream_handler(write_result: Result<(), std::io::Error>) {
    match write_result {
        Ok(()) => {
            //println!("Successfully wrote to stream")
        }

        _error => {
            println!("Failed to write to stream")
        }
    }
}


// REGISTER <client_id>
// SEND <client_id> <message>
// BROADCAST <message>

// ADDVOTE <user_id> <task_id> 
// WITHDRAWVOTE <user_id> <task_id> 
// VALIDATETASK <user_id> <task_id> <signature> 
fn command_parser(mut stream: &std::net::TcpStream, arguments: String, locked_container: &mut MapContainer){
    // let mut connections = &mut locked_container.connections;
    // let mut tasks = &mut locked_container.tasks;

    let mut words: Vec<&str> = arguments.trim().split(' ').collect();
    // let original_cmd = words.clone();
    let mut payload_list = words.split_off(1); // split into command and payload
    
    match words[0].trim() {
        "EXIT" => {
            std::process::exit(0);
        }

        // "ADDVOTE" => {
        //     let mut tasks = &mut locked_container.tasks;

        //     if payload_list.len() == 1 {
        //         let user_id = payload_list[0];

        //         let chosen_task 
        //     }
        // }

        "SERVERREG" => {
            let hardcode_psw = "CHANGE LATER";
            let servers = &mut locked_container.servers;

            if payload_list.len() == 2 {
                let server_id = payload_list[0];
                let psw = payload_list[1];

                if true {
                    servers.insert(server_id.to_string(), stream.try_clone().unwrap());

                } else {
                    stream_handler(stream.write_all("wrong password!".as_bytes()));
                }

            } else {
                stream_handler(stream.write_all("wrong command syntax".as_bytes()));
            }
        }

        "SERVERLIST" => {
            let servers = &locked_container.servers;

            for (_, server_stream) in servers.iter() {
                let addr = server_stream.peer_addr().unwrap().to_string();

                stream_handler(stream.write_all(addr.as_bytes()));
                // println!("{:?}", value);
            }
        }

        // "WITHDRAWVOTE" => {
        //     let user_id = payload_list[0].to_string(); 
        //     let task_name = payload_list[1].to_string();

        //     let tasks = &mut locked_container.tasks;
        //     let task_item = tasks.get_mut(&task_name); 
            
        //     match task_item {
        //         | Some(task) => {
        //             let voters = &mut task.voter_map; 
        //             let voter = voters.get_mut(&user_id); 

        //             match voter {
        //                 | Some(_) => {
        //                     task.total_vote = task.total_vote - 1;
        //                     voters.insert(user_id.to_string(), None);
        //                     stream_handler(stream.write_all("Successfully withdrew vote\n".as_bytes()));
        //                 }, 
        //                 | None => {
        //                     stream_handler(stream.write_all("Voter not found\n".as_bytes()));
        //                 },
        //             }
        //         }, 
        //         | None => {stream_handler(stream.write_all("Task deleted successfully\n".as_bytes()));
        //         },
        //     }
        // }

        // "ADDVOTE" => {
        //     let user_id = payload_list[0].to_string(); 
        //     let task_name = payload_list[1].to_string();

        //     let tasks = &mut locked_container.tasks;
        //     let task_item = tasks.get_mut(&task_name); 
            
        //     match task_item {
        //         | Some(task) => {
        //             let voters = &mut task.voter_map; 
        //             let voter = voters.get_mut(&user_id); 

        //             match voter {
        //                 | Some(_) => {
        //                     stream_handler(stream.write_all("Vote already counted\n".as_bytes()));
        //                 }, 
        //                 | None => {
        //                     task.total_vote = task.total_vote + 1;
        //                     voters.insert(user_id.to_string(), None);
        //                     stream_handler(stream.write_all("Successfully added vote\n".as_bytes()));
        //                 },
        //             }
        //         }, 
        //         | None => {stream_handler(stream.write_all("Task deleted successfully\n".as_bytes()));
        //         },
        //     }
        // }

        // "VALIDATETASK" => {
        //     let user_id = payload_list[0].to_string(); 
        //     let task_name = payload_list[1].to_string();
        //     let signature = payload_list[2].to_string();

        //     let tasks = &mut locked_container.tasks;
        //     let task_item = tasks.get_mut(&task_name); 
            
        //     match task_item {
        //         | Some(task) => {
        //             let voters = &mut task.voter_map; 
        //             let voter = voters.get_mut(&user_id); 

        //             match voter {
        //                 | Some(_) => {
        //                     stream_handler(stream.write_all("Vote already counted\n".as_bytes()));
        //                 }, 
        //                 | None => {
        //                     voters.insert(user_id.to_string(), Some(signature.to_string()));
        //                     stream_handler(stream.write_all("Successfully added vote\n".as_bytes()));
        //                 },
        //             }
        //         }, 
        //         | None => {stream_handler(stream.write_all("Task deleted successfully\n".as_bytes()));
        //         },
        //     }
        // }


        "TASKLIST" => {
            let tasks = &locked_container.tasks;

            for (_, value) in tasks.iter() {
                let encoded = serde_json::to_string(&value).unwrap();

                stream_handler(stream.write_all(encoded.as_bytes()));
                // println!("{:?}", value);
            }
            
        }

        "LIST" => {
            let connections = &mut locked_container.connections;

            let _client_ids = connections.keys();

            // needs formatting
            println!("{:?}", connections.keys());

            let mut keys = Vec::new();

            connections.iter().for_each(|(key, _)| {keys.push(key.as_bytes())});

            // stream.write_all(keys.as_slice());
        }

        "REGISTER" => {
            let connections = &mut locked_container.connections;
            let client_id = payload_list[0].trim().to_string();
            println!("{}", client_id);

            if !connections.contains_key(&client_id) {
                connections.insert(client_id, stream.try_clone().unwrap());
                stream_handler(stream.write_all("Successfully registered\n".as_bytes()));

            } else {
                stream_handler(stream.write_all("Client already exists\n".as_bytes()));
            }
        }

        "SEND" => {
            let connections = &mut locked_container.connections;
            let message = String::from_iter(payload_list.split_off(1)); 
            //payload list is now a length 1 list containing client_id
            let target_client = payload_list[0];
            println!("{}", target_client);

            let client = connections.get(target_client); 
            println!("{:?}", connections.keys());

            stream_handler(client.unwrap().write_all(message.as_bytes()));
            
        }

        "BROADCAST" => {
            let connections = &mut locked_container.connections;
            // send data to all clients that are connected
            let payload = String::from_iter(payload_list);

            connections.iter().for_each(|(_id, mut client)| {
                stream_handler(client.write_all(payload.as_bytes()));
            });
        }

        "RENAME" => {
            let connections = &mut locked_container.connections;
            let old_id = payload_list[0].trim().to_string();
            let new_id = payload_list[1].trim().to_string();
            let old_stream = connections.remove(&old_id).unwrap();
            // println!("{}", client_id);
            
            if connections.contains_key(&new_id) {
                stream_handler(stream.write_all("Could not insert because id is already used".as_bytes()));
            } else {
                connections.insert(new_id, old_stream);
                stream_handler(stream.write_all("ID successfully changed".as_bytes()));
            }
            println!("{:?}", connections.keys());
        }

        "KICK" => {
            let connections = &mut locked_container.connections;
            let old_id = payload_list[0].trim().to_string();
            let old_stream = connections.remove(&old_id).unwrap();
            // println!("{}", client_id);
            
            old_stream.shutdown(std::net::Shutdown::Both).expect("shutdown call failed");
            println!("{:?}", connections.keys());
        }

        &_ => {
            println!("Invalid command");
        }
    }
}

fn handle_connection(mut stream: &std::net::TcpStream, maps: Arc<Mutex<MapContainer>>) {
    let mut buffer = [0; 1024];
    let timeout_duration = Duration::from_millis(200);

    stream.set_read_timeout(Some(timeout_duration)).unwrap();
    
    loop {
        match stream.read(&mut buffer) {
            |Ok(0) => {
                println!("Client disconnected");
                stream.shutdown(std::net::Shutdown::Both).expect("shutdown call failed");
                break;
            },
            |Ok(size) => {
                let my_str = String::from_utf8_lossy(&buffer[..size]);
                let mut answer = String::new();
                answer.push_str("> ");
                answer.push_str(&my_str);
                
                let clients = maps.clone();
                relay_parser(stream, my_str.to_string(), clients);
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
