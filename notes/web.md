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
r2d2_postgres = "0.11"
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
nickel_postgres = { git = "https://github.com/whostolemyhat/nickel-postgres", rev = "7cb8c0b"}
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
  id uuid primary key NOT NULL,
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
extern crate uuid;

use uuid::Uuid;

#[derive(RustcEncodable, RustcDecodable, Debug)]
struct User {
  id: Option<Uuid>,
  username: String,
  email: String,
  password: String
}
```

We're going to use a UUID for the user id (I'll cover installing the UUID crate in a sec), so require the crate in `main.rs`, and set up a struct representing the user, which match the fields in the database. Postgres allows `uuid` as a field type, so we can store it directly rather than converting it to a string. The id needs to be wrapped in an `Option` since we're going to use this struct for creating new users; when we create it, there won't be an id for this user, so `Option` allows this field to be null when we try to serialise to or from JSON. Add the `#[derive]` tag to the struct to enable `rust-serialize` to be able to serialise the data.

To use the [UUID crate](https://doc.rust-lang.org/uuid/uuid/index.html), we need it to be able to serialise to and from JSON; to do so, we need to specify features in the dependency:

``` # Cargo.toml
uuid = { version = "0.3", features = ["v4", "rustc-serialize"] }
```

[Cargo features](http://doc.crates.io/manifest.html#the-features-section) allow crates to be compiled with or without certain modules enabled to reduce the code used if you don't need a certain feature. In this case, we need to be able to serialise the UUID, and create one in the ['v4' format](https://en.wikipedia.org/wiki/Universally_unique_identifier#Version_4_.28random.29). We also need to specify that we're expecting Postgres to use the UUID type, so we need to update the `rust-postgres` crate dependency to enable that feature. We're not using `rust-postgres` directly - it's required by `r2d2_postgres` - but if we specify the features in our base Cargo.toml, it will be compiled with those features for everything in our project. Note: [as long as all your dependencies use the same version](https://github.com/sfackler/rust-postgres/issues/213)!

``` # Cargo.toml
[dependencies]
nickel = "0.9"
rustc-serialize = "0.3"
r2d2 = "0.7"
r2d2_postgres = "0.11"
nickel_postgres = { git = "https://github.com/whostolemyhat/nickel-postgres", rev = "7cb8c0b"}
uuid = { version = "0.3", features = ["v4", "rustc-serialize"] }

# Add postgres dependency with features enabled - used by r2d2_postgres
[dependencies.postgres]
version = "0.12"
features = ["with-uuid"]
```

If you start to get funny errors in situations like this, then check the Cargo.toml files in your dependencies to see if there any clashes, or run `cargo clean && cargo build` to see if there are any warnings about incompatible versions.

Dependencies dealt with, let's actually write some code:

``` // main.rs

fn main() {
  ...

  router.post("/users/new", middleware! { |request, response|
    let user = request.json_as::<User>().unwrap();
    println!("{:?}", user);

    let uuid = Uuid::new_v4();
    let db = request.pg_conn().expect("Failed to get connection from pool");
    let query = db.prepare_cached("INSERT INTO users (id, username, email, password) VALUES ($1, $2, $3, $4)").unwrap();

    query.execute(&[&uuid, &user.username, &user.email, &user.password]).expect("Failed to save");

    format!("Created user {}", uuid)
  });

  ...
}
```
This post handler converts the request body into a `User` struct:
```
let user = request.json_as::<User>().unwrap();
```

If the body of the request isn't valid JSON, it will crash here since we're using `unwrap`.

Once we've got a `User` object, we need to create an id using `Uuid`, then get a database connection from the pool and prepare our query. Finally, call the query on the pool and pass in the parameters.

Let's test it!

![](post.png)


Full code:
```
[package]
name = "api-rs"
version = "0.1.0"
authors = ["James Tease <james@jamestease.co.uk>"]

[dependencies]
nickel = "0.9"
rustc-serialize = "0.3"
r2d2 = "0.7"
r2d2_postgres = "0.11"
nickel_postgres = { git = "https://github.com/whostolemyhat/nickel-postgres", rev = "7cb8c0b"}
uuid = { version = "0.3", features = ["v4", "rustc-serialize"] }

[dependencies.postgres]
version = "0.12"
features = ["with-uuid"]
```

```
#[macro_use]
extern crate nickel;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate nickel_postgres;
extern crate rustc_serialize;
extern crate uuid;

use uuid::Uuid;
use nickel::{ Nickel, HttpRouter, MediaType, JsonBody };
use r2d2::{ Config, Pool };
use r2d2_postgres::{ PostgresConnectionManager, TlsMode };
use nickel_postgres::{ PostgresMiddleware, PostgresRequestExtensions };
use rustc_serialize::json;

#[derive(RustcEncodable, RustcDecodable, Debug)]
struct User {
  id: Option<Uuid>,
  username: String,
  email: String,
  password: String
}

fn main() {
  // add database connection
  let db_url = "postgresql://userapi:socks@localhost:5432/userapi";
  let db = PostgresConnectionManager::new(db_url, TlsMode::None)
    .expect("Unable to connect to database");
  let db_pool = Pool::new(Config::default(), db)
    .expect("Unable to initialise database pool");

  let mut server = Nickel::new();
  let mut router = Nickel::router();

  router.get("/", middleware! { |request, response|
    format!("Hello!")
  });

  router.post("/users/new", middleware! { |request, response|
    let user = request.json_as::<User>().unwrap();
    println!("{:?}", user);

    let uuid = Uuid::new_v4();
    let db = request.pg_conn().expect("Failed to get connection from pool");
    let query = db.prepare_cached("INSERT INTO users (id, username, email, password) VALUES ($1, $2, $3, $4)").unwrap();

    query.execute(&[&uuid, &user.username, &user.email, &user.password]).expect("Failed to save");

    format!("Created user {}", uuid)
  });

  // now use the database in the server
  server.utilize(PostgresMiddleware::with_pool(db_pool));
  server.utilize(router);
  server.listen("localhost:3004").unwrap();
}
```

## Get

Let's add a route to read the users, so we have a simple way of checking what's going on in the database. In the `router.get` function, we'll query the database similar to the `post` function, except just select rows. Once we have the data, we'll transform that into `User` structs then serialise into JSON for display.

```
router.get("/", middleware! { |request, mut response|
  let query = "SELECT * FROM users";
  let mut users = Vec::new();
  let db = request.pg_conn().expect("Failed to get connection from pool");

  for row in &db.query(query, &[]).expect("Failed to connect to db") {
    let user = User {
      id: row.get(0),
      username: row.get(1),
      email: row.get(2),
      password: row.get(3)
    };

    users.push(user);
  }

  response.set(MediaType::Json);
  json::encode(&users).expect("Failed to serialise users")
});
```

Note that we have to mark `response` as mutable in the middleware function, since we need to set the response type to JSON.

## notes
http://hermanradtke.com/2016/05/23/connecting-webservice-database-rust.html

DROP TABLE users;

CREATE TABLE users (
  id uuid primary key NOT NULL,
  username text NOT NULL,
  email text NOT NULL,
  password text NOT NULL,
  date_added timestamp default now()
);

INSERT INTO users (id, username, email, password) VALUES ('e942bd6f-520f-4529-bc7c-9663d3ec27b5', 'bob', 'bob@example.com', 'abc123')
