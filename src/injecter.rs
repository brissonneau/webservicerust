//! Outil en ligne de commande pour injecter des mesures de vent depuis un fichier JSON.
//!
//! Ce binaire lit un fichier JSON contenant un tableau de mesures [`Vent`] et les insère
//! directement dans la base de données SQLite `meteo.db`, sans passer par le serveur HTTP.
//!
//! # Utilisation
//! ```bash
//! cargo run --bin injecter                     # lit vents.json par défaut
//! cargo run --bin injecter -- vents_juin.json  # fichier explicite
//! ```
//!
//! # Format du fichier JSON attendu
//! ```json
//! [
//!   { "vitesse": 15.3, "direction": 90,  "horodatage": "2025-06-01T08:00:00Z" },
//!   { "vitesse": 22.7, "direction": 180, "horodatage": "2025-06-01T12:00:00Z" }
//! ]
//! ```
//!
//! # Prérequis
//! La base `meteo.db` doit exister. Si ce n'est pas le cas, lancez d'abord
//! `cargo run --bin serveur` une fois pour qu'elle soit créée.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::{env, fs, process};

/// Mesure de vent désérialisée depuis le fichier JSON d'entrée.
///
/// Doit correspondre exactement au schéma de la table `vent` en base.
#[derive(Deserialize, Debug)]
struct Vent {
    /// Vitesse en km/h.
    vitesse: f64,
    /// Direction en degrés (0–359).
    direction: i32,
    /// Date et heure de la mesure en UTC.
    horodatage: DateTime<Utc>,
}

/// Point d'entrée du binaire `injecter`.
///
/// Enchaîne : lecture du fichier → désérialisation JSON → connexion SQLite → insertions.
/// Affiche un compte-rendu ligne par ligne et un résumé final.
///
/// # Exits
/// - `0` : toutes les insertions ont réussi (ou partiellement, avec résumé).
/// - `1` : fichier introuvable, JSON invalide, ou base de données inaccessible.
#[tokio::main]
async fn main() {
    
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

    
    let pool = SqlitePool::connect("sqlite:meteo.db?mode=rwc")
        .await
        .unwrap_or_else(|e| {
            eprintln!("Impossible d'ouvrir meteo.db : {}", e);
            eprintln!("Conseil : lancez d'abord 'cargo run --bin serveur' une fois");
            process::exit(1);
        });

    
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

    println!("\n {} insérées  {} échouées", succes, echecs);
}