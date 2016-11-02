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

## Hello world

basic example

```
#[macro_use]
extern crate nickel;

use nickel::{ Nickel, HttpRouter };

fn main() {
  let mut server = Nickel::new();
  let mut router = Nickel::router();

  router.get("/", middleware! { |request, response|
    format!("Hello!")
  });

  server.utilize(router);
  server.listen("localhost:3004").unwrap();
}
```

![](hello-world.png)

## Database

The next stage is to connect to the database. Create a database and user which will be able to access tables within that database, then create a table
called `users`:

```
CREATE TABLE users (
  id text primary key NOT NULL,
  username text NOT NULL,
  email text NOT NULL,
  password text NOT NULL,
  date_added timestamp default now()
);
```

With the table created, we can start connecting in our app.

```// main.rs
#[macro_use]
extern crate nickel;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate nickel_postgres;

use nickel::{ Nickel, HttpRouter };
use r2d2::{ Config, Pool };
use r2d2_postgres::{ PostgresConnectionManager, SslMode };
use nickel_postgres::{ PostgresMiddleware };

fn main() {
  // add database connection
  let db_url = "postgresql://username:password@localhost:5432/databasename";
  let db = PostgresConnectionManager::new(db_url, SslMode::None)
    .expect("Unable to connect to database");
  let db_pool = Pool::new(Config::default(), db)
    .expect("Unable to initialise database pool");

  let mut server = Nickel::new();
  let mut router = Nickel::router();

  router.get("/", middleware! { |request, response|
    format!("Hello!")
  });

  // now use the database in the server
  server.utilize(PostgresMiddleware::with_pool(db_pool));
  server.utilize(router);
  server.listen("localhost:3004").unwrap();
}

```

The key things to note here:

1. Add the `r2d2` and `nickel_postgres` crates; `r2d2` is a database connection pool manager, so we don't need to create a new connection with every request, and `nickel_postgres` is middleware which allows us to access the database connection from the request object.

2. The database url is similar to a web address; replace the different sections with your database details. You need to have the username and password of the user allowed to make changes to your database (`username:password`, the url and port of the database (`locahost:5432` - `5432` is the default port Postgres listens on), and the name of the database `/databasename`.

3. We then create a connection to the database using `PostgresConnectionManager`, and set up a pool using the database connection.

4. The final step is to use the pool in the server: `server.utilize(PostgresMiddleware::with_pool(db_pool));`

![](database.png)

## Post route

The first route we'll add will be the Post route, which will allow us to create records in the database. This will accept JSON data, deserialise into a struct, then use that data to insert into the database.

First, create a struct to hold user data:

```// main.rs
#[derive(RustcEncodable, RustcDecodable, Debug)]
struct User {
  id: String,
  username: String,
  email: String,
  password: String
}
```



```
router.post("/users/new", middleware! { |request, response|
  let user = request.json_as::<User>().unwrap();
  println!("{:?}", user);

  let uuid = Uuid::new_v4().to_string();
  let db = request.pg_conn().expect("Failed to get connection from pool");
  let query = db.prepare_cached("INSERT INTO users (id, username, email, password) VALUES ($1, $2, $3, $4)").unwrap();

  query.execute(&[&uuid, &user.username, &user.email, &user.password]).expect("Failed to save");

  format!("Created user {}", uuid)
});
```


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
