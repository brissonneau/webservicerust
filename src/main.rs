mod models;
mod database;

use axum::{
    extract::{State, Query},
    http::StatusCode,
    response::{Html, IntoResponse},
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
}



async fn page_accueil(State(pool): State<SqlitePool>) -> Html<String> {
    
    let mesures: Vec<models::Vent> = sqlx::query_as::<_, models::Vent>(
        "SELECT vitesse, direction, horodatage FROM vent ORDER BY horodatage DESC"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_else(|_| vec![]); 

    
    let mut lignes_tableau = String::new();
    for m in mesures {
        lignes_tableau.push_str(&format!(
            "<tr><td>{:.2} km/h</td><td>{}°</td><td>{}</td></tr>",
            m.vitesse, 
            m.direction, 
            m.horodatage.format("%d/%m/%Y %H:%M:%S")
        ));
    }

    
    let html_content = format!(r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>Historique Météo</title>
                <style>
                    body {{ font-family: sans-serif; text-align: center; background: #f4f4f9; padding: 20px; }}
                    .container {{ background: white; padding: 20px; border-radius: 10px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); display: inline-block; min-width: 400px; }}
                    table {{ width: 100%; border-collapse: collapse; margin-top: 20px; }}
                    th, td {{ border: 1px solid #ddd; padding: 12px; text-align: center; }}
                    th {{ background-color: #2e7d32; color: white; }}
                    tr:nth-child(even) {{ background-color: #f2f2f2; }}
                    h1 {{ color: #2e7d32; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <h1>Relevés de la Station </h1>
                    <table>
                        <thead>
                            <tr>
                                <th>Vitesse</th>
                                <th>Direction</th>
                                <th>Date et Heure</th>
                            </tr>
                        </thead>
                        <tbody>
                            {}
                        </tbody>
                    </table>
                </div>
            </body>
        </html>
    "#, lignes_tableau);

    Html(html_content)
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