use env_logger::Env;
use server;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(Env::new().default_filter_or("info"));
    server::settings::load_settings()?;
    server::setup_server().await?.await?;
    Ok(())
}
