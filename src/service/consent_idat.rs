use crate::service::consent_fhir_idat::ConsentFhirIdat;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::str::FromStr;
use std::string::ToString;

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConsentIdat {
    pub(crate) consent_key: ConsentKey,
    pub(crate) current_policy_states: Vec<PolicyState>,
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

impl From<ConsentFhirIdat> for ConsentIdat {
    fn from(consent_fhir_idat: ConsentFhirIdat) -> Self {
        ConsentIdat {
            consent_key: ConsentKey {
                consent_template_key: if consent_fhir_idat.is_genomde() {
                    ConsentTemplateKey::GenomDe {
                        version: String::new(),
                    }
                } else if consent_fhir_idat.is_broad_consent() {
                    ConsentTemplateKey::BroadConsent {
                        version: String::new(),
                    }
                } else {
                    ConsentTemplateKey::Other("Unknown".to_string())
                },
                signer_ids: vec![SignerId::PatientId(consent_fhir_idat.patient_id())],
                consent_date: consent_fhir_idat.consent_date(),
            },
            current_policy_states: if consent_fhir_idat.is_genomde() {
                vec![
                    PolicyState {
                        key: PolicyStateKey {
                            domain_name: "GenomDE_MV".to_string(),
                            name: "sequencing".to_string(),
                            version: String::new(),
                        },
                        value: consent_fhir_idat.policy_state("sequencing"),
                    },
                    PolicyState {
                        key: PolicyStateKey {
                            domain_name: "GenomDE_MV".to_string(),
                            name: "case-identification".to_string(),
                            version: String::new(),
                        },
                        value: consent_fhir_idat.policy_state("case-identification"),
                    },
                    PolicyState {
                        key: PolicyStateKey {
                            domain_name: "GenomDE_MV".to_string(),
                            name: "reidentification".to_string(),
                            version: String::new(),
                        },
                        value: consent_fhir_idat.policy_state("reidentification"),
                    },
                ]
            } else {
                // Not required for broad consent
                vec![]
            },
        }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConsentKey {
    pub(crate) consent_template_key: ConsentTemplateKey,
    pub(crate) signer_ids: Vec<SignerId>,
    pub(crate) consent_date: String,
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

impl Serialize for ConsentTemplateKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ConsentTemplateKey", 3)?;
        match self {
            ConsentTemplateKey::GenomDe { version } => {
                let _ = state.serialize_field("domainName", "GenomDE_MV");
                let _ = state.serialize_field("name", "Teilnahmeerklärung GenomDE MV");
                let _ = state.serialize_field("version", version);
            }
            ConsentTemplateKey::BroadConsent { version } => {
                let _ = state.serialize_field("domainName", "MII");
                let _ = state.serialize_field("name", "Teilnahmeerklärung GenomDE MV");
                let _ = state.serialize_field("version", version);
            }
            ConsentTemplateKey::Other(value) => {
                let _ = state.serialize_field("domainName", value);
                let _ = state.serialize_field("name", "Sonstiger Consent");
                let _ = state.serialize_field("version", "");
            }
        }
        state.end()
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum SignerId {
    PatientId(String),
    Other,
}

impl Serialize for SignerId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("SignerId", 2)?;
        match self {
            SignerId::PatientId(id) => {
                let _ = state.serialize_field("idType", "Patienten-ID");
                let _ = state.serialize_field("id", id);
            }
            SignerId::Other => {
                let _ = state.serialize_field("idType", "Other-ID");
                let _ = state.serialize_field("id", "");
            }
        }
        state.end()
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PolicyState {
    key: PolicyStateKey,
    value: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PolicyStateKey {
    domain_name: String,
    name: String,
    version: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::service::consent_fhir_idat::ConsentFhirIdat;
    use crate::service::consent_idat::{
        ConsentIdat, ConsentTemplateKey, PolicyState, PolicyStateKey, SignerId,
    };
    use std::str::FromStr;

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

    #[test]
    fn test_should_map_genomde_consent_fhir() {
        static JSON: &str = include_str!("../../resources/testdata/genom-de_consent_fhir.json");
        let fhir = ConsentFhirIdat::from_str(JSON).unwrap();

        let actual = ConsentIdat::from(fhir);
        assert!(actual.is_genomde());
        assert_eq!(actual.patient_id(), "12345678");
        assert_eq!(actual.consent_key.consent_date, "2026-03-17 12:00:00");
        assert_eq!(
            actual.consent_key.signer_ids[0],
            SignerId::PatientId("12345678".to_string())
        );
        assert_eq!(actual.current_policy_states.len(), 3);
        assert_eq!(
            actual.current_policy_states,
            vec![
                PolicyState {
                    key: PolicyStateKey {
                        domain_name: "GenomDE_MV".to_string(),
                        name: "sequencing".to_string(),
                        version: String::new(),
                    },
                    value: true,
                },
                PolicyState {
                    key: PolicyStateKey {
                        domain_name: "GenomDE_MV".to_string(),
                        name: "case-identification".to_string(),
                        version: String::new(),
                    },
                    value: true,
                },
                PolicyState {
                    key: PolicyStateKey {
                        domain_name: "GenomDE_MV".to_string(),
                        name: "reidentification".to_string(),
                        version: String::new(),
                    },
                    value: true,
                }
            ]
        );
    }
}
