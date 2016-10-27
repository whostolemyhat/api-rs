# using nickel and postgres to build a crud api

After having a play with [Iron to build a basic JSON response server](https://jamestease.co.uk/blether/writing-a-basic-json-web-server-in-rust-using-iron), I decided to make an interactive [CRUD (create, read, update, delete)](https://en.wikipedia.org/wiki/Create,_read,_update_and_delete) API. I'm going to use [Nickel](http://nickel.rs/) as the web framework, since I found Iron a bit clunky to use, and Nickel is very similar to Express, which I'm already familiar with.

## Setup

Create a new binary project with `cargo new api --bin`. We'll use Postgres as the database, so make sure you have that available (we'll cover creating the table in a bit), then add the dependencies to `Cargo.toml`:

```# Cargo.toml
[package]
name = "api-rs"
version = "0.1.0"
authors = ["James Tease <james@jamestease.co.uk>"]

[dependencies]
nickel = "0.9"
nickel_postgres = "0.2"
rustc-serialize = "0.3"
r2d2 = "0.7"
r2d2_postgres = "0.10"
```

`nickel_postgres` is a middleware library which helps connect Nickel and Postgres; `r2d2` is a database connection pool manager, since we don't want to open a new connection with every request (with a lot of traffic, opening a new connection with each request means you'd quickly get to the point where new requests wouldn't be able to connect).

Let's run!

```
cargo run

error[E0277]: the trait bound `for<'r, 'mw, 'conn> nickel_postgres::PostgresMiddleware: std::ops::Fn<(&'r mut nickel
::Request<'mw, 'conn>, nickel::Response<'mw>)>` is not satisfied
  --> src\main.rs:35:10
   |
35 |   server.utilize(PostgresMiddleware::with_pool(db_pool));
   |          ^^^^^^^
   |
   = note: required because of the requirements on the impl of `nickel::Middleware<()>` for `nickel_postgres::Postgr
esMiddleware`
... [more errors]
```

Well, that didn't work.

After a bit of digging, it turns out this error is because `nickel_postgres` requires version `0.8.1`, which is incompatible with the version we're using in our main app, `0.9`. I've submitted a pull request to fix this, but until it gets merged we can use the version from my Github account in Cargo.toml:

```# Cargo.toml
nickel_postgres = { git = "https://github.com/whostolemyhat/nickel-postgres", rev = "8fa89b4"}
```

And run:

```
cargo run
...
Hello World!
```

Hooray!

## notes
http://hermanradtke.com/2016/05/23/connecting-webservice-database-rust.html

DROP TABLE users;

CREATE TABLE users (
  id text primary key NOT NULL,
  username text NOT NULL,
  email text NOT NULL,
  password text NOT NULL,
  date_added timestamp default now()
);

INSERT INTO users (id, username, email, password) VALUES ('e942bd6f-520f-4529-bc7c-9663d3ec27b5', 'bob', 'bob@example.com', 'abc123')
