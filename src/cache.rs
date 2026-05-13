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
                filtered.sort_by(|a, b| {
                    b.created_at.cmp(&a.created_at)
                });
            }
        }

        filtered
    }

    pub fn total(&self) -> usize {
        self.maps.len()
    }
}
