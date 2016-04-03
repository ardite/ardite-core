/// Recursively create a `Value` from many primitive data types whilst also
/// detecting the appropriate type.
///
/// # Examples
/// ```rust
/// # #[macro_use(value)]
/// # extern crate ardite;
/// use ardite::value::Value;
///
/// # fn main() {
/// assert_eq!(value!(), Value::Null);
/// assert_eq!(value!(true), Value::Boolean(true));
/// assert_eq!(value!(42), Value::I64(42));
/// assert_eq!(value!(3.333), Value::F64(3.333));
/// assert_eq!(value!("Hello, world!"), Value::String("Hello, world!".to_owned()));
/// # }
/// ```
///
/// ```rust
/// # #[macro_use(value)]
/// # extern crate ardite;
/// use ardite::value::{Object, Array, Value};
///
/// # fn main() {
/// let value = value!([(), true, 42, 3.333, "Hello, world!", [1, 2, 3, true], { "hello" => "world" }]);
///
/// let mut array = Array::new();
/// array.push(Value::Null);
/// array.push(Value::Boolean(true));
/// array.push(Value::I64(42));
/// array.push(Value::F64(3.333));
/// array.push(Value::String("Hello, world!".to_owned()));
/// array.push(Value::Array({
///   let mut sub_array = Array::new();
///   sub_array.push(Value::I64(1));
///   sub_array.push(Value::I64(2));
///   sub_array.push(Value::I64(3));
///   sub_array.push(Value::Boolean(true));
///   sub_array
/// }));
/// array.push(Value::Object({
///   let mut object = Object::new();
///   object.insert("hello".to_owned(), Value::String("world".to_owned()));
///   object
/// }));
///
/// assert_eq!(value, Value::Array(array));
/// # }
/// ```
///
/// ```rust
/// # #[macro_use(value)]
/// # extern crate ardite;
/// use ardite::value::{Object, Array, Value};
///
/// # fn main() {
/// let value = value!({
///   "null" => (),
///   "boolean" => true,
///   "integer" => 42,
///   "float" => 3.333,
///   "string" => "Hello, world!",
///   "array" => [1, 2, 3, true],
///   "object" => {
///     "hello" => "world"
///   }
/// });
///
/// let mut object = Object::new();
/// object.insert("null".to_owned(), Value::Null);
/// object.insert("boolean".to_owned(), Value::Boolean(true));
/// object.insert("integer".to_owned(), Value::I64(42));
/// object.insert("float".to_owned(), Value::F64(3.333));
/// object.insert("string".to_owned(), Value::String("Hello, world!".to_owned()));
/// object.insert("array".to_owned(), Value::Array({
///   let mut array = Array::new();
///   array.push(Value::I64(1));
///   array.push(Value::I64(2));
///   array.push(Value::I64(3));
///   array.push(Value::Boolean(true));
///   array
/// }));
/// object.insert("object".to_owned(), Value::Object({
///   let mut sub_object = Object::new();
///   sub_object.insert("hello".to_owned(), Value::String("world".to_owned()));
///   sub_object
/// }));
///
/// assert_eq!(value, Value::Object(object));
/// # }
/// ```
#[macro_export]
macro_rules! value {
  () => {{
    $crate::value::Value::Null
  }};

  (()) => {{
    $crate::value::Value::Null
  }};

  ([]) => {{
    $crate::value::Value::Array($crate::value::Array::new())
  }};

  ([$($value:tt),*]) => {{
    let mut array = $crate::value::Array::new();
    $(
      array.push(value!($value));
    )*
    $crate::value::Value::Array(array)
  }};

  ({}) => {{
    $crate::value::Value::Object($crate::value::Object::new())
  }};

  ({ $($key:expr => $value:tt),* }) => {{
    let mut object = $crate::value::Object::new();
    $(
      object.insert($key.to_owned(), value!($value));
    )*
    $crate::value::Value::Object(object)
  }};

  ($value:expr) => {{
    $crate::value::Value::from($value)
  }}
}

#[cfg(test)]
macro_rules! str {
  ($value:expr) => {{
    String::from($value)
  }}
}
