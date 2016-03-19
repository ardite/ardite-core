// TODO: `build.rs` sucks…
// TODO: Ordered maps, `BTreeMap` does its own ordering.

use std::collections::BTreeMap;

/// SchemaType used to deserialize data files into a usable definition type.
#[derive(Deserialize)]
pub struct SerdeDefinition {
  pub types: BTreeMap<String, SerdeSchema>
}

/// Intermediary type used to deserialized data files into a usable schema
/// enum.
#[derive(Deserialize)]
pub struct SerdeSchema {
  #[serde(rename="type")]
  pub type_: Option<String>,
  #[serde(rename="multipleOf")]
  pub multiple_of: Option<f32>,
  pub minimum: Option<f64>,
  #[serde(rename="exclusiveMinimum")]
  pub exclusive_minimum: Option<bool>,
  pub maximum: Option<f64>,
  #[serde(rename="exclusiveMaximum")]
  pub exclusive_maximum: Option<bool>,
  #[serde(rename="minLength")]
  pub min_length: Option<u64>,
  #[serde(rename="maxLength")]
  pub max_length: Option<u64>,
  pub pattern: Option<String>,
  pub items: Option<Box<SerdeSchema>>,
  pub properties: Option<BTreeMap<String, SerdeSchema>>,
  pub required: Option<Vec<String>>,
  #[serde(rename="additionalProperties")]
  pub additional_properties: Option<bool>,
  #[serde(rename="enum")]
  pub enum_: Option<Vec<String>>
}
