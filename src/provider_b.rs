
use serde::{Serialize, Deserialize, Deserializer, de::Error};
use std::collections::{BTreeMap, HashSet};
use chrono::{DateTime, NaiveDate, FixedOffset, Utc};

use crate::model::{NormalizationError, NormalizeData, NormalizeScore, Provider};

type ValidationFunc = fn(&BTreeMap<String, serde_json::Value>) -> bool;
const ID: (&str, ValidationFunc) = 
    ("patient_id", |data| data.contains_key(ID.0) && data[ID.0].is_string());
const NAME: (&str, ValidationFunc) = 
    ("patient_name", |data| data.contains_key(NAME.0) && data[NAME.0].is_string());
const TYPE: (&str, ValidationFunc) = 
    ("assessment_type", |data| data.contains_key(TYPE.0) && data[TYPE.0].is_string());
const NOTES: (&str, ValidationFunc) = 
    ("notes", |data| data.contains_key(NOTES.0) && data[NOTES.0].is_string());

#[derive(Clone, Serialize, Deserialize)]
pub struct ProviderHandler {
    pub data: Vec<BTreeMap<String, serde_json::Value>>,
    pub error_index: HashSet<usize>,
}

impl ProviderHandler {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            error_index: HashSet::new(),
        }
    }

    pub fn name() -> String {"b".into()}
}

impl Provider for ProviderHandler {
    fn get_metadata(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("sourceProvider".to_string(), "provider_b".to_string()),
            ("sourceFormat".to_string(), "flat_key_value".to_string()),
            ("ingestedAt".to_string(), Utc::now().to_rfc3339(),),
            ("version".to_string(), "1.0".to_string()),
        ])
    }  

    fn parse(&mut self, data: &str) -> Result<(), NormalizationError>  {
        match serde_json::from_str::<Vec<BTreeMap<String, serde_json::Value>>>(data) {
            Ok(result) => {
                self.data = result;
                Ok(())
            },
            Err(err) => Err(NormalizationError::Parse(err.to_string())),
        }
    }

    fn validate(&mut self) -> NormalizationError {
        let mut output = Vec::new();
        for (index, data) in self.data.iter().enumerate() {
            if !ID.1(data) ||
                !NAME.1(data) ||
                !TYPE.1(data) ||
                !NOTES.1(data) {
                output.push(NormalizationError::Validate("Data is invalid".into(), index));
                self.error_index.insert(index);
                continue;
            }

            for key in data.keys().filter(|x| x.starts_with("score_")) {
                if !data[key].is_i64() {
                    output.push(NormalizationError::Validate("Data is invalid".into(), index));
                    self.error_index.insert(index);
                    break;
                }
                if let None = data[key].as_i64() {
                    output.push(NormalizationError::Validate("Data is invalid".into(), index));
                    self.error_index.insert(index);
                    break;
                }
            }
        }
        if output.len() > 0 {NormalizationError::Aggregate(output)} else {NormalizationError::None}
    }

    fn convert(&self) -> Vec<NormalizeData> {
        let metadata = self.get_metadata();
        let mut patients: BTreeMap<String, BTreeMap<String, NormalizeData>> = BTreeMap::new();
        for (index, data) in self.data.iter().enumerate() {
            if self.error_index.contains(&index) {
                continue;
            }

            let id = data[ID.0].as_str().unwrap().to_string();
            if !patients.contains_key(&id) {
                patients.insert(id.clone(), BTreeMap::new());
            }
            if let Some(assessments) = patients.get_mut(&id) {
                let assessment_type = data[TYPE.0].as_str().unwrap().to_string();
                if !assessments.contains_key(&assessment_type) {
                    assessments.insert(assessment_type.clone(), 
                        NormalizeData::new(id, assessment_type.clone(), Utc::now(), metadata.clone()));
                }

                if let Some(normalized_data) = assessments.get_mut(&assessment_type) {
                    for key in data.keys().filter(|x| x.starts_with("score_")) {
                        let dimension = &key["score_".len()..];
                        let value = data[key].as_i64().unwrap();
                        normalized_data.scores.push(NormalizeScore {
                            dimension: dimension.to_string(),
                            value,
                            scale: "0-100".into(),
                        });
                    }
                }
            }
        }

        let mut output = Vec::new();
        for (_, assessments) in patients {
            for (_, assessment) in assessments {
                output.push(assessment);
            }
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_b_test() {
        let json_str = r#"[{
            "patient_id": "P123b",
            "patient_name": "last",
            "assessment_type": "cognitive",
            "score_memory": 85,
            "score_processing": 72,
            "notes": "..."
        }]"#;

        let mut handler = ProviderHandler::new();
        let provider: &mut dyn Provider = &mut handler as &mut dyn Provider;
        assert_eq!(provider.parse(json_str).is_ok(), true);
        assert_eq!(provider.validate(), NormalizationError::None);
        let converted = provider.convert();
        assert_eq!(converted.len(), 1);
        assert_eq!(converted[0].scores.len(), 2);
    }
}
