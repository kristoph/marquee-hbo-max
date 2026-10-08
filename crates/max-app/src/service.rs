use std::sync::Arc;

use max_api::client::Client;

#[derive(Clone)]
pub enum Service {
    Live(Arc<Client>),
    Captured,
    SignedOut,
}

impl Service {
    pub fn client(&self) -> Option<Arc<Client>> {
        match self {
            Service::Live(client) => Some(client.clone()),
            Service::Captured | Service::SignedOut => None,
        }
    }

    pub fn is_live(&self) -> bool {
        matches!(self, Service::Live(_))
    }
}
