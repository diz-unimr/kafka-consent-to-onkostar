mod cli;
mod consent_idat;
mod http_client;

use crate::cli::Cli;
use crate::consent_idat::ConsentType;

use futures::TryStreamExt;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::{ClientConfig, Message};
use std::error::Error;
use std::sync::LazyLock;
use tracing::{error, info};

use crate::http_client::HttpClient;

#[cfg(not(test))]
use clap::Parser;
use tokio::time;

#[cfg(not(test))]
static CONFIG: LazyLock<Cli> = LazyLock::new(Cli::parse);

async fn start_service(consumer: StreamConsumer, http_client: &HttpClient) -> Result<(), String> {
    let stream = consumer
        .stream()
        .map_err(|e| e.to_string())
        .try_for_each(|msg| async move {
            let key = msg.key().unwrap_or_default();
            let key_str = std::str::from_utf8(key).unwrap_or_default();

            let message = msg.payload().unwrap_or_default();
            let message_str = std::str::from_utf8(message).unwrap_or_default();

            let consent_idat: consent_idat::ConsentIdat = match serde_json::from_str(message_str) {
                Ok(idat) => idat,
                Err(e) => {
                    error!("Failed to parse consent IDAT: {e}");
                    return Err(format!(
                        "Failed to parse consent IDAT für message '{key_str}': {e}"
                    ));
                }
            };
            let patient_id = consent_idat.patient_id();

            if consent_idat.is_genomde() {
                return match http_client
                    .send_consent(&patient_id, ConsentType::GenomDe, message_str)
                    .await
                {
                    Ok(()) => {
                        info!("GenomDE consent for '{patient_id}' sent to Onkostar");
                        time::sleep(time::Duration::from_secs(1)).await;
                        Ok(())
                    }
                    Err(e) => {
                        error!("Skipping GenomDE consent - {e}");
                        Ok(())
                    }
                };
            } else if consent_idat.is_broad_consent() {
                return match http_client
                    .send_consent(&patient_id, ConsentType::BroadConsent, message_str)
                    .await
                {
                    Ok(()) => {
                        info!("MII consent for '{patient_id}' sent to Onkostar");
                        time::sleep(time::Duration::from_secs(1)).await;
                        Ok(())
                    }
                    Err(e) => {
                        error!("Skipping MII consent - {e}");
                        Ok(())
                    }
                };
            }

            info!("Consent '{key_str}' for '{patient_id}' is not a GenomDE or MII consent");

            Ok(())
        });

    info!("Starting kafka consumer");
    let err = stream.await;
    info!("Stopping kafka consumer");
    err
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
        .set("auto.offset.reset", "earliest")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()?;

    let topic: &str = &CONFIG.topic.clone();
    consumer.subscribe(&[topic])?;

    let http_client = if let Some(username) = &CONFIG.onkostar_username
        && let Some(password) = &CONFIG.onkostar_password
    {
        HttpClient::new(&CONFIG.onkostar_uri, Some((username, password)))?
    } else {
        HttpClient::new(&CONFIG.onkostar_uri, None)?
    };

    start_service(consumer, &http_client).await?;

    Ok(())
}

