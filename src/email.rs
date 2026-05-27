use anyhow::Result;
use regex::Regex;
use reqwest::blocking::Client;
use std::collections::HashSet;
use std::time::Duration;

pub fn is_website_valid_for_email(url: &str) -> bool {
    if url.is_empty() {
        return false;
    }

    let needles = ["facebook", "instagram", "twitter", "linkedin"];
    !needles.iter().any(|n| url.contains(n))
}

pub fn extract_emails_from_url(url: &str) -> Result<Vec<String>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;

    let body = client.get(url).send()?.text()?;
    let re = Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")?;

    let mut set = HashSet::new();
    for cap in re.find_iter(&body) {
        set.insert(cap.as_str().to_string());
    }

    Ok(set.into_iter().collect())
}
