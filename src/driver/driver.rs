use error::{Error, ErrorCode};
use query::{Condition, SortRule, Range, Query};
use schema::Type;
use value::{Value, ValueIter};

/// The driver trait which all drivers will implement. Designed to be
/// interoperable with any data source, however the driver also assumes a
/// collection based data model.
///
/// ## Collection Based Data Model
/// Any value the driver can access is assumed to have a specific “type”
/// associated with it. This way the driver can map to structures like
/// collections in MongoDB, tables or views in a SQL database like PostgreSQL,
/// or labels in Neo4j.
///
/// In addition, operations on the driver are done in a CRUD fashion for a
/// specific type. Looking at the specific methods, like `read`, heavy
/// inspiration is taken from SQL databases and MongoDB which do all their CRUD
/// on collections. The unit for CRUD in an Ardite driver is a type.
///
/// Originally the driver was designed to work strictly like Falcor or GraphQL
/// by assuming a “graph” like structure for *all* data. This turns out to be a
/// nice abstraction, however it is inneficient for creating performant and
/// flexible systems. Abandoning the graph structure of drivers also helps the
/// driver implementors who might find it incredibly difficult and repetitive
/// to copy the same graph interface on their relational database.
///
/// ## Relationship Between the Schema and the Driver
/// The driver and the schema should never interact with each other. The schema
/// should be managed completely by Ardite and used to validate queries and
/// values before being sent to the driver. The driver should only be a
/// low-level consistent interface to a data source and should not make any
/// intelligent decisions about how an Ardite program should run.
///
/// Some unique driver-specific features are encouraged to be included in the
/// driver, such as PostgreSQL computed columns. However, these features should
/// *not* be derived from the schema, these features *should* be consistent,
/// and the driver must consistently perform all other expected functionality
/// *before* implementing new features.
///
/// This sharp seperation is most primarily for a seperation of concerns. It
/// allows developers working with the driver higher up in Ardite to make basic
/// assumptions about the driver. The seperation also prevents code reuse. The
/// `validate_query` or `validate_value` methods on `Schema` may be tempting to
/// use in the driver. There should never be any confusion over where in the
/// algorithm these methods should be called.
pub trait Driver {
  /// Connects to a driver and returns a driver instance. After calling this
  /// the driver is ready to roll!
  ///
  /// No schema definition is provided to the driver in its construction step.
  fn connect(uri: &str) -> Result<Self, Error> where Self: Sized;

  /// Lazily read some values from the driver.
  ///
  /// Designed against a couple of database specifications. Including the
  /// following:
  ///
  /// - [SQL `SELECT` statement][1].
  /// - [MongoDB `find` command][2].
  ///
  /// [1]: http://www.postgresql.org/docs/current/static/sql-select.html
  /// [2]: https://docs.mongodb.org/manual/reference/command/find/
  fn read(
    &self,
    type_: &Type,
    condition: Condition,
    sort: Vec<SortRule>,
    range: Range,
    query: Query
  ) -> Result<ValueIter, Error>;

  /// Read a single value from the driver. The default implementation uses the
  /// driver read method with a range of one.
  ///
  /// If a condition matches more than one value (while not recommended for
  /// this method) the first of these values, using the default sorting
  /// algorithm of the database, is returned.
  ///
  /// This method may be optionally optimized by the driver.
  fn read_one(
    &self,
    type_: &Type,
    condition: Condition,
    query: Query
  ) -> Result<Value, Error> {
    let mut values: Vec<_> = try!(self.read(
      type_,
      condition,
      Default::default(),
      Range::new(None, Some(1)),
      query
    )).collect();

    if values.len() > 1 {
      Err(Error::internal("Read with a limit of one returned more than one value."))
    } else if let Some(value) = values.pop() {
      Ok(value)
    } else {
      Err(Error::new(ErrorCode::NotFound, "No value was found for the condition.", None))
    }
  }
}
