use actix::{Actor, Addr};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use chat::{session, ChatServer};
use serde::Deserialize;

mod chat;
pub mod settings;

#[derive(Deserialize)]
struct Register {
    name: String,
    room: String,
}

#[actix_web::get("/")]
async fn ws_chat(
    req: HttpRequest,
    reg: web::Query<Register>,
    stream: web::Payload,
    server: web::Data<Addr<ChatServer>>,
) -> actix_web::Result<HttpResponse> {
    let Register { name, room } = reg.into_inner();
    ws::start(
        session::ChatSession::new(name, room, server.as_ref().clone()),
        &req,
        stream,
    )
}

/// Setups the application's components.
#[inline]
pub async fn setup_server() -> anyhow::Result<actix_web::dev::Server> {
    let settings = settings::get_app_settings();
    let chat_server = chat::ChatServer::new().start();
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(chat_server.clone()))
            .service(ws_chat)
    })
    .workers(settings.workers())
    .bind(settings)?
    .run();

    Ok(server)
}
