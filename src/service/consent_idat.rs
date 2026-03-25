use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::str::FromStr;

#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConsentIdat {
    pub(crate) consent_key: ConsentKey,
}

pub(crate) enum ConsentType {
    GenomDe,
    BroadConsent,
}

impl ConsentIdat {
    pub(crate) fn is_genomde(&self) -> bool {
        matches!(
            self.consent_key.consent_template_key,
            ConsentTemplateKey::GenomDe { .. }
        )
    }

    pub(crate) fn is_broad_consent(&self) -> bool {
        matches!(
            self.consent_key.consent_template_key,
            ConsentTemplateKey::BroadConsent { .. }
        )
    }

    pub(crate) fn patient_id(&self) -> String {
        self.consent_key
            .signer_ids
            .iter()
            .find_map(|id| match id {
                SignerId::PatientId(id) => Some(id.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }
}

impl FromStr for ConsentIdat {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConsentKey {
    pub(crate) consent_template_key: ConsentTemplateKey,
    pub(crate) signer_ids: Vec<SignerId>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum ConsentTemplateKey {
    GenomDe { version: String },
    BroadConsent { version: String },
    Other(String),
}

impl<'de> Deserialize<'de> for ConsentTemplateKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json: Value = Value::deserialize(deserializer)?;
        let domain_name = json["domainName"]
            .as_str()
            .ok_or_else(|| serde::de::Error::custom("Missing domainName"))?;
        let version = json["version"]
            .as_str()
            .ok_or_else(|| serde::de::Error::custom("Missing version"))?;

        match domain_name {
            "GenomDE_MV" => Ok(ConsentTemplateKey::GenomDe {
                version: version.to_string(),
            }),
            "MII" => Ok(ConsentTemplateKey::BroadConsent {
                version: version.to_string(),
            }),
            _ => Ok(ConsentTemplateKey::Other(domain_name.to_string())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum SignerId {
    PatientId(String),
    Other,
}

impl<'de> Deserialize<'de> for SignerId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json: Value = Value::deserialize(deserializer)?;
        let id_type = json["idType"]
            .as_str()
            .ok_or_else(|| serde::de::Error::custom("Missing idType"))?;
        let id = json["id"]
            .as_str()
            .ok_or_else(|| serde::de::Error::custom("Missing id"))?;

        match id_type {
            "Patienten-ID" => Ok(SignerId::PatientId(id.to_string())),
            _ => Ok(SignerId::Other),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::service::consent_idat::{ConsentIdat, ConsentTemplateKey, SignerId};

    #[test]
    fn test_deserialize_genomde_consent_template_key() {
        let json_str = r#"{"domainName": "GenomDE_MV", "version": "1.0"}"#;
        let actual: ConsentTemplateKey = serde_json::from_str(json_str).unwrap();
        assert_eq!(
            actual,
            ConsentTemplateKey::GenomDe {
                version: "1.0".to_string()
            }
        );
    }

    #[test]
    fn test_deserialize_broad_consent_template_key() {
        let json_str = r#"{"domainName": "MII", "version": "1.6.m"}"#;
        let actual: ConsentTemplateKey = serde_json::from_str(json_str).unwrap();
        assert_eq!(
            actual,
            ConsentTemplateKey::BroadConsent {
                version: "1.6.m".to_string()
            }
        );
    }

    #[test]
    fn test_deserialize_other_template_key() {
        let json_str = r#"{"domainName": "OTHER", "version": "1.6.m"}"#;
        let actual: ConsentTemplateKey = serde_json::from_str(json_str).unwrap();
        assert_eq!(actual, ConsentTemplateKey::Other("OTHER".to_string()));
    }

    #[test]
    fn test_deserialize_invalid_template_key() {
        let json_str = r#"{"domainNaMweeqw": "What?", "versionnix": "?"}"#;
        let actual = serde_json::from_str::<ConsentTemplateKey>(json_str);
        assert!(actual.is_err());
    }

    #[test]
    fn test_deserialize_patient_signer_id() {
        let json_str = r#"{"idType": "Patienten-ID", "id": "123456789"}"#;
        let signer_id: SignerId = serde_json::from_str(json_str).unwrap();
        assert_eq!(signer_id, SignerId::PatientId("123456789".to_string()));
    }

    #[test]
    fn test_deserialize_other_signer_id() {
        let json_str = r#"{"idType": "Other-ID", "id": "123456789"}"#;
        let signer_id: SignerId = serde_json::from_str(json_str).unwrap();
        assert_eq!(signer_id, SignerId::Other);
    }

    #[test]
    fn test_deserialize_invalid_signer_id() {
        let json_str = r#"{"id_TYPE": "Other-ID", "ID": "123456789"}"#;
        let actual = serde_json::from_str::<SignerId>(json_str);
        assert!(actual.is_err());
    }

    #[test]
    fn test_deserialize_genomde_consent_idat() {
        static JSON: &str = include_str!("../../resources/testdata/genom-de_consent.json");

        let actual: ConsentIdat = serde_json::from_str(JSON).unwrap();
        assert!(actual.is_genomde());
        assert_eq!(actual.patient_id(), "12345678");
    }

    #[test]
    fn test_deserialize_mii_consent_idat() {
        static JSON: &str = include_str!("../../resources/testdata/mii_consent.json");

        let actual: ConsentIdat = serde_json::from_str(JSON).unwrap();
        assert!(actual.is_broad_consent());
        assert_eq!(actual.patient_id(), "12345678");
    }
}
