
use serde::{Serialize, Deserialize, Deserializer, de::Error};
use std::collections::{BTreeMap, HashSet};
use chrono::{DateTime, NaiveDate, FixedOffset, Utc};
use chrono::ParseError;

use crate::model::{NormalizationError, NormalizeData, NormalizeScore, Provider};

fn parse_dob<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let parsed_str = String::deserialize(deserializer)?;
    if let Ok(date_time) = NaiveDate::parse_from_str(&parsed_str, "%Y%m%d") {
        return Ok(Some(date_time));
    }

    return Ok(None);
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Patient {
    pub id: String,
    pub name: String,
    #[serde(deserialize_with = "parse_dob")]
    pub dob: Option<NaiveDate>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Assessment {
    #[serde(rename = "type")] 
    pub type_: String,
    pub scores: BTreeMap<String, i64>,
    pub notes: String,

}

#[derive(Clone, Serialize, Deserialize)]
pub struct Data {
    pub patient: Patient,
    pub assessment: Assessment,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProviderHandler {
    pub data: Vec<Data>,
    pub error_index: HashSet<usize>,
}

impl ProviderHandler {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            error_index: HashSet::new(),
        }
    }

    pub fn name() -> String {"a".into()}
}

impl Provider for ProviderHandler {      
    fn get_metadata(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("sourceProvider".to_string(), "provider_a".to_string()),
            ("sourceFormat".to_string(), "nested_json".to_string()),
            ("ingestedAt".to_string(), Utc::now().to_rfc3339(),),
            ("version".to_string(), "1.0".to_string()),
        ])
    }  

    fn parse(&mut self, data: &str) -> Result<(), NormalizationError>  {
        match serde_json::from_str::<Vec<Data>>(data) {
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
            if data.patient.id.len() == 0 &&
                data.patient.name.len() == 0 &&
                data.patient.dob.is_none() && 
                data.assessment.type_.len() == 0 &&
                data.assessment.scores.len() == 0 && 
                data.assessment.notes.len() == 0 {
                output.push(NormalizationError::Validate("Data is invalid".into(), index));
                self.error_index.insert(index);
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

            let id = data.patient.id.clone();
            if !patients.contains_key(&id) {
                patients.insert(id.clone(), BTreeMap::new());
            }
            if let Some(assessments) = patients.get_mut(&id) {
                let assessment_type = data.assessment.type_.clone();
                if !assessments.contains_key(&assessment_type) {
                    assessments.insert(assessment_type.clone(), 
                        NormalizeData::new(id, assessment_type.clone(), Utc::now(), metadata.clone()));
                }

                if let Some(normalized_data) = assessments.get_mut(&assessment_type) {
                    for (dimension, value) in data.assessment.scores.iter() {
                        normalized_data.scores.push(NormalizeScore {
                            dimension: dimension.to_string(),
                            value: value * 10,
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
    fn provider_a_test() {
        let json_str = r#"[{
            "patient": {"id": "P123a", "name": "test", "dob": "20260221"},
            "assessment": {
            "type": "behavioral_screening",
            "scores": {"anxiety": 7, "social": 4, "attention": 6},
            "notes": "This is a note"
            }
        }]"#;

        let mut handler = ProviderHandler::new();
        let provider: &mut dyn Provider = &mut handler as &mut dyn Provider;
        assert_eq!(provider.parse(json_str).is_ok(), true);
        assert_eq!(provider.validate(), NormalizationError::None);
        let converted = provider.convert();
        assert_eq!(converted.len(), 1);
        assert_eq!(converted[0].scores.len(), 3);
    }
}
