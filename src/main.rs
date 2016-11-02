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
  // TODO: move this to a config file for live
  // username:password ... /database name
  let db_url = "postgresql://userapi:socks@localhost:5432/userapi";
  let db = PostgresConnectionManager::new(db_url, TlsMode::None)
    .expect("Unable to connect to database");

  let db_pool = Pool::new(Config::default(), db)
    .expect("Unable to initialise database pool");

  let mut server = Nickel::new();
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

    let uuid = Uuid::new_v4();
    let db = request.pg_conn().expect("Failed to get connection from pool");
    let query = db.prepare_cached("INSERT INTO users (id, username, email, password) VALUES ($1, $2, $3, $4)").unwrap();

    query.execute(&[&uuid, &user.username, &user.email, &user.password]).expect("Failed to save");

    format!("Created user {}", uuid)
  });

  router.delete("/users/:id", middleware! { |request, response|
    let id = request.param("id").unwrap();
    let db = request.pg_conn().expect("Failed to get connection from pool");
    println!("{:?}", id);

    let query = db.prepare_cached("DELETE FROM users WHERE id = $1").unwrap();
    query.execute(&[&id]).expect("Failed to delete user");

    format!("Deleted user {}", id)
  });

  router.put("/users/:id", middleware! { |request, response|
    let id = request.param("id").unwrap().to_string();

    // can't borrow request as immutable since id has borrowed already
    // so create new scope
    {
      let user = request.json_as::<User>().unwrap();

      let db = request.pg_conn().expect("Failed to get connection from pool");
      let query = db.prepare_cached("UPDATE users SET username = $1, email = $2, password = $3 WHERE id = $4").unwrap();
      query.execute(&[&user.username, &user.email, &user.password, &id]).expect("Failed to update user");
    }
    format!("Updated user {}", id)
  });

  server.utilize(PostgresMiddleware::with_pool(db_pool));
  server.utilize(router);
  server.listen("localhost:3004").unwrap();
}
