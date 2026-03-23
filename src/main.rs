mod models;
mod database;

use axum::{
    extract::{State, Query},
    http::StatusCode,
    response::IntoResponse,
    response::Html, 
    routing::{get, post},
    Json, Router,
};
use sqlx::SqlitePool;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    
    let pool = database::initialiser_db().await;

    
    let app = Router::new()
    .route("/", get(page_accueil)) 
    .route("/vent", post(ajouter_vent))
    .route("/vent", get(recuperer_vent))
    .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!(" Serveur météo Rust actif sur http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    async fn page_accueil() -> Html<&'static str> {
    Html(r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>Station Météo Rust</title>
                <style>
                    body { font-family: sans-serif; text-align: center; padding: 50px; background: #f4f4f9; }
                    .card { background: white; padding: 20px; border-radius: 10px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); display: inline-block; }
                    h1 { color: #2e7d32; }
                </style>
            </head>
            <body>
                <div class="card">
                    <h1>Station Météo Rust</h1>
                    <p>Le serveur est opérationnel</p>
                    <hr>
                    <p>Pour voir les données : <a href="/vent?debut=2026-01-01T00:00:00Z">Cliquez ici</a></p>
                </div>
            </body>
        </html>
    "#)
}
}




async fn ajouter_vent(
    State(pool): State<SqlitePool>, 
    Json(payload): Json<models::Vent>,
) -> impl IntoResponse {
    
    let resultat = sqlx::query(
        "INSERT INTO vent (vitesse, direction, horodatage) VALUES (?, ?, ?)"
    )
    .bind(payload.vitesse)
    .bind(payload.direction)
    .bind(payload.horodatage)
    .execute(&pool)
    .await;

    match resultat {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn recuperer_vent(
    State(pool): State<SqlitePool>,
    Query(filtre): Query<models::FiltreMeteo>,
) -> impl IntoResponse {
    
    let fin = filtre.fin.unwrap_or(filtre.debut + chrono::Duration::days(1));

   
    let resultat = sqlx::query_as::<_, models::Vent>(
        "SELECT vitesse, direction, horodatage FROM vent WHERE horodatage BETWEEN ? AND ?"
    )
    .bind(filtre.debut)
    .bind(fin)
    .fetch_all(&pool)
    .await;

    match resultat {
        Ok(mesures) => Json(mesures).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}