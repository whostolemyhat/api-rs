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
use r2d2_postgres::{ PostgresConnectionManager, SslMode };
use nickel_postgres::{ PostgresMiddleware, PostgresRequestExtensions };
use rustc_serialize::json;

#[derive(RustcEncodable, RustcDecodable)]
struct User {
  username: String,
  email: String,
  password: String
}

fn main() {
  // TODO: move this to a config file for live
  // username:password ... /database name
  let db_url = "postgresql://userapi:socks@localhost:5432/userapi";
  let db = PostgresConnectionManager::new(db_url, SslMode::None)
    .expect("Unable to connect to database");

  let db_pool = Pool::new(Config::default(), db)
    .expect("Unable to initialise database pool");

  let mut server = Nickel::new();
  server.utilize(PostgresMiddleware::with_pool(db_pool));
  let mut router = Nickel::router();

  router.get("/users", middleware! { |request, response|
    format!("Hello from GET /users")
  });

  router.post("/users/new", middleware! { |request, response|
    // let user = request.json_as::<User>().unwrap();
    // let username = user.username.to_string();
    // let email = user.email.to_string();
    // let password = user.password.to_string();
    let username = "hello".to_string();
    let email = "test@example.com".to_string();
    let password = "1234".to_string();

    let uuid = Uuid::new_v4();
    // let query = format!("INSERT INTO users (id, username, email, password) VALUES ({}, {}, {}, {})",
        // uuid, username, email, password);
    let query = "INSERT INTO users (id, username, email, password) VALUES ('123', 'tim', 'tim@test.com', '1234')";
    let db = request.pg_conn().expect("Failed to get connection from pool");

    for row in &db.query(&query[..], &[]).expect("Failed to save") {
      println!("{:?}", row);
    }

    format!("Hello from POST /users/new")
  });

  router.delete("/users/:id", middleware! { |request, response|
    format!("Hello from DELETE /users/:id")
  });

  server.utilize(router);
  server.listen("localhost:3004");
}
