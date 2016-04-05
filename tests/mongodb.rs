#![cfg(feature = "driver_mongodb")]

extern crate ardite;
#[macro_use]
extern crate ardite_driver_tests as tests;
extern crate bson;
extern crate linear_map;
extern crate mongodb;
extern crate url;

use ardite::Value;
use ardite::driver::Driver;
use ardite::driver::mongodb::MongoDB;
use bson::Document;
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use url::Url;

test_driver!(Tests);

pub struct Tests;

impl tests::Tests for Tests {
  fn test_driver(name: &str, values: Vec<Value>) -> Box<Driver> {
    let database = Client::connect("localhost", 27017).unwrap().db("ardite_driver_tests");
    database.drop_collection(&name).unwrap();
    let collection = database.collection(name);
    let documents: Vec<Document> = values.into_iter().map(Value::into).collect();
    collection.insert_many(documents, None).unwrap();

    let driver = MongoDB::connect(&Url::parse("mongodb://localhost:27017/ardite_driver_tests").unwrap()).unwrap();

    Box::new(driver)
  }
}
