use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "gmaps-scraper-rs", version, about = "Rust Google Maps scraper")]
pub struct Args {
    #[arg(short, long)]
    pub input: String,
    #[arg(short, long)]
    pub results: Option<String>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
    #[arg(long, default_value_t = 10)]
    pub depth: u32,
    #[arg(long, default_value_t = 8)]
    pub concurrency: u32,
    #[arg(long, default_value_t = false)]
    pub fast_mode: bool,
    #[arg(long, default_value_t = false)]
    pub extra_reviews: bool,
    #[arg(long, default_value_t = false)]
    pub email: bool,
    #[arg(long, default_value = "en")]
    pub lang: String,
    #[arg(long)]
    pub geo: Option<String>,
    #[arg(long)]
    pub zoom: Option<u32>,
    #[arg(long, default_value_t = 10000.0)]
    pub radius: f64,
    #[arg(long)]
    pub proxies: Option<String>,
    #[arg(long, default_value_t = false)]
    pub auto_install_chromium: bool,
    #[arg(long, default_value_t = 0.0)]
    pub min_rating: f64,
    #[arg(long, default_value_t = 5.0)]
    pub max_rating: f64,
    #[arg(long, default_value_t = 0)]
    pub max_results: usize,
}

pub fn parse_input_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some((query, id)) = trimmed.split_once("#!#") {
        let q = query.trim().to_string();
        let i = id.trim().to_string();
        if !q.is_empty() {
            return Some((q, i));
        }
    }

    Some((trimmed.to_string(), String::new()))
}
