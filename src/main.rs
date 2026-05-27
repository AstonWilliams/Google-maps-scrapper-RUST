use anyhow::{anyhow, Result};
use clap::Parser;
use crossbeam_channel::{unbounded, Receiver};
use gmaps_scraper_rs::browser::BrowserClient;
use gmaps_scraper_rs::cli::{parse_input_line, Args};
use gmaps_scraper_rs::email::{extract_emails_from_url, is_website_valid_for_email};
use gmaps_scraper_rs::fast_search::{search_fast, MapSearchParams};
use gmaps_scraper_rs::installer::ensure_chromium_installed;
use gmaps_scraper_rs::models::Entry;
use gmaps_scraper_rs::output::OutputWriter;
use gmaps_scraper_rs::parser::entry_from_json;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};
use std::thread;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let args = Arc::new(args);

    let proxies = args
        .proxies
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s: &str| s.trim().to_string())
        .filter(|s: &String| !s.is_empty())
        .collect::<Vec<String>>();

    let (task_tx, task_rx) = unbounded::<(String, String)>();
    let (entry_tx, entry_rx) = unbounded::<Entry>();
    let max_results = Arc::new(AtomicUsize::new(0));

    let writer_args = Arc::clone(&args);
    let writer_handle = thread::spawn(move || -> Result<()> {
        let mut writer = OutputWriter::new(writer_args.results.as_deref(), writer_args.json)?;
        for entry in entry_rx.iter() {
            writer.write_entry(&entry)?;
        }
        Ok(())
    });

    if !args.fast_mode {
        ensure_chromium_installed(args.auto_install_chromium)?;
    }

    let worker_count = args.concurrency.max(1) as usize;
    let mut worker_handles = Vec::with_capacity(worker_count);

    for i in 0..worker_count {
        let rx = task_rx.clone();
        let tx = entry_tx.clone();
        let args = Arc::clone(&args);
        let proxy = if proxies.is_empty() {
            None
        } else {
            proxies.get(i % proxies.len()).cloned()
        };

        let counter = Arc::clone(&max_results);
        let handle = thread::spawn(move || worker_loop(i, rx, tx, args, proxy.as_deref(), counter));
        worker_handles.push(handle);
    }

    let file = File::open(&args.input)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if let Some((query, input_id)) = parse_input_line(&line) {
            task_tx.send((query, input_id))?;
        }
    }

    drop(task_tx);
    drop(entry_tx);

    for handle in worker_handles {
        handle.join().map_err(|_| anyhow!("worker panicked"))??;
    }

    writer_handle.join().map_err(|_| anyhow!("writer panicked"))??;

    Ok(())
}

fn worker_loop(
    id: usize,
    rx: Receiver<(String, String)>,
    tx: crossbeam_channel::Sender<Entry>,
    args: Arc<Args>,
    proxy: Option<&str>,
    max_results: Arc<AtomicUsize>,
) -> Result<()> {
    let browser = if args.fast_mode { None } else { Some(BrowserClient::new(true, proxy)?) };

    for (query, input_id) in rx.iter() {
        if args.fast_mode {
            let (lat, lon) = parse_geo(args.geo.as_deref())
                .ok_or_else(|| anyhow!("fast mode requires -geo and -zoom"))?;
            let params = MapSearchParams {
                query: query.clone(),
                lat,
                lon,
                zoom: args.zoom.unwrap_or(15) as f64,
                radius: args.radius,
                lang: args.lang.clone(),
            };

            let entries = search_fast(&params, proxy)?;
            for mut entry in entries {
                if args.max_results > 0 && max_results.load(Ordering::Relaxed) >= args.max_results {
                    break;
                }
                if !rating_in_range(entry.review_rating, args.min_rating, args.max_rating) {
                    continue;
                }
                entry.input_id = input_id.clone();
                if args.email && is_website_valid_for_email(&entry.website) {
                    if let Ok(emails) = extract_emails_from_url(&entry.website) {
                        entry.emails = emails;
                    }
                }
                tx.send(entry)?;
                if args.max_results > 0 {
                    max_results.fetch_add(1, Ordering::Relaxed);
                }
            }
        } else {
            let browser = browser.as_ref().ok_or_else(|| anyhow!("browser unavailable"))?;
            let url = if let (Some(geo), Some(zoom)) = (&args.geo, args.zoom) {
                format!(
                    "https://www.google.com/maps/search/{}/@{},{}z",
                    urlencoding::encode(&query),
                    geo,
                    zoom
                )
            } else {
                format!(
                    "https://www.google.com/maps/search/{}",
                    urlencoding::encode(&query)
                )
            };

            let place_links = browser.fetch_place_links(&url, args.depth)?;
            for place_url in place_links {
                if args.max_results > 0 && max_results.load(Ordering::Relaxed) >= args.max_results {
                    break;
                }
                let raw = browser.fetch_app_state(&place_url)?;
                let mut entry = entry_from_json(&raw)?;
                if !rating_in_range(entry.review_rating, args.min_rating, args.max_rating) {
                    continue;
                }
                entry.input_id = input_id.clone();
                if entry.link.is_empty() {
                    entry.link = place_url.clone();
                }
                if args.extra_reviews {
                    if let Ok(dom_reviews) = browser.fetch_dom_reviews(&place_url, 40) {
                        if !dom_reviews.is_empty() {
                            entry.user_reviews_extended = dom_reviews;
                        } else {
                            entry.user_reviews_extended = entry.user_reviews.clone();
                        }
                    } else {
                        entry.user_reviews_extended = entry.user_reviews.clone();
                    }
                }
                if args.email && is_website_valid_for_email(&entry.website) {
                    if let Ok(emails) = extract_emails_from_url(&entry.website) {
                        entry.emails = emails;
                    }
                }
                tx.send(entry)?;
                if args.max_results > 0 {
                    max_results.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        tracing::info!(worker = id, "completed query");
    }

    Ok(())
}

fn rating_in_range(rating: f64, min: f64, max: f64) -> bool {
    rating >= min && rating <= max
}

fn parse_geo(geo: Option<&str>) -> Option<(f64, f64)> {
    let geo = geo?;
    let mut parts = geo.split(',').map(|s| s.trim());
    let lat = parts.next()?.parse().ok()?;
    let lon = parts.next()?.parse().ok()?;
    Some((lat, lon))
}
