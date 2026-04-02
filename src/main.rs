//! Point d'entrée du serveur REST météo.
//!
//! Ce binaire expose trois routes HTTP :
//!
//! | Méthode | Route   | Description                                      |
//! |---------|---------|--------------------------------------------------|
//! | `GET`   | `/`     | Interface web HTML avec formulaire et historique |
//! | `POST`  | `/vent` | Ajoute une mesure de vent (JSON)                 |
//! | `GET`   | `/vent` | Récupère les mesures filtrées par date (JSON)    |
//!
//! # Lancement
//! ```bash
//! cargo run --bin serveur
//! ```
//! Le serveur écoute sur `http://127.0.0.1:3000`.

mod database;
mod models;
mod injecter;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use chrono::Duration;
use sqlx::SqlitePool;
use std::net::SocketAddr;

/// Initialise la base de données et démarre le serveur Axum.
#[tokio::main]
async fn main() {
    let pool = database::initialiser_db().await;

    let app = Router::new()
        .route("/", get(page_accueil))
        .route("/vent", post(ajouter_vent))
        .route("/vent", get(recuperer_vent))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Serveur météo Rust actif sur http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Sert la page d'accueil HTML.
///
/// Affiche :
/// - Un formulaire permettant d'ajouter une mesure directement depuis le navigateur.
/// - Un tableau de tous les relevés existants, triés du plus récent au plus ancien.
/// - Des statistiques simples (nombre de mesures, vitesse min / moyenne / max).
///
/// Le formulaire soumet les données via un `fetch` JavaScript vers `POST /vent`,
/// puis insère la nouvelle ligne dans le tableau sans recharger la page.
///
/// # Paramètres
/// - `pool` : pool de connexions SQLite injecté par Axum via [`State`].
async fn page_accueil(State(pool): State<SqlitePool>) -> Html<String> {
    let mesures: Vec<models::Vent> = sqlx::query_as::<_, models::Vent>(
        "SELECT vitesse, direction, horodatage FROM vent ORDER BY horodatage DESC",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_else(|_| vec![]);

    let mut lignes = String::new();
    for m in &mesures {
        lignes.push_str(&format!(
            "<tr><td>{:.1} km/h</td><td>{}°</td><td>{}</td></tr>",
            m.vitesse,
            m.direction,
            m.horodatage.format("%d/%m/%Y %H:%M:%S")
        ));
    }

    let stats = if mesures.is_empty() {
        "<p>Aucune mesure enregistrée.</p>".to_string()
    } else {
        let moy = mesures.iter().map(|m| m.vitesse).sum::<f64>() / mesures.len() as f64;
        let max = mesures.iter().map(|m| m.vitesse).fold(f64::MIN, f64::max);
        let min = mesures.iter().map(|m| m.vitesse).fold(f64::MAX, f64::min);
        format!(
            "<p>📊 {} mesures &nbsp;|&nbsp; moy : <b>{:.1} km/h</b> \
             &nbsp;|&nbsp; min : <b>{:.1}</b> &nbsp;|&nbsp; max : <b>{:.1}</b></p>",
            mesures.len(), moy, min, max
        )
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <title>Station Météo</title>
    <style>
        body {{ font-family: sans-serif; background:#f0f4f8; margin:0; padding:30px; }}
        h1   {{ color:#1565c0; }}
        .carte {{
            background:white; border-radius:12px; padding:24px;
            box-shadow:0 4px 12px rgba(0,0,0,.1); max-width:680px; margin:auto;
        }}
        .stats {{ color:#555; margin-bottom:12px; }}
        table {{ width:100%; border-collapse:collapse; margin-top:16px; }}
        th    {{ background:#1565c0; color:white; padding:10px; }}
        td    {{ border:1px solid #ddd; padding:10px; text-align:center; }}
        tr:nth-child(even) {{ background:#f5f5f5; }}
    </style>
</head>
<body>
<div class="carte">
    <h1>🌬️ Relevés de la Station</h1>
    <div class="stats">{stats}</div>
    <table>
        <thead><tr><th>Vitesse</th><th>Direction</th><th>Date et Heure</th></tr></thead>
        <tbody>{lignes}</tbody>
    </table>
</div>
</body>
</html>"#,
        stats = stats,
        lignes = lignes
    );

    Html(html)
}

/// Insère une nouvelle mesure de vent dans la base de données.
///
/// # Route
/// `POST /vent`
///
/// # Corps de la requête
/// JSON correspondant à la structure [`models::Vent`] :
/// ```json
/// {
///   "vitesse": 15.3,
///   "direction": 90,
///   "horodatage": "2025-06-01T08:00:00Z"
/// }
/// ```
///
/// # Réponses
/// - `201 Created` : mesure enregistrée avec succès.
/// - `500 Internal Server Error` : échec de l'insertion SQLite (corps = message d'erreur).
async fn ajouter_vent(
    State(pool): State<SqlitePool>,
    Json(payload): Json<models::Vent>,
) -> impl IntoResponse {
    let res = sqlx::query(
        "INSERT INTO vent (vitesse, direction, horodatage) VALUES (?, ?, ?)",
    )
    .bind(payload.vitesse)
    .bind(payload.direction)
    .bind(payload.horodatage)
    .execute(&pool)
    .await;

    match res {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}//créer avec la commande curl 

/// Récupère les mesures de vent filtrées par plage de dates.
///
/// # Route
/// `GET /vent`
///
/// # Paramètres query
/// Voir [`models::FiltreMeteo`] :
/// - `debut` (obligatoire) : borne inférieure en RFC 3339 / ISO 8601.
/// - `fin` (optionnel) : borne supérieure. Si absent, `debut + 24h` est utilisé.
///
/// # Exemples
/// ```text
/// GET /vent?debut=2025-06-01T00:00:00Z
/// GET /vent?debut=2025-06-01T00:00:00Z&fin=2025-06-03T00:00:00Z
/// ```
///
/// # Réponses
/// - `200 OK` : tableau JSON de [`models::Vent`], trié par horodatage croissant.
/// - `500 Internal Server Error` : échec de la requête SQLite.
async fn recuperer_vent(
    State(pool): State<SqlitePool>,
    Query(filtre): Query<models::FiltreMeteo>,
) -> impl IntoResponse {
    let fin = filtre
        .fin
        .unwrap_or_else(|| filtre.debut + Duration::days(1));

    let res = sqlx::query_as::<_, models::Vent>(
        "SELECT vitesse, direction, horodatage FROM vent \
         WHERE horodatage BETWEEN ? AND ? \
         ORDER BY horodatage ASC",
    )
    .bind(filtre.debut)
    .bind(fin)
    .fetch_all(&pool)
    .await;

    match res {
        Ok(mesures) => Json(mesures).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}