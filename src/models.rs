use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Map {
    pub id: u64,
    pub title: String,
    pub creator: String,
    pub star_rating: f64,
    pub play_count: u64,
    pub tags: String,
    pub duration_ms: u64,
    pub created_at: Option<String>,
    pub status: String,
}

impl Map {
    /// Max RP at 100% accuracy: round(star_rating² × 5)
    pub fn max_rp(&self) -> u64 {
        let sr = self.star_rating;
        (sr * sr * 5.0).round() as u64
    }

    pub fn duration_str(&self) -> String {
        let total_secs = self.duration_ms / 1000;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{}:{:02}", mins, secs)
    }

    pub fn url(&self) -> String {
        format!("https://www.rhythia.com/maps/{}", self.id)
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiPage {
    pub total: u64,
    #[allow(dead_code)]
    #[serde(alias = "viewPerPage")]
    pub view_per_page: u64,
    #[allow(dead_code)]
    #[serde(alias = "currentPage")]
    pub current_page: u64,
    pub beatmaps: Option<Vec<ApiMap>>,
}

#[derive(Debug, Deserialize)]
pub struct ApiMap {
    pub id: u64,
    pub title: Option<String>,
    #[serde(alias = "ownerUsername")]
    pub owner_username: Option<String>,
    #[serde(alias = "starRating")]
    pub star_rating: Option<f64>,
    pub playcount: Option<u64>,
    pub tags: Option<String>,
    pub length: Option<u64>,
    pub created_at: Option<String>,
    pub status: Option<String>,
}

impl From<ApiMap> for Map {
    fn from(a: ApiMap) -> Self {
        Map {
            id: a.id,
            title: a.title.unwrap_or_default(),
            creator: a.owner_username.unwrap_or_default(),
            star_rating: a.star_rating.unwrap_or(0.0),
            play_count: a.playcount.unwrap_or(0),
            tags: a.tags.unwrap_or_default(),
            duration_ms: a.length.unwrap_or(0),
            created_at: a.created_at,
            status: a.status.unwrap_or_default(),
        }
    }
}
