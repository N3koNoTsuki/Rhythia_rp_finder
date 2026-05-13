use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_json::json;

use crate::models::{ApiPage, Map};

const BASE_URL: &str = "https://production.rhythia.com";
const API_BEATMAPS: &str = "/api/getBeatmaps";
const PAGE_SIZE: u64 = 50;
const CONCURRENCY: usize = 8;

#[derive(Clone)]
pub struct RhythiaClient {
    client: Client,
}

impl RhythiaClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(CONCURRENCY + 2)
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { client })
    }

    fn fetch_page(&self, page: u64) -> Result<ApiPage> {
        let url = format!("{}{}", BASE_URL, API_BEATMAPS);
        let body = json!({ "session": "", "page": page });
        let mut attempt = 0u32;

        loop {
            match self
                .client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .json(&body)
                .send()
            {
                Err(e) => return Err(e).context(format!("Network error page {}", page)),
                Ok(r) if r.status() == 429 => {
                    if attempt >= 5 {
                        anyhow::bail!("Rate-limited after {} retries on page {}", attempt, page);
                    }
                    thread::sleep(Duration::from_millis(500 * (1u64 << attempt)));
                    attempt += 1;
                }
                Ok(r) if !r.status().is_success() => {
                    anyhow::bail!("HTTP {} on page {}", r.status(), page);
                }
                Ok(r) => {
                    return r
                        .json::<ApiPage>()
                        .context(format!("Failed to parse JSON for page {}", page));
                }
            }
        }
    }

    /// Fetch all maps using a parallel worker pool.
    pub fn fetch_all<F>(&self, on_progress: F) -> Result<Vec<Map>>
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        // Page 1 first — discovers total and total_pages
        let first = self.fetch_page(1)?;
        let total = first.total;
        let total_pages = (total + PAGE_SIZE - 1) / PAGE_SIZE;

        let first_maps: Vec<Map> = first
            .beatmaps
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect();

        let fetched = Arc::new(AtomicU64::new(first_maps.len() as u64));
        on_progress(fetched.load(Ordering::Relaxed), total);

        if total_pages <= 1 {
            return Ok(first_maps);
        }

        // Shared atomic page counter — workers grab the next page to fetch
        let next_page = Arc::new(AtomicU64::new(2));
        let on_progress = Arc::new(on_progress);
        let (tx, rx) = std::sync::mpsc::channel::<Result<(u64, Vec<Map>)>>();

        let workers: Vec<_> = (0..CONCURRENCY.min((total_pages - 1) as usize))
            .map(|_| {
                let client = self.clone();
                let next_page = next_page.clone();
                let fetched = fetched.clone();
                let on_progress = on_progress.clone();
                let tx = tx.clone();

                thread::spawn(move || loop {
                    let p = next_page.fetch_add(1, Ordering::Relaxed);
                    if p > total_pages {
                        break;
                    }
                    match client.fetch_page(p) {
                        Ok(data) => {
                            let maps: Vec<Map> = data
                                .beatmaps
                                .unwrap_or_default()
                                .into_iter()
                                .map(Into::into)
                                .collect();
                            let n = maps.len() as u64;
                            let prev = fetched.fetch_add(n, Ordering::Relaxed);
                            on_progress(prev + n, total);
                            let _ = tx.send(Ok((p, maps)));
                        }
                        Err(e) => {
                            let _ = tx.send(Err(e));
                        }
                    }
                })
            })
            .collect();

        drop(tx); // rx will drain once all workers are done

        // Collect results keyed by page number
        let mut page_results: HashMap<u64, Vec<Map>> = HashMap::new();
        let mut first_error: Option<anyhow::Error> = None;

        for result in rx {
            match result {
                Ok((p, maps)) => {
                    page_results.insert(p, maps);
                }
                Err(e) if first_error.is_none() => {
                    first_error = Some(e);
                }
                _ => {}
            }
        }

        for w in workers {
            let _ = w.join();
        }

        if let Some(e) = first_error {
            return Err(e);
        }

        // Assemble in page order (client-side sort happens later anyway,
        // but consistent ordering makes the cache deterministic)
        let mut all_maps = first_maps;
        for p in 2..=total_pages {
            if let Some(maps) = page_results.remove(&p) {
                all_maps.extend(maps);
            }
        }

        Ok(all_maps)
    }
}
