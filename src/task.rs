use std::collections::HashMap;
use std::net::{TcpStream};
// use bytevec::{ByteEncodable, ByteDecodable};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: Option<String>,
    pub content: String, 
    pub vote: i32,
    pub deadline: Option<String>,
    pub status: String,
    pub voted: HashMap<String, Option<String>>,
    pub creator: String,
    pub detail: String,
    pub comments: HashMap<String, String>,
    //Array<Json>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MyObj {
    pub body: String,
    pub user: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewComment {
    pub user_id: String,
    pub comment: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewVote {
    pub user_id: String,
    pub signature: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
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
