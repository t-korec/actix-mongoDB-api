use actix_cors::Cors;
use actix_web::{http, middleware, App, HttpServer};
use dotenv::dotenv;
use mongodb::{bson::doc, error::Result, options::ClientOptions, sync::Client};
use std::env;
use user_service::UserService;
use std::error;

mod user_router;
mod user_service;

pub struct ServiceManager {
    user: UserService,
}

impl ServiceManager {
    pub fn new(user: UserService) -> Self {
        ServiceManager { user }
    }
}

pub struct AppState {
    service_manager: ServiceManager,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // init env
    dotenv().ok();
    for (key, value) in env::vars() {
        println!("{}: {}", key, value);
    }

    // init logger middleware
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();


     // Parse your connection string into an options struct
     let mut client_options =
     ClientOptions::parse("mongodb+srv://root:root@cluster0.cewrv.mongodb.net/local?retryWrites=true&w=majority")
         .await.expect("Some error message");

    // Manually set an option
    client_options.app_name = Some("Rust Demo".to_string());
    // Get a handle to the cluster
    let client = Client::with_options(client_options).expect("Some error message");
    // Ping the server to see if you can connect to the cluster
    client
        .database("local")
        .run_command(doc! {"ping": 1}, None)
        .expect("Some error message");
    println!("Connected successfully.");
    // List the names of the databases in that cluster
    for db_name in client.list_database_names(None, None).expect("Some error message") {
        println!("{}", db_name);
    }

    // Parse a connection string into an options struct.
   // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
  //  let client_options = ClientOptions::parse(&database_url).unwrap();

    // Get a handle to the deployment.
    //let client = Client::with_options(client_options).unwrap();
  //  let client = Client::with_uri_str("mongodb+srv://root:root@cluster0.cewrv.mongodb.net/local?retryWrites=true&w=majority").await?;
    // Get a handle to a database.
    let database_name = env::var("DATABASE_NAME").expect("DATABASE_NAME is not set in .env file");
    let db = client.database(&database_name);

    // Get a handle to a collection in the database.
    let user_collection_name = 
        env::var("USER_COLLECTION_NAME").expect("USER_COLLECTION_NAME is not set in .env file");
    let user_collection = db.collection(&user_collection_name);

    // server url
    let server_url = env::var("SERVER_URL").expect("SERVER_URL is not set in .env file");

    // start server
    HttpServer::new(move || {
        let user_service_worker = UserService::new(user_collection.clone());
        let service_manager = ServiceManager::new(user_service_worker);

        // cors
        let cors_middleware = Cors::default()
            .allowed_methods(vec!["GET", "POST", "DELETE", "PUT"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        // launch http server
        App::new()
            .wrap(cors_middleware)
            .wrap(middleware::Logger::default())
            // https://github.com/actix/examples/blob/8dab533b40d9d0640e5c75922c9e8e292ed4a7d5/sqlx_todo/src/main.rs#L41
            // pass database pool to application so we can access it inside handlers
            .data(AppState { service_manager })
            .configure(user_router::init)
    })
    .bind(server_url).expect("some eroor")
    .run()
    .await
}
