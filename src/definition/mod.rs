pub mod schema;
pub mod serde;

use definition::schema::Schema;

#[derive(PartialEq, Debug)]
pub struct Definition {
  pub data: Schema
}
