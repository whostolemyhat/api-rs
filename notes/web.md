# using nickel and postgres to build a crud api

After having a play with [Iron to build a basic JSON response server](https://jamestease.co.uk/blether/writing-a-basic-json-web-server-in-rust-using-iron), I decided to make an interactive [CRUD (create, read, update, delete)](https://en.wikipedia.org/wiki/Create,_read,_update_and_delete) API. I'm going to use [Nickel](http://nickel.rs/) as the web framework, since I found Iron a bit clunky to use, and Nickel is very similar to Express, which I'm already familiar with.

## Setup

We'll use Postgres as the database, so make sure you have that available, then add the dependencies to `Cargo.toml`:

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

<!-- Make sure you are using version `0.8` of Nickel - as of October 2016, `nickel_postgres` doesn't yet work with the latest version (`0.9`) of Nickel, and you'll get lots of middleware trait errors. -->

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
