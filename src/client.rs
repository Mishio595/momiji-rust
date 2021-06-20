use crate::standard_framework::StandardFramework;
use tracing::debug;
use momiji::Context;
use momiji::core::timers::TimerClient;
use momiji::db::DatabaseConnection;
use momiji::{core::handler::EventHandler};
use momiji::framework::parser::Parser;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::cluster::Cluster;
use twilight_http::Client as HttpClient;
use twilight_model::gateway::{
    Intents,
    payload::update_status::UpdateStatusInfo,
    presence::{Activity, ActivityType, Status}
};
use std::collections::HashMap;
use std::sync::Arc;

#[non_exhaustive]
pub struct Client {
    pub handler: EventHandler,
    pub ctx: Context,
}

impl Client {
    pub async fn new(token: &str, intents: Intents) -> Self {
        let http = HttpClient::new(token);
        let cache = InMemoryCache::new();
        let db = DatabaseConnection::connect();
        let parser = Parser;
        
        let app_info = http.current_user_application().await.expect("Unable to retrieve application info.");
        let user = http.current_user().await.expect("Unable to retrieve current user.");

        let owner = app_info.owner;
        let mut owners = HashMap::new();
        owners.insert(owner.id, Arc::new(owner));

        let cluster = Cluster::builder(token, intents)
            .http_client(http.clone())
            .presence(status_info())
            .build()
            .await
            .unwrap_or_else(|err| {
                panic!("Unable to start cluster\n{}", err);
            });

        let tc = TimerClient::new(http.clone(), cache.clone(), db.clone());

        let ctx = Context {
            cache,
            cluster,
            db,
            http,
            parser,
            tc,
            owners: Arc::new(owners),
            user: Arc::new(user)
        };

        let framework = StandardFramework::new(ctx.clone());
        let handler = EventHandler::new(framework, ctx.clone());

        Self {
            ctx,
            handler,
        }
    }

    pub async fn start(self) {
        let cluster_spawn = self.ctx.cluster.clone();
        let timers = self.ctx.tc.clone();

        debug!("Starting Cluster");
        let cluster_handle = tokio::spawn(async move {
            cluster_spawn.up().await;
        });

        debug!("Starting Event Handler");
        let handler_handle = tokio::spawn(async move {
            self.start_handler().await;
        });

        //TODO add signal handling with signal-hook crate

        debug!("Starting Timer Client");
        timers.start().await;
    }

    async fn start_handler(&self) {
        self.handler.start().await
    }
}

fn status_info() -> UpdateStatusInfo {
    let activity = Activity {
        application_id: None,
        assets: None,
        buttons: Vec::new(),
        created_at: None,
        details: None,
        emoji: None,
        flags: None,
        id: None,
        instance: None,
        kind: ActivityType::Listening,
        name: "Awoo ASMR".to_string(),
        party: None,
        secrets: None,
        state: None,
        timestamps: None,
        url: None,
    };
    UpdateStatusInfo {
        activities: Some(vec![activity]),
        afk: false,
        since: None,
        status: Status::Online
    }
}