use std::fmt::{Debug, Display};

use actix::{Message, Recipient};
use serde::Serialize;

/// A message from the `server` to all the `client`s.
#[derive(Clone, Message)]
#[rtype(result = "()")]
#[repr(transparent)]
pub struct ServerMsg(pub String);

/// A message from a `client`.
#[derive(Debug, Message, Serialize)]
#[rtype(result = "()")]
pub struct ClientMsg {
    pub(super) client_id: usize,
    pub(super) client_name: String,
    pub(super) message: String,
    pub(super) room: String,
}

impl Display for ClientMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ClientMsg {
            client_name,
            message,
            room,
            ..
        } = self;
        write!(
            f,
            r#"[message]: client "{}" in room "{}" says: "{}""#,
            client_name, room, message
        )
    }
}

/// A new connetion to the `server`.
#[derive(Debug, Message)]
#[rtype(result = "usize")]
pub struct Connect {
    pub(super) name: String,
    pub(super) room: String,
    pub(super) addr: Recipient<ServerMsg>,
}

impl Display for Connect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"[connection]: client "{}" has connected to the server in room "{}""#,
            self.name, self.room
        )
    }
}

/// A disconnection from `server`.
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub(super) id: usize,
    pub(super) name: String,
}

impl Display for Disconnect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"[disconnection]: client "{}" has been disconnected from the server"#,
            self.name
        )
    }
}
