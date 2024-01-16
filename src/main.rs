mod schema;

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use bigdecimal::BigDecimal;

use diesel::{prelude::*,};
use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection, RunQueryDsl,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use dotenvy::dotenv;
use diesel::pg::data_types::PgInterval;
use serde::{Serialize, Deserialize};



#[derive(Selectable, Queryable, Debug)]
#[diesel(table_name = schema::runs)]
struct Run {
    id: i32,
    distance: BigDecimal,
    duration: PgInterval,
    created_at: chrono::NaiveDateTime,
}

#[derive(Debug)]
pub struct SerializablePgInterval(PgInterval);

impl Serialize for SerializablePgInterval {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Convert the PgInterval to a string or some other serializable format
        let s = format!("{:?}", self.0);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for SerializablePgInterval {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Parse the string into a chrono::Duration
        let duration = parse_duration(&s).map_err(serde::de::Error::custom)?;
        // Convert the chrono::Duration to a PgInterval
        let interval = PgInterval::from_microseconds(duration.num_microseconds().unwrap_or(0));
        Ok(SerializablePgInterval(interval))
    }
}

fn parse_duration(s: &str) -> Result<chrono::Duration, Box<dyn std::error::Error>> {
    let std_duration = humantime::parse_duration(s)?;
    let chrono_duration = chrono::Duration::from_std(std_duration)?;
    Ok(chrono_duration)
}

#[derive(Insertable)]
#[diesel(table_name = schema::runs)]
struct NewRun {
    distance: BigDecimal,
    duration: PgInterval,
}

#[derive(Serialize, Deserialize)]
struct SerializableRun {
    id: Option<i32>,
    distance: BigDecimal,
    duration: SerializablePgInterval,
    created_at: Option<chrono::NaiveDateTime>,
}

type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_diesel_async_postgres=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = std::env::var("DATABASE_URL").unwrap();
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(db_url);
    let pool = bb8::Pool::builder().build(config).await.unwrap();

    // build our application with a route
    let app = Router::new()
        .route("/", get(handler))
        .route("/runs", post(create_run))
        .with_state(pool);

    // run it
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn create_run(
    State(pool): State<Pool>,
    Json(new_run): Json<SerializableRun>,
) -> Result<Json<SerializableRun>, (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;

    // Convert SerializableNewRun to NewRun
    let new_run = NewRun {
        distance: new_run.distance,
        duration: new_run.duration.0, // Extract the PgInterval from the SerializablePgInterval
    };

    let res = diesel::insert_into(schema::runs::table)
        .values(new_run)
        .returning(Run::as_returning())
        .get_result::<Run>(&mut conn)
        .await
        .map_err(internal_error)?;

    println!("{:?}", res);

    // Convert Run to SerializableRun if needed
    let res = SerializableRun {
        id: Some(res.id),
        distance: res.distance,
        duration: SerializablePgInterval(res.duration),
        created_at: Some(res.created_at),
    };

    Ok(Json(res))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

async fn handler() -> String {
    "Hello, World!".to_string()
}
