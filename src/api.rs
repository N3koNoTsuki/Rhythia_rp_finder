use anyhow::{Context, Result};
use reqwest::blocking::Client;
use std::time::Duration;

use crate::models::{ApiPage, Map};

const BASE_URL: &str = "https://www.rhythia.com";
const API_MAPS: &str = "/api/maps";
const PAGE_SIZE: u64 = 50;

pub struct RhythiaClient {
    client: Client,
}

impl RhythiaClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(30))
            .danger_accept_invalid_certs(false)
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { client })
    }

    fn fetch_page(&self, page: u64) -> Result<ApiPage> {
        let url = format!("{}{}", BASE_URL, API_MAPS);
        let mut attempt = 0u32;

        loop {
            let resp = self
                .client
                .get(&url)
                .header("Accept", "application/json")
                .header("Referer", format!("{}/maps", BASE_URL))
                .query(&[
                    ("page", page.to_string()),
                    ("limit", PAGE_SIZE.to_string()),
                    ("ranked", "true".to_string()),
                    ("sort", "plays".to_string()),
                ])
                .send();

            match resp {
                Err(e) => {
                    return Err(e).context(format!("Network error fetching page {}", page));
                }
                Ok(r) if r.status() == 429 => {
                    if attempt >= 4 {
                        anyhow::bail!("Rate-limited after {} retries on page {}", attempt, page);
                    }
                    let delay = Duration::from_millis(500 * (1u64 << attempt));
                    eprintln!("Rate limited, retrying in {}ms…", delay.as_millis());
                    std::thread::sleep(delay);
                    attempt += 1;
                }
                Ok(r) if !r.status().is_success() => {
                    anyhow::bail!("API returned HTTP {} for page {}", r.status(), page);
                }
                Ok(r) => {
                    let page_data: ApiPage = r
                        .json()
                        .context(format!("Failed to parse JSON for page {}", page))?;
                    return Ok(page_data);
                }
            }
        }
    }

    /// Fetch all ranked maps, calling `on_progress(fetched, total)` after each page.
    pub fn fetch_all<F>(&self, on_progress: F) -> Result<Vec<Map>>
    where
        F: Fn(u64, u64),
    {
        let first = self.fetch_page(1)?;
        let total = first.meta.total;
        let total_pages = (total + PAGE_SIZE - 1) / PAGE_SIZE;

        let mut maps: Vec<Map> = first.data.into_iter().map(Into::into).collect();
        on_progress(maps.len() as u64, total);

        for p in 2..=total_pages {
            let page = self.fetch_page(p)?;
            maps.extend(page.data.into_iter().map(Into::into));
            on_progress(maps.len() as u64, total);
            // Polite pause to avoid hammering the server
            if p < total_pages {
                std::thread::sleep(Duration::from_millis(100));
            }
        }

        Ok(maps)
    }
}
