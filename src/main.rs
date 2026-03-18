mod cli;
mod consent_idat;

use crate::cli::Cli;
use crate::consent_idat::ConsentType;
use clap::Parser;

use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::{ClientConfig, Message};
use std::error::Error;
use std::sync::LazyLock;
use tracing::{error, info};

#[cfg(test)]
use httpmock::MockServer;

#[cfg(not(test))]
static CONFIG: LazyLock<Cli> = LazyLock::new(Cli::parse);

async fn start_service(consumer: StreamConsumer) -> Result<(), Box<dyn Error>> {
    let topic: &str = &CONFIG.topic.clone();
    consumer.subscribe(&[topic])?;
    info!("Kafka topic '{}' subscribed", CONFIG.topic);

    while let Ok(msg) = consumer.recv().await {
        let message = msg.payload().unwrap_or_default();
        let message_str = std::str::from_utf8(message).unwrap_or_default();
        let consent_idat: consent_idat::ConsentIdat = match serde_json::from_str(message_str) {
            Ok(idat) => idat,
            Err(e) => {
                error!("Failed to parse consent IDAT: {e}");
                continue;
            }
        };

        if consent_idat.is_genomde() {
            let patient_id = consent_idat.patient_id();
            match send_consent(&patient_id, ConsentType::GenomDe, message_str).await {
                Ok(()) => info!("GenomDE consent for '{patient_id}' sent to Onkostar"),
                Err(e) => {
                    error!("Failed to send GenomDE consent for '{patient_id}' to Onkostar: {e}");
                }
            }
        } else if consent_idat.is_broad_consent() {
            let patient_id = consent_idat.patient_id();
            match send_consent(&patient_id, ConsentType::BroadConsent, message_str).await {
                Ok(()) => info!("MII consent for '{patient_id}' sent to Onkostar"),
                Err(e) => {
                    error!("Failed to send MII consent for '{patient_id}'  to Onkostar: {e}");
                }
            }
        }
    }

    Ok(())
}

#[allow(clippy::expect_used)]
async fn send_consent(
    patient_id: &str,
    consent_type: ConsentType,
    content: &str,
) -> Result<(), String> {
    let user_agent_string = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let client = reqwest::Client::builder()
        .user_agent(user_agent_string)
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())?;

    let onkostar_uri = if CONFIG.onkostar_uri.ends_with('/') {
        &CONFIG.onkostar_uri[0..&CONFIG.onkostar_uri.len() - 1].to_string()
    } else {
        &CONFIG.onkostar_uri
    };

    let url = match consent_type {
        ConsentType::GenomDe => format!("{onkostar_uri}/x-api/patient/{patient_id}/consent/mv64e"),
        ConsentType::BroadConsent => {
            format!("{onkostar_uri}/x-api/patient/{patient_id}/consent/research")
        }
    };

    let _ = client
        .put(url)
        .json(&content)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut client_config = ClientConfig::new();
    client_config.set("bootstrap.servers", &CONFIG.bootstrap_servers);

    let mut client_config = if CONFIG.ssl_cert_file.is_some() || CONFIG.ssl_key_file.is_some() {
        client_config
            .set("security.protocol", "ssl")
            .set(
                "ssl.ca.location",
                CONFIG.ssl_ca_file.clone().unwrap_or_default(),
            )
            .set(
                "ssl.certificate.location",
                CONFIG.ssl_cert_file.clone().unwrap_or_default(),
            )
            .set(
                "ssl.key.location",
                CONFIG.ssl_key_file.clone().unwrap_or_default(),
            );
        if let Some(ssl_key_password) = &CONFIG.ssl_key_password {
            client_config.set("ssl.key.password", ssl_key_password);
        }
        client_config
    } else {
        client_config
    };

    let consumer: StreamConsumer = client_config
        .set("group.id", &CONFIG.group_id)
        .set("enable.partition.eof", "false")
        .set("auto.offset.reset", "earliest")
        .set("enable.auto.commit", "false")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()?;

    start_service(consumer).await?;

    Ok(())
}

#[cfg(test)]
static MOCK_SERVER: LazyLock<MockServer> = LazyLock::new(|| {
    let server = MockServer::start();
    println!("Starting mock server at {}", server.base_url());
    server
});

// Test Configuration
#[cfg(test)]
static CONFIG: LazyLock<Cli> = LazyLock::new(|| Cli {
    bootstrap_servers: "localhost:9094".to_string(),
    topic: "test-topic".to_string(),
    group_id: "test-group-id".to_string(),
    onkostar_uri: format!("{}/onkostar", MOCK_SERVER.base_url()),
    onkostar_username: None,
    onkostar_password: None,
    ssl_ca_file: None,
    ssl_cert_file: None,
    ssl_key_file: None,
    ssl_key_password: None,
});

#[cfg(test)]
mod tests {
    use crate::consent_idat::ConsentType;
    use crate::{MOCK_SERVER, send_consent};
    use httpmock::Method::PUT;

    #[tokio::test]
    async fn test_should_send_genomde_consent_to_onkostar() {
        let mock = MOCK_SERVER.mock(|when, then| {
            when.method(PUT)
                .path("/onkostar/x-api/patient/12345678/consent/mv64e");
            then.status(202);
        });

        let result = send_consent("12345678", ConsentType::GenomDe, "").await;

        mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_should_send_broad_consent_to_onkostar() {
        let mock = MOCK_SERVER.mock(|when, then| {
            when.method(PUT)
                .path("/onkostar/x-api/patient/12345678/consent/research");
            then.status(202);
        });

        let result = send_consent("12345678", ConsentType::BroadConsent, "").await;

        mock.assert();
        assert!(result.is_ok());
    }
}
