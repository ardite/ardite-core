macro_rules! linear_map {
  (@single $($x:tt)*) => (());
  (@count $($rest:expr),*) => (<[()]>::len(&[$(linear_map!(@single $rest)),*]));
  ($($key:expr => $value:expr,)+) => { linear_map!($($key => $value),+) };
  ($($key:expr => $value:expr),*) => {
    {
      let _cap = linear_map!(@count $($key),*);
      let mut _map = ::linear_map::LinearMap::with_capacity(_cap);
      $(
        _map.insert($key, $value);
      )*
      _map
    }
  };
}

macro_rules! str {
  ($value:expr) => {
    String::from($value)
  }
}

macro_rules! point {
  ($($key:expr),*) => {
    {
      let mut _vec = Vec::new();
      $(
        _vec.push(str!($key));
      )*
      _vec
    }
  }
}

macro_rules! vnull {
  () => {
    $crate::value::Value::Null
  }
}

macro_rules! vbool {
  ($value:expr) => {
    $crate::value::Value::Boolean($value)
  }
}

macro_rules! vi64 {
  ($value:expr) => {
    $crate::value::Value::I64(i64::from($value))
  }
}

macro_rules! vf64 {
  ($value:expr) => {
    $crate::value::Value::F64(f64::from($value))
  }
}

macro_rules! vstring {
  ($value:expr) => {
    $crate::value::Value::String(str!($value))
  }
}

macro_rules! vobject {
  ($($key:expr => $value:expr),*) => {
    {
      let mut _map = ::linear_map::LinearMap::new();
      $(
        _map.insert(str!($key), $value);
      )*
      $crate::value::Value::Object(_map)
    }
  }
}

macro_rules! varray {
  ($($value:expr),*) => {
    {
      let mut _vec = Vec::new();
      $(
        _vec.push($value);
      )*
      $crate::value::Value::Array(_vec)
    }
  }
}
