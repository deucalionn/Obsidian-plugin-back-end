use diesel::pg::PgConnection;
use diesel::r2d2::{ ConnectionManager, Pool};
use std::env;


pub fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    let database_url: String = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in the environment");
        
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    // Refer to the `r2d2` documentation for more methods to use
    // when building a connection pool
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}