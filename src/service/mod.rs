use crate::service::consent_fhir_idat::ConsentFhirIdat;
use crate::service::consent_idat::ConsentIdat;
use consent_idat::ConsentType;
use futures::TryStreamExt;
use http_client::HttpClient;
use rdkafka::Message;
use rdkafka::consumer::StreamConsumer;
use reqwest::StatusCode;
use std::str::FromStr;
use tokio::time;
use tracing::{error, info};

mod consent_fhir_idat;
mod consent_idat;
pub mod http_client;

pub async fn start(consumer: StreamConsumer, http_client: &HttpClient) -> Result<(), String> {
    let stream = consumer
        .stream()
        .map_err(|e| e.to_string())
        .try_for_each(|msg| async move {
            let key = msg.key().unwrap_or_default();
            let key_str = std::str::from_utf8(key).unwrap_or_default();

            let message = msg.payload().unwrap_or_default();
            let message_str = std::str::from_utf8(message).unwrap_or_default();

            let consent_fhir_idat = match ConsentFhirIdat::from_str(message_str) {
                Ok(idat) => idat,
                Err(e) => {
                    error!("Failed to parse consent FHIR IDAT: {e}");
                    return Err(format!(
                        "Failed to parse consent FHIR IDAT für message '{key_str}': {e}"
                    ));
                }
            };
            let patient_id = &consent_fhir_idat.patient_id();

            if consent_fhir_idat.is_genomde() {
                let Ok(message_str) = serde_json::to_string(&ConsentIdat::from(consent_fhir_idat))
                else {
                    return Err(format!("Could not serialize JSON IDAT for: '{key_str}'"));
                };

                return match http_client
                    .send_consent(patient_id, ConsentType::GenomDe, &message_str)
                    .await
                {
                    Ok(status_code) => {
                        info!(
                            "GenomDE consent for '{patient_id}' sent to Onkostar: {}",
                            match status_code {
                                StatusCode::CREATED => "Procedure created",
                                StatusCode::ACCEPTED => "Procedure updated",
                                _ => "Unknown Status",
                            }
                        );
                        time::sleep(time::Duration::from_secs(1)).await;
                        Ok(())
                    }
                    Err(e) => {
                        error!("Skipping GenomDE consent - {e}");
                        Ok(())
                    }
                };
            } else if consent_fhir_idat.is_broad_consent() {
                let Ok(message_str) = serde_json::to_string(&ConsentIdat::from(consent_fhir_idat))
                else {
                    return Err(format!("Could not serialize JSON IDAT for: '{key_str}'"));
                };

                return match http_client
                    .send_consent(patient_id, ConsentType::BroadConsent, &message_str)
                    .await
                {
                    Ok(status_code) => {
                        info!(
                            "MII consent for '{patient_id}' sent to Onkostar: {}",
                            match status_code {
                                StatusCode::CREATED => "Procedure created",
                                StatusCode::ACCEPTED => "Procedure updated",
                                _ => "Unknown Status",
                            }
                        );
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
