use std::collections::{HashSet, HashMap};
use std::sync::{Arc, Mutex};

pub struct ConnectionState {
    listeners: HashSet<String>,
    role: String
}

pub struct WebSocketServer {
    connections: Arc<Mutex<HashMap<String, ConnectionState>>>,
}

impl WebSocketServer {
    pub fn new() -> WebSocketServer {
        WebSocketServer {
            connections: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn new_connection(&self, connection_id: String, role: String) {
        let mut connections = self.connections.lock().unwrap();
        connections.insert(connection_id, ConnectionState {
            listeners: HashSet::new(),
            role
        });
    }

    pub fn handle_listen_event(&self, connection_id: String, event_name: String) {
        let mut connections = self.connections.lock().unwrap();
        if let Some(connection_state) = connections.get_mut(&connection_id) {
            connection_state.listeners.insert(event_name);
        }
    }

    pub fn broadcast_event(&self, event_name: String, event_data: String) {
        let connections = self.connections.lock().unwrap();
        for (connection_id, connection_state) in connections.iter() {
            if connection_state.listeners.contains(&event_name) {
                // Send event to connection
            }
        }
    }
}