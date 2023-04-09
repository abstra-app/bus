use std::collections::{HashSet, HashMap};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use crate::validator::{RequestMessage, ResponseMessage, ListenMessage, BroadcastMessage};
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Clone)]
struct Listener {
    connection_id: Uuid,
    callback: Arc<dyn Fn(BroadcastMessage) + Send + Sync>,
}

impl PartialEq for Listener {
    fn eq(&self, other: &Self) -> bool {
        self.connection_id == other.connection_id
    }
}

impl Eq for Listener {}

impl Hash for Listener {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.connection_id.hash(state);
    }
}

impl fmt::Debug for Listener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Listener")
            .field("connection_id", &self.connection_id)
            .finish()
    }
}

#[derive(Clone)]
struct Responder {
    connection_id: Uuid,
    callback: Arc<dyn Fn(RequestMessage) -> ResponseMessage + Send + Sync>,
}

impl PartialEq for Responder {
    fn eq(&self, other: &Self) -> bool {
        self.connection_id == other.connection_id
    }
}

impl Eq for Responder {}

impl Hash for Responder {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.connection_id.hash(state);
    }
}

impl fmt::Debug for Responder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Responder")
            .field("connection_id", &self.connection_id)
            .finish()
    }
}

#[derive(Clone)]
struct Requester {
    connection_id: Uuid,
    message: RequestMessage,
    callback: Arc<dyn Fn(ResponseMessage) + Send + Sync>,
}

pub struct Broker {
    listeners: Arc<Mutex<HashMap<String, Arc<Mutex<HashSet<Listener>>>>>>,
    responders: Arc<Mutex<HashMap<String, Responder>>>,
    requests: Arc<HashMap<String, Mutex<Vec<Requester>>>>,
}

impl Broker {
    pub fn new() -> Broker {
        Broker {
            listeners: Arc::new(Mutex::new(HashMap::new())),
            responders: Arc::new(Mutex::new(HashMap::new())),
            requests: Arc::new(HashMap::new()),
        }
    }

    pub fn listen(&self, connection_id: Uuid, message: ListenMessage, callback: Arc<dyn Fn(BroadcastMessage) + Send + Sync>) {
        let mut listeners_map = self.listeners.lock().unwrap();
        let listeners = listeners_map.entry(message.channel.clone()).or_insert(Arc::new(Mutex::new(HashSet::new())));
        let mut listeners = listeners.lock().unwrap();
        let listener = Listener {
            connection_id,
            callback,
        };
        listeners.insert(listener);
    }

    pub fn broadcast(&self, message: BroadcastMessage) {
        let listeners_map = self.listeners.lock().unwrap();
        if let Some(listeners) = listeners_map.get(&message.channel) {
            let listeners = listeners.lock().unwrap();
            for listener in listeners.iter() {
                (listener.callback)(message.clone());
            }
        }
    }

    pub fn respond(&self, connection_id: Uuid, message: ResponseMessage, callback: Arc<dyn Fn(RequestMessage) -> ResponseMessage + Send + Sync>) {
        let mut responders = self.responders.lock().unwrap();
        let responder = Responder {
            connection_id,
            callback,
        };
        responders.insert(message.channel, responder);
    }

    pub fn request(&self, connection_id: Uuid, message: RequestMessage, callback: Box<dyn Fn(ResponseMessage) + Send + Sync>) {
        let channel = message.channel.clone();
        let mut requests = self.requests.get(&channel);
        if requests.is_none(){return};
        let mut requests = requests.unwrap().lock().unwrap();
        let requester = Requester {
            connection_id,
            message,
            callback: Arc::new(move |response| {
                callback(response);
            }),
        };
        requests.push(requester);
    }

    fn consume_request(&self, channel: String) {
        let mut requests = self.requests.get(&channel).unwrap().lock().unwrap();
        let request = requests.pop();
        match request {
            Some(request) => {
                let responders = self.responders.lock().unwrap();
                if let Some(responder) = responders.get(&channel) {
                    (responder.callback)(request.message);
                }
            },
            None => {}
        }
    }
}