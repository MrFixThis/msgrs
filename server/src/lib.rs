use actix::{Actor, Addr};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use chat::{session, ChatServer};

mod chat;
pub mod settings;

#[actix_web::get("/")]
async fn ws_chat(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<ChatServer>>,
) -> actix_web::Result<HttpResponse> {
    ws::start(
        session::ChatSession::new(String::new(), String::new(), server.as_ref().clone()),
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
