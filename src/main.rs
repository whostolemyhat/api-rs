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

#[derive(RustcEncodable, RustcDecodable, Debug)]
struct User {
  id: Option<String>,
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

  router.get("/users", middleware! { |request, mut response|
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

  // only accepts JSON post data
  router.post("/users/new", middleware! { |request, response|
    let user = request.json_as::<User>().unwrap();
    println!("{:?}", user);

    let uuid = Uuid::new_v4().to_string();
    let db = request.pg_conn().expect("Failed to get connection from pool");
    let query = db.prepare_cached("INSERT INTO users (id, username, email, password) VALUES ($1, $2, $3, $4)").unwrap();

    query.execute(&[&uuid, &user.username, &user.email, &user.password]).expect("Failed to save");

    format!("Hello from POST /users/new")
  });

  router.delete("/users/:id", middleware! { |request, response|
    format!("Hello from DELETE /users/:id")
  });

  router.put("/users/:id", middleware! { |request, response|
    format!("Hello from PUT /users/:id")
  });

  server.utilize(router);
  server.listen("localhost:3004");
}
