use std::time::{Duration, Instant};

use actix::{
    fut, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Handler, StreamHandler,
    WrapFuture, ContextFutureSpawner
};
use actix_web_actors::ws;

use crate::{chat::message::Disconnect, settings};

use super::message::{self, Connect, ClientMsg};

pub struct ChatSession {
    id: usize,
    name: String,
    room: String,
    hb: Instant,
    addr: Addr<super::ChatServer>,
}

impl ChatSession {
    pub fn new(name: String, room: String, addr: Addr<super::ChatServer>) -> Self {
        Self {
            id: 0,
            name,
            room,
            hb: Instant::now(),
            addr,
        }
    }

    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        let settings = settings::get_app_settings();
        let interval = Duration::from_secs(settings.hb_interval());

        ctx.run_interval(interval, move |act, ctx| {
            let timeout = Duration::from_secs(settings.timeout());
            if Instant::now().duration_since(act.hb) > timeout {
                log::info!("[client]: heartbeat signal failed");

                act.addr.do_send(Disconnect {
                    id: act.id,
                    name: act.name.clone(),
                });
                ctx.stop();
            }

            ctx.ping(b"")
        });
    }
}

impl Actor for ChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
        self.addr
            .send(Connect {
                name: self.name.clone(),
                room: self.room.clone(),
                addr: ctx.address().recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(id) => act.id = id,
                    _ => ctx.stop(),
                }

                fut::ready(())
            })
            .wait(ctx)
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        let ChatSession { id, name, .. } = self;
        self.addr.do_send(Disconnect {
            id: *id,
            name: name.clone(),
        });
        actix::Running::Stop
    }
}

impl Handler<message::ServerMsg> for ChatSession {
    type Result = ();

    fn handle(&mut self, msg: message::ServerMsg, ctx: &mut Self::Context) {
        ctx.text(msg.message)
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSession {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(msg) => match msg {
                ws::Message::Text(txt) => {
                    let msg = txt.trim();
                    self.addr.do_send(ClientMsg {
                        client_id: self.id,
                        client_name: self.name.clone(),
                        message: msg.to_owned(),
                        room: self.room.clone(),
                    })
                },
                ws::Message::Ping(msg) => {
                    self.hb = Instant::now();
                    ctx.pong(&msg);
                },
                ws::Message::Pong(_) => self.hb = Instant::now(),
                ws::Message::Close(reason) => ctx.close(reason),
                ws::Message::Continuation(_) => ctx.stop(),
                _ => ()
            },
            Err(_) => ctx.stop(),
        }
    }
}
