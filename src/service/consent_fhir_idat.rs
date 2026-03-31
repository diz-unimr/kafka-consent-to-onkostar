use fhir_model::r4b::resources::{Bundle, Resource};
use fhir_model::r4b::types::ExtensionValue;
use std::str::FromStr;

pub struct ConsentFhirIdat {
    bundle: Bundle,
}

impl ConsentFhirIdat {
    pub(crate) fn is_genomde(&self) -> bool {
        if self.bundle.entry.len() < 2 {
            return false;
        }

        for entry in self.bundle.entry.clone() {
            if let Some(entry) = entry
                && let Some(Resource::Consent(consent)) = entry.resource.clone()
            {
                for extension in consent.extension.clone() {
                    for extension in extension.extension.clone() {
                        if let Some(extension_value) = extension.value.clone()
                            && let ExtensionValue::Reference(reference) = extension_value
                            && let Some(reference) = reference.reference.clone()
                        {
                            return reference.contains("ResearchStudy?identifier=https://fhir.diz.uni-marburg.de/fhir/sid/consent-domain-id|GenomDE_MV");
                        }
                    }
                }
            }
        }

        false
    }

    pub(crate) fn is_broad_consent(&self) -> bool {
        if self.bundle.entry.len() < 2 {
            return false;
        }

        for entry in self.bundle.entry.clone() {
            if let Some(entry) = entry
                && let Some(Resource::Consent(consent)) = entry.resource.clone()
            {
                for extension in consent.extension.clone() {
                    for extension in extension.extension.clone() {
                        if let Some(extension_value) = extension.value.clone()
                            && let ExtensionValue::Reference(reference) = extension_value
                            && let Some(reference) = reference.reference.clone()
                        {
                            return reference.contains("ResearchStudy?identifier=https://fhir.diz.uni-marburg.de/fhir/sid/consent-domain-id|MII");
                        }
                    }
                }
            }
        }

        false
    }

    pub(crate) fn patient_id(&self) -> String {
        for entry in self.bundle.entry.clone() {
            if let Some(entry) = entry
                && let Some(Resource::Consent(consent)) = entry.resource.clone()
            {
                while let Some(patient) = &consent.patient {
                    if let Some(patient_reference) = patient.reference.clone() {
                        return patient_reference
                            .split('|')
                            .next_back()
                            .unwrap_or_default()
                            .to_string();
                    }
                }
            }
        }
        String::new()
    }
}

impl FromStr for ConsentFhirIdat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bundle = serde_json::from_str::<Bundle>(s).map_err(|e| e.to_string())?;
        Ok(Self { bundle })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::service::consent_fhir_idat::ConsentFhirIdat;
    use std::str::FromStr;

    #[test]
    fn test_deserialize_genomde_consent_fhir_idat() {
        static JSON: &str = include_str!("../../resources/testdata/genom-de_consent_fhir.json");

        let actual = ConsentFhirIdat::from_str(JSON).unwrap();
        assert!(actual.is_genomde());
        assert_eq!(actual.patient_id(), "12345678");
    }

    #[test]
    fn test_deserialize_mii_consent_fhir_idat() {
        static JSON: &str = include_str!("../../resources/testdata/mii_consent_fhir.json");

        let actual = ConsentFhirIdat::from_str(JSON).unwrap();
        assert!(actual.is_broad_consent());
        assert_eq!(actual.patient_id(), "12345678");
    }
}
