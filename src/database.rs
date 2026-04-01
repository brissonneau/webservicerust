//! Initialisation et connexion à la base de données SQLite.
//!
//! La base est créée automatiquement au premier lancement sous le nom `meteo.db`
//! dans le répertoire courant d'exécution.use sqlx::{Pool, Sqlite, SqlitePool};

use sqlx::{Pool, Sqlite, SqlitePool};

/// Ouvre (ou crée) la base de données SQLite et s'assure que le schéma est en place.
///
/// Cette fonction est appelée au démarrage du serveur. Elle effectue deux opérations :
///
/// 1. Connexion au fichier `meteo.db` via SQLx (créé s'il n'existe pas grâce à `mode=rwc`).
/// 2. Création de la table `vent` si elle n'existe pas encore (`CREATE TABLE IF NOT EXISTS`).
///
/// # Panics
///
/// Panique si la connexion à SQLite échoue ou si la création de la table renvoie une erreur.
/// Ces erreurs sont fatales : le serveur ne peut pas démarrer sans base de données.
///
/// # Retourne
///
/// Un [`Pool<Sqlite>`] prêt à l'emploi, partagé entre tous les handlers via l'état Axum.
pub async fn initialiser_db() -> Pool<Sqlite> {
    let database_url = "sqlite:meteo.db?mode=rwc";

    let pool = SqlitePool::connect(database_url)
        .await
        .expect("Impossible de se connecter à la base SQLite");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS vent (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            vitesse     REAL    NOT NULL,
            direction   INTEGER NOT NULL,
            horodatage  DATETIME NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("Erreur lors de la création de la table 'vent'");

    println!("Base de données initialisée et prête !");
    pool
}