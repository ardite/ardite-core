//! Types representing for data which will be retrieved from the database.
//! Currently this data is expected to look like a JSON object but this may be
//! changed in the future. Driver authors must cast the data they retrieve from
//! the database to these types.

/// Represents a property of a collection document.
pub type Property = String;

/// A document from the database is just a JSON object.
pub type Document = Value::Object;

/// Various value types. Based on types in the [JSON standard][1] (see section
/// 5).
///
/// [1]: http://ecma-international.org/publications/files/ECMA-ST/ECMA-404.pdf
pub enum Value {
  Object(Object),
  Array(Array),
  Number(f64), // TODO: @svmnotn is `f64` the right choice?
  String(String),
  Boolean(bool),
  Null
}

/// An array of values.
pub type Array = Vec<Value>;

/// An object of key/value pais.
// TODO: @svmnotn is `Map` right here? Maybe a binary tree map?
pub type Object = Map<Property, Value>;
