use std::collections::HashMap;
use std::net::{TcpStream};
// use bytevec::{ByteEncodable, ByteDecodable};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub detail: String,
    pub duedate: String,
    pub owner: String,
    pub total_vote: i32,
}

pub struct MapContainer {
    pub connections: HashMap<String, TcpStream>,
    pub tasks: HashMap<String, Task>, 
    pub servers: HashMap<String, TcpStream>, 
}

impl Clone for MapContainer {
    fn clone(&self) -> MapContainer {
        MapContainer { 
            connections: self.connections.iter().map(
                |(key, value)| (key.clone(), value.try_clone().unwrap())
            ).collect(),
            tasks: self.tasks.clone(), 
            servers: self.servers.iter().map(
                |(key, value)| (key.clone(), value.try_clone().unwrap())
            ).collect(),
        }
    }
}
