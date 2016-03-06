extern crate bson;
extern crate mongodb;

use std::error::Error as ErrorTrait;
use std::ops::Deref;
use linear_map::LinearMap;
use self::bson::{Bson, Document};
use self::mongodb::{Client, ThreadedClient, Error as MongoDBError};
use self::mongodb::connstring;
use self::mongodb::db::{Database, ThreadedDatabase};
use definition::Definition;
use definition::schema::Schema;
use driver::Driver;
use error::{Error, ErrorCode};
use patch::Patch;
use query::Query;
use value::Value;

struct MongoDBDriver {
  db: Database
}

impl Driver for MongoDBDriver {
  fn connect(uri: &str) -> Result<Box<Self>, Error> {
    let config = try!(connstring::parse(uri));
    if let Some(db_name) = config.clone().database {
      Ok(Box::new(MongoDBDriver {
        db: try!(Client::with_config(config, None, None)).db(&db_name)
      }))
    } else {
      Err(Error {
        code: ErrorCode::BadRequest,
        message: String::from(format!("Database name not provided in connection string '{}'.", uri)),
        hint: None
      })
    }
  }
  
  fn validate_definition(definition: &Definition) -> Result<(), Error> {
    match &definition.data {
      &Schema::Object{ref properties,..} => {
        for (key, value) in properties {
          match value {
            &Schema::Array{ref items} => {
              match items.deref() {
                &Schema::Object{..} => (),
                _ => return Err(Error::validation(
                  format!("Items for collection '{}' must be an object.", key),
                  format!("The MongoDB driver only supports inserting objects into collections, try changing the schema type of the '{}' collection to reflect this.", key)
                ))
              }
            },
            _ => return Err(Error::validation(
              format!("Schema type for the '{}' property must be an array.", key),
              format!("The MongoDB driver only supports array collections as the first level data types, try changing the '{}' property in your schema to reflect this.", key)
            ))
          }
        }
      },
      _ => return Err(Error::validation(
        "Root data schema is not object.",
        "The MongoDB driver only supports an object as the root data type, try changing your schema to reflect this."
      ))
    }
    Ok(())
  }
  
  fn query(&self, query: Query) -> Result<Value, Error> {
    match query {
      Query::Value => Err(Error {
        code: ErrorCode::Forbidden,
        message: String::from("Can’t query the entire MongoDB database."),
        hint: Some(String::from("Query something more specfic instead of the entire database."))
      }),
      Query::Object(collection_queries) => {
        // First level is the collection.
        let mut object = LinearMap::new();
        for (coll_name, query) in collection_queries {
          let collection = self.db.collection(&coll_name);
          match query {
            // TODO: Make this a range error when implementing selection by
            // range.
            Query::Value => {
              let mut cursor = try!(collection.find(None, None));
              let mut values = Vec::new();
              if let Some(Err(error)) = cursor.find(|entry| match entry {
                &Ok(ref document) => { values.push(Value::from(document.clone())); false },
                &Err(_) => true
              }) {
                return Err(Error::from(error));
              } else {
                object.insert(coll_name, Value::Array(values));
              }
            },
            // TODO: When implementing collections consider not using the
            // MongoDB `_id` property as the key.
            Query::Object(_) => {
              
            }
          }
        }
        Ok(Value::Object(object))
      }
    }
  }
  
  fn patch(&self, _: Vec<Patch>) -> Result<Value, Error> {
    Err(Error::unimplemented("Patching not implemented for MongoDB driver."))
  }
}

impl From<MongoDBError> for Error {
  fn from(error: MongoDBError) -> Self {
    Error::internal(error.description())
  }
}

impl From<Document> for Value {
  fn from(document: Document) -> Self {
    let mut map = LinearMap::new();
    for key in &document.keys {
      if let Some(value) = document.get(key) {
        map.insert(key.clone(), Value::from(value.clone()));
      }
    }
    Value::Object(map)
  }
}

