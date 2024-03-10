use std::{net::SocketAddr, path::Path};

use async_trait::async_trait;
use axum::Router as AxumRouter;
use lazy_static::lazy_static;
use loco_rs::{
    app::Hooks,
    boot::{create_app, BootResult, ServeParams, StartMode},
    controller::AppRoutes,
    db::truncate_table,
    environment::Environment,
    prelude::*,
    task::Tasks,
    worker::Processor,
};
use migration::Migrator;
use sea_orm::DatabaseConnection;

use crate::{controllers, initializers, models::_entities::user};

lazy_static! {
    pub static ref REQWEST_CLIENT: ReqwestClient = ReqwestClient::new().unwrap();
}

pub struct ReqwestClient {
    pub client: reqwest::Client,
}

#[derive(thiserror::Error, Debug)]
enum ReqwestClientInitError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

impl ReqwestClient {
    fn new() -> std::result::Result<Self, ReqwestClientInitError> {
        let client = reqwest::Client::builder()
            .user_agent("threads-client")
            .build()?;

        Ok(Self { client })
    }
}

pub struct App;
#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(mode: StartMode, environment: &Environment) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment).await
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .prefix("/api")
            .add_route(controllers::vote::routes())
            .add_route(controllers::leaderboard::routes())
    }

    async fn truncate(db: &DatabaseConnection) -> Result<()> {
        truncate_table(db, user::Entity).await?;
        Ok(())
    }

    async fn seed(_db: &DatabaseConnection, _base: &Path) -> Result<()> {
        unimplemented!("not gonna implement")
    }

    fn connect_workers<'a>(_p: &'a mut Processor, _ctx: &'a AppContext) {
        unimplemented!("not gonna implement");
    }

    fn register_tasks(_tasks: &mut Tasks) {
        unimplemented!("not gonna implement");
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![Box::new(initializers::ip_getter::IPGetterInitializer)])
    }

    async fn serve(app: AxumRouter, server_config: ServeParams) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(&format!(
            "{}:{}",
            server_config.binding, server_config.port
        ))
        .await?;

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;

        Ok(())
    }
}
