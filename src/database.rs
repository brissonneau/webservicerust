use sqlx::{sqlite::SqlitePool, Pool, Sqlite};


pub async fn initialiser_db() -> Pool<Sqlite> {
    
    let database_url = "sqlite:meteo.db?mode=rwc";
    
    let pool = SqlitePool::connect(database_url)
        .await
        .expect("❌ Impossible de se connecter à la base SQLite");

   
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS vent (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            vitesse REAL NOT NULL,
            direction INTEGER NOT NULL,
            horodatage DATETIME NOT NULL
        )"
    )
    .execute(&pool)
    .await
    .expect("❌ Erreur lors de la création de la table 'vent'");

    println!("✅ Base de données initialisée (Fichier: meteo.db)");
    
    pool
}