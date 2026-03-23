use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)] 
pub struct Vent {
    pub vitesse: f64,        
    pub direction: i32,      
    pub horodatage: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct FiltreMeteo {
    pub debut: DateTime<Utc>,
    pub fin: Option<DateTime<Utc>>,
}