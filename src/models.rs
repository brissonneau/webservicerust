//! Modèles de données échangées par l'API météo.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Représente une mesure de vent enregistrée par la station.
///
/// Cette structure est utilisée à la fois pour la sérialisation JSON (API REST)
/// et pour la désérialisation depuis la base de données SQLite.
///
#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Vent {
    /// Vitesse du vent en kilomètres par heure. Doit être positive ou nulle.
    pub vitesse: f64,
    /// Direction du vent en degrés (0–359). 0° = Nord, 90° = Est, 180° = Sud, 270° = Ouest.
    pub direction: i32,
    /// Date et heure de la mesure, toujours stockée en UTC.
    pub horodatage: DateTime<Utc>,
}

/// Paramètres de filtrage pour la route `GET /vent`.
///
/// Passés en query string dans l'URL.
///
/// Si `fin` est absent, la route renvoie les mesures du jour entier à partir de `debut`
/// (soit `debut` + 24 heures).
#[derive(Deserialize)]
pub struct FiltreMeteo {
    /// Borne inférieure de la plage temporelle (incluse).
    pub debut: DateTime<Utc>,
    /// Borne supérieure de la plage temporelle (incluse). Optionnelle.
    pub fin: Option<DateTime<Utc>>,
}