impl From<Bson> for Value {
  fn from(bson: Bson) -> Self {
    match bson {
      Bson::FloatingPoint(value) => Value::F64(f64::from(value)),
      Bson::String(value) => Value::String(value),
      Bson::Array(values) => Value::Array(values.into_iter().map(|v| Value::from(v)).collect()),
      Bson::Document(document) => Value::from(document),
      Bson::Boolean(value) => Value::Boolean(value),
      Bson::Null => Value::Null,
      Bson::RegExp(value, _) => Value::String(value),
      Bson::JavaScriptCode(value) => Value::String(value),
      Bson::JavaScriptCodeWithScope(value, _) => Value::String(value),
      Bson::I32(value) => Value::I64(i64::from(value)),
      Bson::I64(value) => Value::I64(i64::from(value)),
      Bson::TimeStamp(value) => Value::I64(i64::from(value)),
      // TODO: Actual transformation of binary type.
      Bson::Binary(_, _) => Value::Null,
      Bson::ObjectId(object_id) => Value::String(object_id.to_string()),
      Bson::UtcDatetime(time) => Value::String(time.to_rfc3339())
    }
  }
}

// fn query_to_projection(query: Query) -> Bson {
//   match query {
//     Query::Value => Bson::I32(1),
//     Query::Object(query_properties) => {
//       let mut document = Document::new();
//       for (key, query) in query_properties {
//         document.insert(key, query_to_projection(query));
//       }
//       Bson::Document(document)
//     }
//   }
// }

#[cfg(test)]
mod tests {
  use super::bson::{Bson, Document};
  use super::mongodb::db::{ThreadedDatabase};
  use definition::Definition;
  use definition::schema::Schema;
  use driver::Driver;
  use driver::mongodb::MongoDBDriver;
  use value::Value;
  
  #[test]
  fn test_validate_definition() {
    assert!(MongoDBDriver::validate_definition(&Definition { data: Schema::Boolean }).is_err());
    assert!(MongoDBDriver::validate_definition(&Definition {
      data: Schema::Object {
        required: vec![],
        additional_properties: false,
        properties: linear_map! {}
      }
    }).is_ok());
    assert!(MongoDBDriver::validate_definition(&Definition {
      data: Schema::Object {
        required: vec![],
        additional_properties: false,
        properties: linear_map! {
          String::from("foo") => Schema::Boolean
        }
      }
    }).is_err());
    assert!(MongoDBDriver::validate_definition(&Definition {
      data: Schema::Object {
        required: vec![],
        additional_properties: false,
        properties: linear_map! {
          String::from("foo") => Schema::Array {
            items: Box::new(Schema::Boolean)
          }
        }
      }
    }).is_err());
    assert!(MongoDBDriver::validate_definition(&Definition {
      data: Schema::Object {
        required: vec![],
        additional_properties: false,
        properties: linear_map! {
          String::from("foo") => Schema::Array {
            items: Box::new(Schema::Object {
              required: vec![],
              additional_properties: true,
              properties: linear_map! {}
            })
          }
        }
      }
    }).is_ok());
  }
  
  #[test]
  fn test_database() {
    let driver = MongoDBDriver::connect("mongodb://localhost:27017/ardite_test").unwrap();
    let coll_name = "ardite_test_collection";
    driver.db.drop_collection(coll_name).unwrap();
    let collection = driver.db.collection(coll_name);
    let mut doc1 = Document::new();
    doc1.insert(String::from("title"), Bson::String(String::from("Back to the future!")));
    doc1.insert(String::from("foo"), Bson::String(String::from("bar")));
    let mut doc2 = Document::new();
    doc2.insert(String::from("buz"), Bson::String(String::from("baz")));
    let id1 = collection.insert_one(doc1, None).unwrap().inserted_id.unwrap();
    let id2 = collection.insert_one(doc2, None).unwrap().inserted_id.unwrap();
    assert!(driver.query(qvalue!()).is_err());
    assert_eq!(driver.query(qobject! { coll_name => qvalue!() }).unwrap(), vobject! {
      coll_name => varray![
        vobject! {
          "_id" => Value::from(id1),
          "title" => vstring!("Back to the future!"),
          "foo" => vstring!("bar")
        }, vobject! {
          "_id" => Value::from(id2),
          "buz" => vstring!("baz")
        }
      ]
    });
    driver.db.drop_collection(coll_name).unwrap();
  }
}