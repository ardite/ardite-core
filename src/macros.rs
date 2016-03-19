#[macro_export]
macro_rules! point {
  () => {{ $crate::value::Pointer::new() }};

  ($($key:expr),*) => {{
    let mut pointer = $crate::value::Pointer::new();
    $(
      pointer.push(String::from($key));
    )*
    pointer
  }}
}

#[macro_export]
macro_rules! value {
  () => {{
    $crate::value::Value::Null
  }};

  (()) => {{
    $crate::value::Value::Null
  }};

  ([$($value:tt),*]) => {{
    let mut array = $crate::value::Array::new();
    $(
      array.push(value!($value));
    )*
    $crate::value::Value::Array(array)
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
