use crate::service::consent_idat::ConsentType;
use reqwest::StatusCode;

pub(crate) struct HttpClient {
    base_url: String,
    username: String,
    password: Option<String>,
    client: reqwest::Client,
}

impl HttpClient {
    pub(crate) fn new(
        base_url: &str,
        credentials: Option<(&str, &str)>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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

        match credentials {
            Some((username, password)) => Ok(Self {
                base_url,
                username: username.to_string(),
                password: Some(password.to_string()),
                client,
            }),
            None => Ok(Self {
                base_url,
                username: String::new(),
                password: None,
                client,
            }),
        }
    }

    pub(crate) async fn send_consent(
        &self,
        patient_id: &str,
        consent_type: ConsentType,
        content: &str,
    ) -> Result<StatusCode, Box<dyn std::error::Error>> {
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

        let res = self
            .client
            .put(url.clone())
            .basic_auth(self.username.clone(), self.password.clone())
            .header("Content-Type", "application/json")
            .body(content.to_string())
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("Failed to send consent to '{}': {}", url, res.status()).into());
        }

        Ok(res.status())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use crate::service::consent_idat::ConsentType;
    use crate::service::http_client::HttpClient;
    use httpmock::Method::PUT;
    use httpmock::MockServer;

    #[tokio::test]
    async fn test_should_send_genomde_consent_to_onkostar() {
        let mock_server = MockServer::start();
        let mock = mock_server.mock(|when, then| {
            when.method(PUT)
                .path_prefix("/x-api/patient/12345678/consent/mv64e");
            then.status(202);
        });

        let http_client =
            HttpClient::new(&mock_server.base_url(), None).expect("Failed to create http client");

        let result = http_client
            .send_consent("12345678", ConsentType::GenomDe, "")
            .await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_should_send_broad_consent_to_onkostar() {
        let mock_server = MockServer::start();
        let mock = mock_server.mock(|when, then| {
            when.method(PUT)
                .path_prefix("/x-api/patient/12345678/consent/research");
            then.status(202);
        });

        let http_client =
            HttpClient::new(&mock_server.base_url(), None).expect("Failed to create http client");

        let result = http_client
            .send_consent("12345678", ConsentType::BroadConsent, "")
            .await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_should_send_consent_to_onkostar_using_http_basic_auth() {
        let mock_server = MockServer::start();
        let mock = mock_server.mock(|when, then| {
            when.method(PUT)
                .header("Authorization", "Basic dXNlcjpwYXNz")
                .path_prefix("/x-api/patient/12345678/consent/research");
            then.status(202);
        });

        let http_client = HttpClient::new(&mock_server.base_url(), Some(("user", "pass")))
            .expect("Failed to create http client");

        let result = http_client
            .send_consent("12345678", ConsentType::BroadConsent, "")
            .await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }
}
