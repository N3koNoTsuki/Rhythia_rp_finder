use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Map {
    pub id: u64,
    pub title: String,
    pub artist: String,
    pub creator: String,
    pub star_rating: f64,
    pub play_count: u64,
    pub tags: Vec<String>,
    pub duration: u64,
    pub bpm: Option<f64>,
    pub ranked: bool,
}

impl Map {
    /// Max RP at 100% accuracy: round(star_rating² × 2.5)
    /// Formula from cunev/rhythia-web-utils: calculatePerformancePoints(sr, 1.0)
    pub fn max_rp(&self) -> u64 {
        let sr = self.star_rating;
        ((sr * 50.0).powi(2) / 1000.0).round() as u64
    }

    pub fn duration_str(&self) -> String {
        let mins = self.duration / 60;
        let secs = self.duration % 60;
        format!("{}:{:02}", mins, secs)
    }

    pub fn tags_str(&self) -> String {
        if self.tags.is_empty() {
            "—".to_string()
        } else {
            self.tags.join(", ")
        }
    }

    pub fn url(&self) -> String {
        format!("https://www.rhythia.com/maps/{}", self.id)
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiPage {
    pub data: Vec<ApiMap>,
    pub meta: ApiMeta,
}

#[derive(Debug, Deserialize)]
pub struct ApiMeta {
    pub total: u64,
    #[allow(dead_code)]
    pub page: u64,
    #[allow(dead_code)]
    pub per_page: u64,
}

/// Raw API map shape — may differ from actual API; adapt field names here.
#[derive(Debug, Deserialize)]
pub struct ApiMap {
    pub id: u64,
    #[serde(alias = "name")]
    pub title: String,
    pub artist: String,
    #[serde(alias = "mapper")]
    pub creator: String,
    #[serde(alias = "starRating", alias = "star_rating")]
    pub star_rating: f64,
    #[serde(alias = "playCount", alias = "play_count", default)]
    pub play_count: u64,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(alias = "length", default)]
    pub duration: u64,
    pub bpm: Option<f64>,
    #[serde(default = "default_true")]
    pub ranked: bool,
}

fn default_true() -> bool {
    true
}

impl From<ApiMap> for Map {
    fn from(a: ApiMap) -> Self {
        Map {
            id: a.id,
            title: a.title,
            artist: a.artist,
            creator: a.creator,
            star_rating: a.star_rating,
            play_count: a.play_count,
            tags: a.tags,
            duration: a.duration,
            bpm: a.bpm,
            ranked: a.ranked,
        }
    }
}
