use crate::models::Map;
use std::io::{self, Write};

const PAGE_SIZE: usize = 25;

fn colors_enabled() -> bool {
    std::env::var("NO_COLOR").is_err()
        && std::env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
}

struct Colors {
    bold: &'static str,
    cyan: &'static str,
    yellow: &'static str,
    green: &'static str,
    dim: &'static str,
    reset: &'static str,
}

impl Colors {
    fn new(enabled: bool) -> Self {
        if enabled {
            Colors {
                bold: "\x1b[1m",
                cyan: "\x1b[36m",
                yellow: "\x1b[33m",
                green: "\x1b[32m",
                dim: "\x1b[2m",
                reset: "\x1b[0m",
            }
        } else {
            Colors {
                bold: "",
                cyan: "",
                yellow: "",
                green: "",
                dim: "",
                reset: "",
            }
        }
    }
}

fn print_map(rank: usize, map: &Map, c: &Colors) {
    let bpm_str = map
        .bpm
        .map(|b| format!("{:.0}", b))
        .unwrap_or("?".to_string());

    println!(
        "{bold}[#{rank}]{reset} {cyan}{title}{reset} — {artist}",
        bold = c.bold,
        reset = c.reset,
        cyan = c.cyan,
        rank = rank,
        title = map.title,
        artist = map.artist,
    );
    println!(
        "     {dim}Mapper:{reset} {mapper}  |  {yellow}Max RP: {rp}{reset}  |  ⭐ {stars:.1}  |  Plays: {plays}",
        dim = c.dim,
        reset = c.reset,
        yellow = c.yellow,
        mapper = map.creator,
        rp = map.max_rp(),
        stars = map.star_rating,
        plays = format_number(map.play_count),
    );
    println!(
        "     {dim}Tags:{reset} {tags}  |  Durée: {dur}  |  BPM: {bpm}",
        dim = c.dim,
        reset = c.reset,
        tags = map.tags_str(),
        dur = map.duration_str(),
        bpm = bpm_str,
    );
    println!(
        "     {green}🔗 {url}{reset}",
        green = c.green,
        reset = c.reset,
        url = map.url(),
    );
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(' ');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

pub fn progress_bar(current: u64, total: u64) {
    let width = 20usize;
    let filled = if total == 0 {
        width
    } else {
        ((current as f64 / total as f64) * width as f64) as usize
    };
    let empty = width.saturating_sub(filled);
    print!(
        "\rChargement des maps... [{filled}{empty}] {current}/{total}   ",
        filled = "█".repeat(filled),
        empty = "░".repeat(empty),
        current = current,
        total = total,
    );
    io::stdout().flush().ok();
}

pub fn paginate(maps: &[&Map], total_found: usize, low: u64, high: u64) {
    if maps.is_empty() {
        println!(
            "Aucune map trouvée avec un Max RP entre {} et {}.",
            low, high
        );
        return;
    }

    let c = Colors::new(colors_enabled());
    let total_pages = (maps.len() + PAGE_SIZE - 1) / PAGE_SIZE;
    let mut page = 0usize;

    loop {
        let start = page * PAGE_SIZE;
        let end = (start + PAGE_SIZE).min(maps.len());
        let page_maps = &maps[start..end];

        println!(
            "\n{bold}Page {}/{} — {} maps trouvées (RP {}-{}){reset}\n",
            page + 1,
            total_pages,
            total_found,
            low,
            high,
            bold = c.bold,
            reset = c.reset,
        );

        for (i, map) in page_maps.iter().enumerate() {
            print_map(start + i + 1, map, &c);
            println!();
        }

        if total_pages == 1 {
            break;
        }

        print!("[n]ext  [p]rev  [q]uit  > ");
        io::stdout().flush().ok();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        match input.trim() {
            "n" | "next" => {
                if page + 1 < total_pages {
                    page += 1;
                } else {
                    println!("Déjà sur la dernière page.");
                }
            }
            "p" | "prev" => {
                if page > 0 {
                    page -= 1;
                } else {
                    println!("Déjà sur la première page.");
                }
            }
            "q" | "quit" => break,
            _ => println!("Commande inconnue. Utilisez n, p, ou q."),
        }
    }
}
