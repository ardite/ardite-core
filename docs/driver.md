# Driver
The driver manages data from *any* external source given that data may be modeled by an Ardite schema. A driver could be made for any traditional database like PostgreSQL, MySQL, MongoDB, or Neo4j. A driver could even be made for more modern innovations like Internet of Things devices. Basically any data source which one can *set* data to and *get* data back from can have an Ardite driver made for it to connect the data source to the Ardite ecosystem.

## Popular Drivers
- [MongoDB][1]: Provides an interface to the NoSQL MongoDB database. Currently this driver is a feature named `driver_mongodb` of [`ardite-core`][2], once a good dynamic loading system has been developed the MongoDB driver will be moved out of `ardite-core`. To build this driver with `ardite-core` run the following in the `ardite-core` source directory: `cargo build --features driver_mongodb`.

[1]: https://github.com/ardite/ardite-core/blob/f091b01cd96eeea0595a17442e493044a8d6bf9f/src/driver/mongodb.rs
[2]: https://github.com/ardite/ardite-core

## Collection Based Data Model
Any value the driver can access is assumed to have a specific “type” associated with it. This way the driver can map to structures like collections in MongoDB, tables or views in a SQL database like PostgreSQL, or labels in Neo4j.

In addition, operations on the driver are done in a CRUD fashion for a specific type. Looking at the specific methods, like `read`, heavy inspiration is taken from SQL databases and MongoDB which do all their CRUD on collections. The unit for CRUD in an Ardite driver is a type.

Originally the driver was designed to work strictly like Falcor or GraphQL by assuming a “graph” like structure for *all* data. This turns out to be a nice abstraction, however it is inneficient for creating performant and flexible systems. Abandoning the graph structure of drivers also helps the driver implementors who might find it incredibly difficult and repetitive to copy the same graph interface on their relational database.

## Relationship Between the Schema and the Driver
The driver and the schema should never interact with each other. The schema should be managed completely by Ardite and used to validate queries and values before being sent to the driver. The driver should only be a low-level consistent interface to a data source and should not make any intelligent decisions about how an Ardite program should run.

Some unique driver-specific features are encouraged to be included in the driver, such as PostgreSQL computed columns. However, these features should *not* be derived from the schema, these features *should* be consistent, and the driver must consistently perform all other expected functionality *before* implementing new features.

This sharp seperation is most primarily for a seperation of concerns. It allows developers working with the driver higher up in Ardite to make basic assumptions about the driver. The seperation also prevents code reuse. The `validate_query` or `validate_value` methods on `Schema` may be tempting to use in the driver. There should never be any confusion over where in the algorithm these methods should be called.
