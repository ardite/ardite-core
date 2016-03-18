![Ardite Logo](logo.png)

# Ardite Core

[![Build Status][1]][2]
[![Clippy Linting Result][3]][4]

[1]: https://travis-ci.org/ardite/ardite-core.svg?branch=master
[2]: https://travis-ci.org/ardite/ardite-core
[3]: https://clippy.bashy.io/github/ardite/ardite-core/develop/badge.svg
[4]: https://clippy.bashy.io/github/ardite/ardite-core/develop/log

> In the end, most web apps are just glorified CRUD interfaces.

Ever heard that before?

At the end of the day, the glory of our web apps comes from our front-end choices even if the basic CRUD operations are the same. So why do we have four or so standardized front-end frameworks, but many APIs are unique snowflakes developed in house? *Shouldn’t this be the other way around?* Shouldn’t our APIs be standardized?

The aim of Ardite is to fix this by providing an interface which allows you to connect the API design *you* want and need with the database *you’re* already using.

## Progress
This repository provides the core interfaces between the database “driver” and the API of your dreams, the “service.” The goal is to have the `0.1` core API’s `read` interface finished by the end of March and a GraphQL or REST API with a MongoDB driver.

Currently, the MongoDB driver is complete.

## Contributing
This package provides core interfaces to connect drivers with the user facing binaries. To get started with Ardite, we need code reviews! Start with `src/value.rs` and `src/query.rs` where most of the work is currently being done. If you see something noteworthy open an issue.
