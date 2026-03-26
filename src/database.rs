use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use chrono::Utc;

pub async fn initialiser_db() -> Pool<Sqlite> {
    let database_url = "sqlite:meteo.db?mode=rwc";
    
    let pool = SqlitePool::connect(database_url)
        .await
        .expect(" Impossible de se connecter à la base SQLite");

    
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
    .expect(" Erreur lors de la création de la table 'vent'");

    
    remplir_donnees_test(&pool).await;

    println!(" Base de données initialisée et prête !");
    pool
}

async fn remplir_donnees_test(pool: &Pool<Sqlite>) {
    
    let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM vent")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    if count == 0 {
        println!(" Base vide, insertion de données de test...");
        
        let donnees = [
            (12.5, 180, Utc::now()),
            (22.0, 270, Utc::now()),
            (5.5, 45, Utc::now()),
        ];

        for (vit, dir, date) in donnees {
            sqlx::query("INSERT INTO vent (vitesse, direction, horodatage) VALUES (?, ?, ?)")
                .bind(vit)
                .bind(dir)
                .bind(date)
                .execute(pool)
                .await
                .unwrap();
        }
    }
}