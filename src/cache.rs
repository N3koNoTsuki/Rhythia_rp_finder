use crate::models::Map;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Plays,
    Date,
}

pub struct Cache {
    maps: Vec<Map>,
}

impl Cache {
    pub fn new(maps: Vec<Map>) -> Self {
        Cache { maps }
    }

    pub fn filter_by_rp(&self, low: u64, high: u64, sort: SortBy) -> Vec<&Map> {
        let mut filtered: Vec<&Map> = self
            .maps
            .iter()
            .filter(|m| {
                let rp = m.max_rp();
                rp >= low && rp <= high
            })
            .collect();

        match sort {
            SortBy::Plays => {
                filtered.sort_by(|a, b| b.play_count.cmp(&a.play_count));
            }
            SortBy::Date => {
                // Without a date field in the API we keep fetch order (chronological).
                // If the API provides a ranked_at or created_at, sort by it here.
            }
        }

        filtered
    }

    pub fn total(&self) -> usize {
        self.maps.len()
    }
}