// Test Configuration
#[cfg(test)]
#[allow(clippy::expect_used)]
static CONFIG: LazyLock<Cli> = LazyLock::new(|| Cli {
    bootstrap_servers: "localhost:9094".to_string(),
    topic: "test-topic".to_string(),
    group_id: "test-group-id".to_string(),
    onkostar_uri: "http://localhost:8080/onkostar".to_string(),
    onkostar_username: None,
    onkostar_password: None,
    ssl_ca_file: None,
    ssl_cert_file: None,
    ssl_key_file: None,
    ssl_key_password: None,
});

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use crate::http_client::HttpClient;
    use crate::start_service;
    use httpmock::Method::PUT;
    use httpmock::MockServer;
    use rdkafka::ClientConfig;
    use rdkafka::consumer::{Consumer, StreamConsumer};
    use rdkafka::mocking::MockCluster;
    use rdkafka::producer::{FutureProducer, FutureRecord};
    use rstest::rstest;
    use std::fs;
    use std::time::Duration;

    #[rstest]
    #[case(
        "resources/testdata/genom-de_consent.json",
        "/x-api/patient/12345678/consent/mv64e"
    )]
    #[case(
        "resources/testdata/mii_consent.json",
        "/x-api/patient/12345678/consent/research"
    )]
    #[tokio::test]
    async fn test_should_handle_kafka_record(#[case] file: &str, #[case] expected_path: &str) {
        let mock_server = MockServer::start();
        let mock = mock_server.mock(|when, then| {
            when.method(PUT).path(expected_path);
            then.status(202);
        });

        let http_client =
            HttpClient::new(&mock_server.base_url(), None).expect("Failed to create http client");

        let mock_cluster = MockCluster::new(1).expect("Failed to create mock cluster");
        let bootstrap = mock_cluster.bootstrap_servers();

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &bootstrap)
            .set("group.id", "test-group")
            .set("auto.offset.reset", "earliest")
            .create()
            .expect("Failed to create consumer");
        consumer.subscribe(&["test-topic"]).expect("subscriber");

        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &bootstrap)
            .create()
            .expect("Failed to create producer");

        await_stable_mock_cluster(&producer, &consumer).await;

        let json = fs::read_to_string(file).expect("Failed to read file");
        producer
            .send(
                FutureRecord::to("test-topic").payload(&json).key("random"),
                Duration::from_secs(0),
            )
            .await
            .expect("Failed to send record");

        let handle = start_service(consumer, &http_client);

        await_service_stop(&producer, handle).await;

        // Assert that the mock server received the expected request
        mock.assert();
    }

    #[allow(clippy::panic)]
    #[tokio::test]
    async fn test_should_stop_service_on_bad_payload() {
        let http_client =
            HttpClient::new("https://???:1234567", None).expect("Failed to create http client");

        let mock_cluster = MockCluster::new(1).expect("Failed to create mock cluster");
        let bootstrap = mock_cluster.bootstrap_servers();

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &bootstrap)
            .set("group.id", "test-group")
            .set("auto.offset.reset", "earliest")
            .create()
            .expect("Failed to create consumer");
        consumer.subscribe(&["test-topic"]).expect("subscriber");

        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &bootstrap)
            .create()
            .expect("Failed to create producer");

        await_stable_mock_cluster(&producer, &consumer).await;

        producer
            .send(
                FutureRecord::to("test-topic")
                    .payload("bad payload")
                    .key("random"),
                Duration::from_secs(0),
            )
            .await
            .expect("Failed to send record");

        let result = start_service(consumer, &http_client).await;
        assert!(result.is_err());
    }

    #[rstest]
    #[case("/x-api/patient/12345678/consent/mv64e")]
    #[case("/x-api/patient/12345678/consent/research")]
    #[tokio::test]
    async fn test_should_not_send_request(#[case] expected_path: &str) {
        let mock_server = MockServer::start();
        let mock = mock_server.mock(|when, then| {
            when.method(PUT).path(expected_path);
            then.status(202);
        });

        let http_client =
            HttpClient::new(&mock_server.base_url(), None).expect("Failed to create http client");

        let mock_cluster = MockCluster::new(1).expect("Failed to create mock cluster");
        let bootstrap = mock_cluster.bootstrap_servers();

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &bootstrap)
            .set("group.id", "test-group")
            .set("auto.offset.reset", "earliest")
            .create()
            .expect("Failed to create consumer");
        consumer.subscribe(&["test-topic"]).expect("subscriber");

        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &bootstrap)
            .create()
            .expect("Failed to create producer");

        await_stable_mock_cluster(&producer, &consumer).await;

        let handle = start_service(consumer, &http_client);

        await_service_stop(&producer, handle).await;

        // Assert that the mock server received no request
        mock.assert_calls(0);
    }

    async fn await_stable_mock_cluster(producer: &FutureProducer, consumer: &StreamConsumer) {
        // Wait for Consumer to get ready
        producer
            .send(
                FutureRecord::to("test-topic")
                    .payload("initial-payload")
                    .key("random"),
                Duration::from_secs(0),
            )
            .await
            .expect("Failed to send initial record");
        let _ = consumer.recv().await;
        // Consumer is ready to receive messages from Mock Cluster
    }

    async fn await_service_stop(producer: &FutureProducer, handle: impl Future) {
        // Bad record content to stop the service
        producer
            .send(
                FutureRecord::to("test-topic")
                    .payload("bad payload")
                    .key("random"),
                Duration::from_secs(0),
            )
            .await
            .expect("Failed to send record");

        handle.await;
    }
}
