use crate::{config::Config, service::ContactService};
use libs::domain::manager::ContactManager;
use std::sync::Arc;
use tokio::sync::RwLock;

// ContactManager is added to ApiState because it holds core business logic
// Adding ContactManager to ApiState avoid excessive use of memory for creating a Manager for every (concurent) handler
// ContactManager access is cordinated by an async RWLock primitive
#[derive(Clone)]
pub struct ApiState {
    pub config: Arc<Config>,
    pub service: Arc<ContactService>,
}

impl ApiState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let manager = Arc::new(RwLock::new(ContactManager::new().await?));

        Ok(Self {
            config: Arc::new(config),
            service: Arc::new(ContactService::new(manager)),
        })
    }
}
