use std::collections::{HashMap, HashSet};

pub mod message;
pub mod session;
use message::*;

use actix::{Actor, Context, Handler, Recipient};
use rand::{rngs::ThreadRng, Rng};

pub struct ChatServer {
    sessions: HashMap<usize, Recipient<ServerMsg>>, // <- ws handhle
    rooms: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
    last: usize,
}

impl ChatServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            rng: Default::default(),
            last: 0,
        }
    }

    fn send_msg(&self, skip: usize, room: &str, msg: ServerMsg) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if skip.ne(id) {
                    if let Some(r) = self.sessions.get(id) {
                        r.do_send(msg.clone())
                    }
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

        let Connect { name, room, addr } = msg;
        let id: usize = loop {
            let n = self.rng.gen();
            if n != self.last {
                self.last = n;
                break n;
            }
        };

        self.send_msg(
            id,
            &room,
            ServerMsg(format!(
                r#"client "{name}" has connected to the room"#
            )),
        );

        self.sessions.insert(id, addr);
        self.rooms
            .entry(room)
            .or_insert_with_key(|r| {
                log::info!(r#"[server]: inexistent room "{r}" created"#);
                HashSet::new()
            })
            .insert(id);

        id
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) {
        log::info!("{msg}");

        let Disconnect { id, name } = msg;
        {
            _ = self.sessions.remove(&id);
            self.rooms.iter_mut().for_each(|(_, c)| _ = c.remove(&id));
        }

        for (room, _) in self.rooms.iter() {
            self.send_msg(
                id,
                room,
                ServerMsg(format!(
                    r#"client "{name}" has been disconnected from the server"#
                )),
            );
        }
    }
}

impl Handler<ClientMsg> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMsg, _: &mut Self::Context) {
        log::info!("{msg}");

        let ClientMsg {
            client_id,
            client_name,
            message,
            room,
        } = msg;

        self.send_msg(
            client_id,
            &room,
            ServerMsg(
                serde_json::json!({
                    "client_id": client_id,
                    "client_name": client_name,
                    "message": message
                })
                .to_string(),
            ),
        )
    }
}
