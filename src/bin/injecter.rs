// Utilisation :
//   cargo run --bin injecter                    <- lit vents.json par défaut
//   cargo run --bin injecter -- vents.json      <- fichier explicite
//   cargo run --bin injecter -- vents_juin.json

use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::{env, fs, process};

#[derive(Deserialize, Debug)]
struct Vent {
    vitesse: f64,
    direction: i32,
    horodatage: DateTime<Utc>,
}

#[tokio::main]
async fn main() {
    // 1. Fichier JSON à lire
    let chemin = env::args().nth(1).unwrap_or_else(|| "vents.json".to_string());

    let contenu = fs::read_to_string(&chemin).unwrap_or_else(|e| {
        eprintln!("Erreur lecture '{}' : {}", chemin, e);
        process::exit(1);
    });

    let vents: Vec<Vent> = serde_json::from_str(&contenu).unwrap_or_else(|e| {
        eprintln!("JSON invalide dans '{}' : {}", chemin, e);
        process::exit(1);
    });

    println!("Fichier '{}' — {} mesure(s) à insérer", chemin, vents.len());

    // 2. Connexion directe à la base (même fichier que le serveur)
    let pool = SqlitePool::connect("sqlite:meteo.db?mode=rwc")
        .await
        .unwrap_or_else(|e| {
            eprintln!("Impossible d'ouvrir meteo.db : {}", e);
            eprintln!("Conseil : lancez d'abord 'cargo run --bin serveur' une fois");
            process::exit(1);
        });

    // 3. Insertion une par une avec compte-rendu
    let mut succes = 0u32;
    let mut echecs = 0u32;

    for vent in &vents {
        let res = sqlx::query(
            "INSERT INTO vent (vitesse, direction, horodatage) VALUES (?, ?, ?)",
        )
        .bind(vent.vitesse)
        .bind(vent.direction)
        .bind(vent.horodatage)
        .execute(&pool)
        .await;

        match res {
            Ok(_) => {
                println!(
                    "  ✓  {:>6.1} km/h   {:>3}°   {}",
                    vent.vitesse,
                    vent.direction,
                    vent.horodatage.format("%d/%m/%Y %H:%M")
                );
                succes += 1;
            }
            Err(e) => {
                eprintln!("  ✗  Erreur insertion : {}", e);
                echecs += 1;
            }
        }
    }

    println!("\n✅ {} insérées   ❌ {} échouées", succes, echecs);
}