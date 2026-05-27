use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use reqwest::Proxy;
use serde_json::Value;
use crate::models::Entry;
use crate::parser::{get_hours_value, get_popular_times_value};

pub struct MapSearchParams {
    pub query: String,
    pub lat: f64,
    pub lon: f64,
    pub zoom: f64,
    pub radius: f64,
    pub lang: String,
}

pub fn search_fast(params: &MapSearchParams, proxy: Option<&str>) -> Result<Vec<Entry>> {
    let client = build_client(proxy)?;

    let mut url = reqwest::Url::parse("https://maps.google.com/search")?;
    url.query_pairs_mut()
        .append_pair("tbm", "map")
        .append_pair("authuser", "0")
        .append_pair("hl", &params.lang)
        .append_pair("q", &params.query)
        .append_pair("pb", &build_pb(params));

    let body = client.get(url).send()?.text()?;
    let body = remove_first_line(body.as_bytes());
    if body.is_empty() {
        return Err(anyhow!("empty response"));
    }

    let entries = parse_search_results(body)?;
    let filtered = filter_radius(entries, params.lat, params.lon, params.radius);
    Ok(filtered)
}

fn build_client(proxy: Option<&str>) -> Result<Client> {
    let mut builder = Client::builder();
    if let Some(p) = proxy {
        builder = builder.proxy(Proxy::all(p)?);
    }
    Ok(builder.build()?)
}

fn remove_first_line(data: &[u8]) -> &[u8] {
    if data.is_empty() {
        return data;
    }
    match data.iter().position(|&b| b == b'\n') {
        Some(idx) => &data[idx + 1..],
        None => &[],
    }
}

fn parse_search_results(raw: &[u8]) -> Result<Vec<Entry>> {
    let data: Value = serde_json::from_slice(raw)?;
    let container = data.get(0).and_then(|v| v.as_array()).ok_or_else(|| anyhow!("invalid structure"))?;
    let items = container.get(1).and_then(|v| v.as_array()).ok_or_else(|| anyhow!("empty list"))?;

    let mut entries = Vec::new();
    for item in items.iter().skip(1) {
        let arr = item.as_array().ok_or_else(|| anyhow!("invalid item"))?;
        let business = arr.get(14).and_then(|v| v.as_array()).ok_or_else(|| anyhow!("invalid business"))?;
        let b = Value::Array(business.clone());

        let mut entry = Entry::default();
        entry.input_id = get_str(&b, &[0]);
        entry.title = get_str(&b, &[11]);
        entry.categories = get_array(&b, &[13])
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        if let Some(first) = entry.categories.first() {
            entry.category = first.clone();
        }
        entry.website = get_str(&b, &[7, 0]);
        entry.review_rating = get_f64(&b, &[4, 7]);
        entry.review_count = get_i32(&b, &[4, 8]);
        entry.address = join_address(get_array(&b, &[2]));
        entry.latitude = get_f64(&b, &[9, 2]);
        entry.longitude = get_f64(&b, &[9, 3]);
        entry.phone = get_str(&b, &[178, 0, 0]).replace(' ', "");
        entry.open_hours = get_hours_value(&b);
        entry.status = get_str(&b, &[34, 4, 4]);
        entry.timezone = get_str(&b, &[30]);
        entry.data_id = get_str(&b, &[10]);
        // plus_code calculation omitted here (can be implemented with a crate later)
        entry.plus_code = String::new();
        entry.popular_times = get_popular_times_value(&b);

        entries.push(entry);
    }

    Ok(entries)
}

fn join_address(items: &[Value]) -> String {
    let mut parts = Vec::new();
    for v in items {
        if let Some(s) = v.as_str() {
            parts.push(s.to_string());
        } else {
            parts.push(format!("{}", v));
        }
    }
    parts.join(", ")
}

fn filter_radius(entries: Vec<Entry>, lat: f64, lon: f64, radius: f64) -> Vec<Entry> {
    if radius <= 0.0 {
        return entries;
    }

    entries
        .into_iter()
        .filter(|e| haversine(lat, lon, e.latitude, e.longitude) <= radius)
        .collect()
}

fn haversine(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371e3_f64;
    let to_rad = |d: f64| d * std::f64::consts::PI / 180.0;
    let dlat = to_rad(lat2 - lat1);
    let dlon = to_rad(lon2 - lon1);
    let a = (dlat / 2.0).sin().powi(2)
        + to_rad(lat1).cos() * to_rad(lat2).cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

fn get_value<'a>(v: &'a Value, path: &[usize]) -> Option<&'a Value> {
    let mut cur = v;
    for &idx in path {
        match cur {
            Value::Array(arr) => {
                cur = arr.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(cur)
}

fn get_str(v: &Value, path: &[usize]) -> String {
    get_value(v, path).and_then(|v| v.as_str()).unwrap_or_default().to_string()
}

fn get_f64(v: &Value, path: &[usize]) -> f64 {
    get_value(v, path).and_then(|v| v.as_f64()).unwrap_or_default()
}

fn get_i32(v: &Value, path: &[usize]) -> i32 {
    get_f64(v, path) as i32
}

fn get_array<'a>(v: &'a Value, path: &[usize]) -> &'a Vec<Value> {
    get_value(v, path).and_then(|v| v.as_array()).unwrap_or(&EMPTY_ARRAY)
}

static EMPTY_ARRAY: Vec<Value> = Vec::new();

fn build_pb(params: &MapSearchParams) -> String {
    let part1 = format!(
        "!4m12!1m3!1d3826.902183192154!2d{lon:.4}!3d{lat:.4}!2m3!1f0!2f0!3f0!3m2!1i600!2i800!4f{zoom:.1}!7i20!8i0",
        lon = params.lon,
        lat = params.lat,
        zoom = params.zoom
    );
    let part2 = "!10b1!12m22!1m3!18b1!30b1!34e1!2m3!5m1!6e2!20e3!4b0!10b1!12b1!13b1!16b1!17m1!3e1!20m3!5e2!6b1!14b1!46m1!1b0";
    let part3 = "!96b1!19m4!2m3!1i360!2i120!4i8";
    format!("{}{}{}", part1, part2, part3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_pb_contains_coords_and_zoom() {
        let params = MapSearchParams {
            query: "coffee".to_string(),
            lat: 37.7749,
            lon: -122.4194,
            zoom: 15.0,
            radius: 10000.0,
            lang: "en".to_string(),
        };
        let pb = build_pb(&params);
        assert!(pb.contains("2d-122.4194"));
        assert!(pb.contains("3d37.7749"));
        assert!(pb.contains("4f15.0"));
    }
}
