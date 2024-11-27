use crate::error::*;
use crate::server::WizardServer;
use derive_more::Display;
use std::sync::Arc;
use thiserror::Error;

#[derive(Default, Clone, Debug)]
pub struct WizardServerBuilder {
    port: Option<u16>,
    ngrok_authtoken: Option<String>,
}

#[derive(Error, Display, Debug)]
pub enum WizardServerBuilderError {
    NoPort,
    NgrokError,
}

impl WizardServerBuilder {
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);

        self
    }

    pub fn with_ngrok(mut self, authtoken: impl Into<String>) -> Self {
        self.ngrok_authtoken = Some(authtoken.into());

        self
    }

    pub async fn build(self) -> Result<Arc<WizardServer>> {
        if self.port.is_none() {
            return Err(Error::from(WizardServerBuilderError::NoPort));
        }

        WizardServer::new(self.port.unwrap(), self.ngrok_authtoken).await
    }
}
