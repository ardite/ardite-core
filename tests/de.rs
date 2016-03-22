#[macro_use(value)]
extern crate ardite;
extern crate regex;
extern crate url;

use std::path::PathBuf;

use regex::Regex;
use url::Url;

use ardite::{Definition, Type, DriverConfig, Schema};

fn create_basic_definition() -> Definition {
  // TODO: use order in file, not serde’s `BTreeMap` order.
  let mut definition = Definition::new();

  definition.add_type("person", {
    let mut person = Type::new();
    person.set_required(vec!["email"]);
    person.add_property("email", {
      let mut email = Schema::string();
      email.set_min_length(4);
      email.set_max_length(256);
      email.set_pattern(Regex::new(r".+@.+\..+").unwrap());
      email
    });
    person.add_property("name", {
      let mut name = Schema::string();
      name.set_min_length(2);
      name.set_max_length(64);
      name
    });
    person
  });

  definition.add_type("post", {
    let mut post = Type::new();
    post.set_required(vec!["headline"]);
    post.add_property("headline", {
      let mut headline = Schema::string();
      headline.set_min_length(4);
      headline.set_max_length(1024);
      headline
    });
    post.add_property("text", {
      let mut text = Schema::string();
      text.set_max_length(65536);
      text
    });
    post.add_property("topic", {
      Schema::enum_(vec!["showcase", "help", "ama"])
    });
    post
  });

  definition
}

fn create_kitchen_sink_definition() -> Definition {
  let mut definition = Definition::new();

  definition.set_driver(
    DriverConfig::new(Url::parse("scheme://host:1234?key1=value1&key2=value2#fragment").unwrap())
  );

  definition.add_type("a", Type::new());

  definition.add_type("b", {
    let mut b = Type::new();
    b.set_driver(DriverConfig::new(Url::parse("party://fun:4242").unwrap()));
    b
  });

  definition.add_type("c", {
    let mut c = Type::new();
    c.add_property("array", {
      let mut array = Schema::array();
      array.set_items({
        let mut sub_array = Schema::array();
        sub_array.set_items(Schema::null());
        sub_array
      });
      array
    });
    c.add_property("boolean", Schema::boolean());
    c.add_property("enum", Schema::enum_(vec![value!("red"), value!(2), value!(false), value!({ "hello" => { "world" => 8 } })]));
    c.add_property("integer", {
      let mut number = Schema::number();
      number.set_multiple_of(1.0);
      number.set_minimum(8.0);
      number.set_maximum(30.0);
      number
    });
    c.add_property("null", Schema::null());
    c.add_property("number", {
      let mut number = Schema::number();
      number.set_multiple_of(1.1);
      number.set_minimum(2.2);
      number.set_maximum(9.9);
      number.enable_exclusive_maximum();
      number
    });
    c.add_property("object", {
      let mut object = Schema::object();
      object.set_required(vec!["hello"]);
      object.enable_additional_properties();
      object.add_property("george", Schema::string());
      object.add_property("hello", {
        let mut hello = Schema::object();
        hello.add_property("world", Schema::null());
        hello
      });
      object
    });
    c
  });

  definition
}

#[test]
fn test_basic_json() {
  assert_eq!(
    Definition::from_file(PathBuf::from("tests/fixtures/definitions/basic.json")).unwrap(),
    create_basic_definition()
  );
}

#[test]
fn test_basic_yaml() {
  assert_eq!(
    Definition::from_file(PathBuf::from("tests/fixtures/definitions/basic.yml")).unwrap(),
    create_basic_definition()
  );
}

#[test]
fn test_kitchen_sink_yaml() {
  assert_eq!(
    Definition::from_file(PathBuf::from("tests/fixtures/definitions/kitchen-sink.yml")).unwrap(),
    create_kitchen_sink_definition()
  );
}
