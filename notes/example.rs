#[macro_use]
extern crate nickel;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate nickel_postgres;

use nickel::{ Nickel, HttpRouter, MediaType, JsonBody };
use r2d2::{ Config, Pool };
use r2d2_postgres::{ PostgresConnectionManager, SslMode };
use nickel_postgres::{ PostgresMiddleware, PostgresRequestExtensions };

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