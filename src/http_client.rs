use crate::consent_idat::ConsentType;

pub(crate) struct HttpClient {
    base_url: String,
    client: reqwest::Client,
}

impl HttpClient {
    pub(crate) fn new(base_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let user_agent_string = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        let client = reqwest::Client::builder()
            .user_agent(user_agent_string)
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| e.to_string())?;

        let base_url = if base_url.ends_with('/') {
            base_url[0..&base_url.len() - 1].to_string()
        } else {
            base_url.to_string()
        };

        Ok(Self { base_url, client })
    }

    pub(crate) async fn send_consent(
        &self,
        patient_id: &str,
        consent_type: ConsentType,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = match consent_type {
            ConsentType::GenomDe => {
                format!("{}/x-api/patient/{patient_id}/consent/mv64e", self.base_url)
            }
            ConsentType::BroadConsent => {
                format!(
                    "{}/x-api/patient/{patient_id}/consent/research",
                    self.base_url
                )
            }
        };

        let _ = self
            .client
            .put(url)
            .json(&content)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
