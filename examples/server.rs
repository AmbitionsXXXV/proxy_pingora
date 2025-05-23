use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use http::{Request, Response};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tower_http::trace::TraceLayer;
use tracing::{Span, info};

// User model
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u64,
    email: String,
    #[serde(skip_serializing)]
    password: String,
    name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// Request models
#[derive(Debug, Deserialize)]
struct CreateUser {
    email: String,
    password: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct UpdateUser {
    email: Option<String>,
    password: Option<String>,
    name: Option<String>,
}

// App state
#[derive(Clone)]
struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    next_id: AtomicU64,
    users: DashMap<u64, User>,
    argon2: Argon2<'static>,
}

impl AppState {
    fn new() -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                next_id: AtomicU64::new(1),
                users: DashMap::new(),
                argon2: Argon2::default(),
            }),
        }
    }

    fn get_user(&self, id: u64) -> Option<User> {
        self.inner.users.get(&id).map(|user| user.clone())
    }

    fn create_user(&self, create_user: CreateUser) -> Result<User, anyhow::Error> {
        let password_hash = hash_password(&self.inner.argon2, &create_user.password)?;

        let id = self.inner.next_id.fetch_add(1, Ordering::SeqCst);
        let now = Utc::now();

        let user = User {
            id,
            email: create_user.email,
            password: password_hash,
            name: create_user.name,
            created_at: now,
            updated_at: now,
        };

        self.inner.users.insert(id, user.clone());

        Ok(user)
    }

    fn update_user(&self, id: u64, update: UpdateUser) -> Option<User> {
        let mut user = self.get_user(id)?;

        if let Some(email) = update.email {
            user.email = email;
        }

        if let Some(password) = update.password {
            let password_hash = hash_password(&self.inner.argon2, &password).ok()?;
            user.password = password_hash;
        }

        if let Some(name) = update.name {
            user.name = name;
        }

        user.updated_at = Utc::now();
        self.inner.users.insert(id, user.clone());
        Some(user)
    }

    fn delete_user(&self, id: u64) -> Option<User> {
        self.inner.users.remove(&id).map(|(_, user)| user)
    }

    fn list_users(&self) -> Vec<User> {
        self.inner
            .users
            .iter()
            .map(|ref_multi| ref_multi.value().clone())
            .collect()
    }

    fn health(&self) -> bool {
        true
    }
}

fn hash_password(argon2: &Argon2<'static>, password: &str) -> Result<String, anyhow::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| anyhow::anyhow!("Failed to hash password"))?
        .to_string();
    Ok(password_hash)
}

// Route handlers
async fn get_user(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<User>, StatusCode> {
    state.get_user(id).map(Json).ok_or(StatusCode::NOT_FOUND)
}

async fn list_users(State(state): State<AppState>) -> Json<Vec<User>> {
    Json(state.list_users())
}

async fn create_user(
    State(state): State<AppState>,
    Json(create_user): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), (StatusCode, String)> {
    state
        .create_user(create_user)
        .map(|user| (StatusCode::CREATED, Json(user)))
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn update_user(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    Json(update_user): Json<UpdateUser>,
) -> Result<Json<User>, StatusCode> {
    state
        .update_user(id, update_user)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn delete_user(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<User>, StatusCode> {
    state.delete_user(id).map(Json).ok_or(StatusCode::NOT_FOUND)
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

async fn health_check(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: if state.health() {
            "healthy"
        } else {
            "unhealthy"
        },
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app_state = AppState::new();

    let app = Router::new()
        .route("/users/{id}", get(get_user))
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/{id}", put(update_user))
        .route("/users/{id}", delete(delete_user))
        .route("/health", get(health_check))
        .with_state(app_state)
        .layer(
            TraceLayer::new_for_http()
                .on_request(|request: &Request<_>, _span: &Span| {
                    info!("request: {:?}", request.headers());
                })
                .on_response(
                    |_response: &Response<_>, _latency: Duration, _span: &Span| {
                        info!("response: {:?}", _response.headers());
                    },
                ),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Server running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
