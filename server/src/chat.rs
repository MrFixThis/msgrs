use std::collections::{HashMap, HashSet};

pub mod message;
pub mod session;
use message::*;

use actix::{Actor, Context, Handler, Recipient};

pub struct ChatServer {
    sessions: HashMap<usize, Recipient<ServerMsg>>,
    rooms: HashMap<String, HashSet<usize>>,
}

impl ChatServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
        }
    }

    fn send_msg(&self, room: &str, msg: ServerMsg) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if let Some(r) = self.sessions.get(id) {
                    r.do_send(msg.clone())
                }
            }
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        log::info!("{msg}");

        let Connect {
            name: client_name,
            room,
            addr,
        } = msg;

        self.send_msg(
            &room,
            ServerMsg {
                message: format!(r#"client "{}" connected to room "{}""#, client_name, room),
                client_name,
            },
        );

        let id = self.sessions.len();
        self.sessions.insert(id, addr);
        self.rooms
            .entry(room.clone())
            .or_insert_with(|| {
                log::info!(r#"creating room "{}""#, room);
                HashSet::new()
            })
            .insert(id);

        id
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        log::info!("{msg}");

        let Disconnect {
            id,
            name: client_name,
        } = msg;

        {
            _ = self.sessions.remove(&id);
            self.rooms.iter_mut().for_each(|(_, c)| _ = c.remove(&id));
        }

        for (room, _) in self.rooms.iter() {
            self.send_msg(
                room,
                ServerMsg {
                    message: format!(r#"client "{}" disconnected from the server"#, client_name),
                    client_name: client_name.clone(),
                },
            );
        }
    }
}

impl Handler<ClientMsg> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMsg, _: &mut Self::Context) {
        log::info!("{msg}");

        let ClientMsg {
            client_name,
            message,
            room,
            ..
        } = msg;

        self.send_msg(
            &room,
            ServerMsg {
                client_name,
                message,
            },
        )
    }
}
