use std::sync::Arc;
use crate::config::Config;
use libs::{domain::manager::ContactManager};
use tokio::sync::RwLock;


#[derive(Clone)]
pub struct ApiState {
    pub config: Arc<Config>,
    pub manager: Arc<RwLock<ContactManager>>,
}

impl ApiState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        Ok(
            Self { 
                config: Arc::new(config),
                manager: Arc::new(RwLock::new(ContactManager::new().await?))
            }
        )
    }
}