#[macro_use(value)]
extern crate ardite;
extern crate regex;
extern crate url;

use std::path::PathBuf;

use url::Url;

use ardite::schema::{Schema, Type, Driver, from_file};

fn create_forum_schema() -> Schema {
  // TODO: use order in file, not serde’s `BTreeMap` order.
  let mut schema = Schema::new();

  schema.insert_type("person", Type::new());
  schema.insert_type("post", Type::new());

  schema
}

fn create_kitchen_sink_schema() -> Schema {
  let mut schema = Schema::new();

  schema.set_driver(
    Driver::new(Url::parse("scheme://host:1234?key1=value1&key2=value2#fragment").unwrap())
  );

  schema.insert_type("a", Type::new());
  schema.insert_type("b", Type::new());
  schema.insert_type("c", Type::new());

  schema
}

#[test]
fn test_de_forum_json() {
  assert_eq!(
    from_file(PathBuf::from("tests/fixtures/forum.json")).unwrap(),
    create_forum_schema()
  );
}

#[test]
fn test_de_forum_yaml() {
  assert_eq!(
    from_file(PathBuf::from("tests/fixtures/forum.yml")).unwrap(),
    create_forum_schema()
  );
}

#[test]
fn test_de_kitchen_sink_yaml() {
  assert_eq!(
    from_file(PathBuf::from("tests/fixtures/kitchen-sink.yml")).unwrap(),
    create_kitchen_sink_schema()
  );
}
