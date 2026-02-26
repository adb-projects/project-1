
use serde::{Serialize, Deserialize, Deserializer, de::Error};
use std::collections::{BTreeMap, HashSet};
use chrono::{DateTime, NaiveDate, FixedOffset, Utc};

#[derive(Clone, Serialize, Deserialize)]
pub struct NormalizeScore {
    pub dimension: String,
    pub value: i64,
    pub scale: String,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct NormalizeData {
    pub patientId: String,
    pub assessmentDate: String,
    pub assessmentType: String,
    pub scores: Vec<NormalizeScore>,
    pub metadata: BTreeMap<String, String>,
}

impl NormalizeData {
    pub fn new(
        patientId: String,
        assessmentType: String,
        date: DateTime<Utc>,
        metadata: BTreeMap<String, String>) -> Self {
        Self {
            patientId,
            assessmentType,
            assessmentDate: date.to_rfc3339(),
            scores: Vec::new(),
            metadata: metadata,
        }

    }
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub enum NormalizationError{
    None,
    Parse(String),
    Validate(String, usize),
    Aggregate(Vec<NormalizationError>),
    Unknown(String),
}

pub trait Provider {
    fn get_metadata(&self) -> BTreeMap<String, String>;
    fn parse(&mut self, data: &str) -> Result<(), NormalizationError>;
    fn validate(&mut self) -> NormalizationError;
    fn convert(&self) -> Vec<NormalizeData>;
}
