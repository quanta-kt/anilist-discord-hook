use std::fs;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Datastore {
    pub last_published_timestamp: u64,
}

impl Default for Datastore {
    fn default() -> Self {
        Datastore {
            last_published_timestamp: 0,
        }
    }
}

impl Datastore {
    pub fn read() -> Datastore {
        let Ok(contents) = fs::read_to_string("data.json") else {
            return Datastore::default();
        };

        match serde_json::from_str(&contents) {
            Ok(config) => config,
            _ => Datastore::default(),
        }
    }
    
    pub fn write(&self) {
        let json = serde_json::to_string(self).unwrap();
        fs::write("data.json", json).unwrap();
    }
}
