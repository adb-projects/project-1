
use serde::{Serialize, Deserialize, Deserializer, de::Error};
use std::collections::{BTreeMap, HashSet};
use std::vec::Vec;
use chrono::{DateTime, NaiveDate, FixedOffset, Utc};

use crate::model::{NormalizationError, NormalizeData, NormalizeScore, Provider};

fn parse_assessment_date(date: &str) -> Option<NaiveDate>
{
    if let Ok(date_time) = NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
        return Some(date_time);
    }

    None
}

type ValidationFunc = fn(&BTreeMap<String, String>) -> bool;
const ID: (&str, ValidationFunc) = 
    ("patient_id", |data| data.contains_key(ID.0) && data[ID.0].len() > 0);
const DATE: (&str, ValidationFunc) = 
    ("assessment_date", |data| data.contains_key(DATE.0) && 
        parse_assessment_date(&data[DATE.0]).is_some());
const METRIC: (&str, ValidationFunc) = 
    ("metric_name", |data| data.contains_key(METRIC.0) && data[METRIC.0].len() > 0);
const VALUE: (&str, ValidationFunc) = 
    ("metric_value", |data| {
        if !data.contains_key(VALUE.0) {
            return false;
        }
        if let Ok(val) = data[VALUE.0].parse::<i64>() {
            return val >= 0 && val <= 100;
        }

        return false;
    });
const CATEGORY: (&str, ValidationFunc) = 
    ("category", |data| data.contains_key(CATEGORY.0) && data[CATEGORY.0].len() > 0);

pub struct ProviderHandler {
    pub data: Vec<BTreeMap<String, String>>,
    pub error_index: HashSet<usize>,
}

impl ProviderHandler {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            error_index: HashSet::new(),
        }
    }

    pub fn name() -> String {"c".into()}
}

impl Provider for ProviderHandler {
    fn get_metadata(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("sourceProvider".to_string(), "provider_c".to_string()),
            ("sourceFormat".to_string(), "csv".to_string()),
            ("ingestedAt".to_string(), Utc::now().to_rfc3339(),),
            ("version".to_string(), "1.0".to_string()),
        ])
    }  

    fn parse(&mut self, data: &str) -> Result<(), NormalizationError>  {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(data.as_bytes());
    
        let mut headers: Vec<String> = Vec::new();
        let maybe_headers = rdr.headers();
        if let Ok(rdr_headers) = maybe_headers {
            for value in rdr_headers {
                headers.push(value.into());
            }
        } else {
            return Err(NormalizationError::Parse("Missing header row".into()));
        }


        let mut result = rdr.records().next();
        let header_count = headers.len();
        while result.is_some() {
            if let Ok(values) = result.unwrap() {
                if values.len() != header_count {
                    return Err(NormalizationError::Parse("Field count is not equal to header count".into()));
                }

                let mut row = BTreeMap::new();
                let mut index = 0;
                for value in &values {
                    row.insert(headers[index].clone(), value.trim().into());
                    index += 1;
                }

                self.data.push(row);
            }
            result = rdr.records().next();
        }

        Ok(())
    }

    fn validate(&mut self) -> NormalizationError {
        let mut output = Vec::new();
        for (index, data) in self.data.iter().enumerate() {
            if !ID.1(data) ||
                !DATE.1(data) ||
                !METRIC.1(data) ||
                !VALUE.1(data) ||
                !CATEGORY.1(data) {
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

            let id = data[ID.0].clone();
            if !patients.contains_key(&id) {
                patients.insert(id.clone(), BTreeMap::new());
            }
            if let Some(assessments) = patients.get_mut(&id) {
                let assessment_type = data[CATEGORY.0].clone();
                if !assessments.contains_key(&assessment_type) {
                    assessments.insert(assessment_type.clone(), 
                        NormalizeData::new(id, assessment_type.clone(), Utc::now(), metadata.clone()));
                }

                if let Some(normalized_data) = assessments.get_mut(&assessment_type) {
                    let dimension = data[METRIC.0].clone();
                    let value = data[VALUE.0].parse::<i64>().unwrap();
                    normalized_data.scores.push(NormalizeScore {
                        dimension: dimension.to_string(),
                        value: value * 10,
                        scale: "0-100".into(),
                    });
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
    fn provider_c_test() {
        let csv_c = "patient_id,assessment_date,metric_name,metric_value,category\n
            P123c,2024-10-15,attention_span,6,behavioral
            P123c,2024-10-15,social_engagement,4,behavioral
            P124c,2024-10-15,social_engagement,4,behavioral";

        let mut handler = ProviderHandler::new();
        let provider: &mut dyn Provider = &mut handler as &mut dyn Provider;
        assert_eq!(provider.parse(csv_c).is_ok(), true);
        assert_eq!(provider.validate(), NormalizationError::None);
        let converted = provider.convert();
        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0].scores.len(), 2);
        
    }
}
