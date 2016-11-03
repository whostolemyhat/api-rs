#[macro_use]
extern crate nickel;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate nickel_postgres;
extern crate uuid;

use uuid::Uuid;
use nickel::{ Nickel, HttpRouter, JsonBody };
use r2d2::{ Config, Pool };
use r2d2_postgres::{ PostgresConnectionManager, TlsMode };
use nickel_postgres::{ PostgresMiddleware, PostgresRequestExtensions };

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

  router.post("/new", middleware! { |request, response|
    let user = request.json_as::<User>().unwrap();

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