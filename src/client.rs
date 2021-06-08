use crate::standard_framework::StandardFramework;
use tracing::debug;
use momiji::core::timers::TimerClient;
use momiji::db::DatabaseConnection;
use momiji::{core::handler::EventHandler};
use momiji::framework::parser::StandardParser;
use std::collections::HashSet;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::cluster::Cluster;
use twilight_http::Client as HttpClient;
use twilight_model::gateway::{
    Intents,
    payload::update_status::UpdateStatusInfo,
    presence::{Activity, ActivityType, Status}
};

#[non_exhaustive]
pub struct Client {
    pub cache: InMemoryCache,
    pub cluster: Cluster,
    pub handler: EventHandler<StandardParser>,
    pub http: HttpClient,
    pub database: DatabaseConnection,
    pub timers: TimerClient,
}

impl Client {
    pub async fn new(token: &str, intents: Intents) -> Self {
        let http = HttpClient::new(token);
        let cache = InMemoryCache::new();
        let database = DatabaseConnection::connect();
        
        let owner = http.current_user_application().await.expect("Unable to retrieve application info.").owner.id;
        let mut owners = HashSet::new();
        owners.insert(owner);

        let cluster = Cluster::builder(token, intents)
            .http_client(http.clone())
            .presence(status_info())
            .build()
            .await
            .unwrap_or_else(|err| {
                panic!("Unable to start cluster\n{}", err);
            });

        let timers = TimerClient::new(http.clone(), cache.clone(), database.clone());

        let framework = StandardFramework::new(owners, cluster.clone());
        let handler = EventHandler::new(framework);

        Self {
            cache,
            cluster,
            handler,
            http,
            database,
            timers,
        }
    }

    pub async fn start(self) {
        let cluster_spawn = self.cluster.clone();
        let timers = self.timers.clone();

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
        self.handler.start(&self.cluster, &self.http, &self.cache, self.database.clone(), self.timers.clone()).await
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