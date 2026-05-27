use crate::models::*;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;

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
    get_value(v, path)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

fn get_f64(v: &Value, path: &[usize]) -> f64 {
    get_value(v, path)
        .and_then(|v| v.as_f64())
        .unwrap_or_default()
}

fn get_i32(v: &Value, path: &[usize]) -> i32 {
    get_f64(v, path) as i32
}

fn get_array<'a>(v: &'a Value, path: &[usize]) -> &'a Vec<Value> {
    get_value(v, path)
        .and_then(|v| v.as_array())
        .unwrap_or(&EMPTY_ARRAY)
}

static EMPTY_ARRAY: Vec<Value> = Vec::new();

pub fn entry_from_json(raw: &str) -> Result<Entry> {
    let jd: Value = serde_json::from_str(raw)?;

    let darray = get_value(&jd, &[6])
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("invalid json structure"))?;

    let d = Value::Array(darray.clone());

    let mut entry = Entry::default();

    entry.review_count = get_i32(&d, &[4, 8]);
    entry.link = get_str(&d, &[27]);
    entry.title = get_str(&d, &[11]);

    let cats = get_array(&d, &[13]);
    entry.categories = cats.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
    if let Some(first) = entry.categories.first() {
        entry.category = first.clone();
    }

    let addr = get_str(&d, &[18]);
    entry.address = addr.trim_start_matches(&format!("{},", entry.title)).trim().to_string();

    entry.open_hours = get_hours_value(&d);
    entry.popular_times = get_popular_times_value(&d);
    entry.website = extract_actual_url(&get_str(&d, &[7, 0]));
    entry.phone = get_str(&d, &[178, 0, 0]);
    entry.plus_code = get_str(&d, &[183, 2, 2, 0]);
    entry.review_rating = get_f64(&d, &[4, 7]);
    entry.latitude = get_f64(&d, &[9, 2]);
    entry.longitude = get_f64(&d, &[9, 3]);
    entry.cid = get_str(&jd, &[25, 3, 0, 13, 0, 0, 1]);
    entry.status = get_str(&d, &[34, 4, 4]);
    entry.description = get_str(&d, &[32, 1, 1]);
    entry.reviews_link = get_str(&d, &[4, 3, 0]);
    entry.thumbnail = get_str(&d, &[72, 0, 1, 6, 0]);
    entry.timezone = get_str(&d, &[30]);
    entry.price_range = get_str(&d, &[4, 2]);
    entry.data_id = get_str(&d, &[10]);
    entry.place_id = get_str(&d, &[78]);

    entry.images = get_link_source(&d, &[171, 0], &[3, 0, 6, 0], &[2])
        .into_iter()
        .map(|ls| Image { title: ls.source, image: ls.link })
        .collect();

    entry.reservations = get_link_source(&d, &[46], &[0], &[1]);

    let mut order_online = get_link_source(&d, &[75, 0, 1, 2], &[1, 2, 0], &[0, 0]);
    if order_online.is_empty() {
        order_online = get_link_source(&d, &[75, 0, 0, 2], &[1, 2, 0], &[0, 0]);
    }
    entry.order_online = order_online;

    entry.menu = LinkSource {
        link: get_str(&d, &[38, 0]),
        source: get_str(&d, &[38, 1]),
    };

    entry.owner = Owner {
        id: get_str(&d, &[57, 2]),
        name: get_str(&d, &[57, 1]),
        link: String::new(),
    };
    if !entry.owner.id.is_empty() {
        entry.owner.link = format!("https://www.google.com/maps/contrib/{}", entry.owner.id);
    }

    entry.complete_address = Address {
        borough: get_str(&d, &[183, 1, 0]),
        street: get_str(&d, &[183, 1, 1]),
        city: get_str(&d, &[183, 1, 3]),
        postal_code: get_str(&d, &[183, 1, 4]),
        state: get_str(&d, &[183, 1, 5]),
        country: get_str(&d, &[183, 1, 6]),
    };

    entry.about = get_about(&d);

    entry.reviews_per_rating = HashMap::from([
        (1, get_i32(&d, &[175, 3, 0])),
        (2, get_i32(&d, &[175, 3, 1])),
        (3, get_i32(&d, &[175, 3, 2])),
        (4, get_i32(&d, &[175, 3, 3])),
        (5, get_i32(&d, &[175, 3, 4])),
    ]);

    let mut reviews = get_array(&d, &[175, 9, 0, 0]).clone();
    if reviews.is_empty() {
        reviews = get_array(&d, &[175, 9, 0]).clone();
    }
    entry.user_reviews = parse_reviews(&reviews);

    Ok(entry)
}

fn get_about(d: &Value) -> Vec<About> {
    let about_i = get_array(d, &[100, 1]);
    let mut out = Vec::new();

    for item in about_i.iter() {
        let id = get_str(item, &[0]);
        let name = get_str(item, &[1]);
        let mut about = About { id, name, options: Vec::new() };

        let opts = get_array(item, &[2]);
        for opt in opts {
            let enabled = get_i32(opt, &[2, 1, 0, 0]) == 1;
            let name = get_str(opt, &[1]);
            if !name.is_empty() {
                about.options.push(OptionItem { name, enabled });
            }
        }

        out.push(about);
    }

    out
}

fn parse_reviews(reviews_i: &[Value]) -> Vec<Review> {
    let mut out = Vec::new();

    for el in reviews_i {
        let mut entry = get_value(el, &[0]).cloned();
        if entry.is_none() {
            entry = Some(el.clone());
        }
        let entry = match entry {
            Some(Value::Array(_)) => entry.unwrap(),
            _ => continue,
        };

        let time = get_array(&entry, &[2, 2, 0, 1, 21, 6, 8]);
        let time = if time.is_empty() {
            get_array(&entry, &[2, 2, 0, 1, 6, 8])
        } else {
            time
        };

        let when = if time.len() >= 3 {
            format!("{}-{}-{}", time[0], time[1], time[2])
        } else {
            String::new()
        };

        let mut profile_picture = decode_url(&get_str(&entry, &[1, 4, 5, 1])).unwrap_or_default();
        if profile_picture.is_empty() {
            profile_picture = get_str(&entry, &[1, 2, 0]);
        }
        if profile_picture.is_empty() {
            profile_picture = get_str(&entry, &[0, 2, 0]);
        }

        let mut author_name = get_str(&entry, &[1, 4, 5, 0]);
        if author_name.is_empty() {
            author_name = get_str(&entry, &[1, 4, 4]);
        }
        if author_name.is_empty() {
            author_name = get_str(&entry, &[0, 1]);
        }

        let mut rating = get_i32(&entry, &[2, 0, 0]);
        if rating == 0 {
            rating = get_i32(&entry, &[2, 0]);
        }
        if rating == 0 {
            rating = get_i32(&entry, &[1, 0, 0]);
        }

        let mut description = get_str(&entry, &[2, 15, 0, 0]);
        if description.is_empty() {
            description = get_str(&entry, &[2, 15, 0]);
        }
        if description.is_empty() {
            description = get_str(&entry, &[3, 0]);
        }

        if author_name.is_empty() {
            continue;
        }

        let mut review = Review {
            name: author_name,
            profile_picture,
            rating,
            description,
            images: Vec::new(),
            when,
        };

        let imgs = get_array(&entry, &[2, 2]);
        for img in imgs {
            let url = get_str(img, &[1, 6, 0]);
            if !url.is_empty() {
                review.images.push(url);
            }
        }

        out.push(review);
    }

    out
}

fn get_link_source(d: &Value, arr_path: &[usize], link_path: &[usize], source_path: &[usize]) -> Vec<LinkSource> {
    let arr = get_array(d, arr_path);
    let mut out = Vec::new();

    for item in arr.iter() {
        let source = get_str(item, source_path);
        let link = get_str(item, link_path);
        if !source.is_empty() && !link.is_empty() {
            out.push(LinkSource { link, source });
        }
    }

    out
}

pub fn get_hours_value(d: &Value) -> HashMap<String, Vec<String>> {
    let mut items = get_array(d, &[203, 0]).clone();
    if items.is_empty() {
        items = get_array(d, &[34, 1]).clone();
    }

    let mut hours = HashMap::new();

    for item in items {
        let day = get_str(&item, &[0]);
        if day.is_empty() {
            continue;
        }

        let time_slots = get_array(&item, &[3]);
        if !time_slots.is_empty() {
            let mut times = Vec::new();
            for slot in time_slots {
                let t = get_str(slot, &[0]);
                if !t.is_empty() {
                    times.push(t);
                }
            }
            if !times.is_empty() {
                hours.insert(day, times);
            }
        } else {
            let times_i = get_array(&item, &[1]);
            let mut times = Vec::new();
            for t in times_i {
                if let Some(s) = t.as_str() {
                    times.push(s.to_string());
                }
            }
            if !times.is_empty() {
                hours.insert(day, times);
            }
        }
    }

    hours
}

pub fn get_popular_times_value(d: &Value) -> HashMap<String, HashMap<i32, i32>> {
    let items = get_array(d, &[84, 0]);
    let day_of_week = HashMap::from([
        (1, "Monday".to_string()),
        (2, "Tuesday".to_string()),
        (3, "Wednesday".to_string()),
        (4, "Thursday".to_string()),
        (5, "Friday".to_string()),
        (6, "Saturday".to_string()),
        (7, "Sunday".to_string()),
    ]);

    let mut out = HashMap::new();

    for item in items {
        let day = get_i32(item, &[0]);
        let times_i = get_array(item, &[1]);
        let mut times = HashMap::new();

        for t in times_i {
            let h = get_i32(t, &[0]);
            let v = get_i32(t, &[1]);
            times.insert(h, v);
        }

        if let Some(name) = day_of_week.get(&day) {
            out.insert(name.clone(), times);
        }
    }

    out
}

fn decode_url(raw: &str) -> Result<String> {
    let quoted = format!("\"{}\"", raw.replace('"', "\\\""));
    let unquoted: String = serde_json::from_str(&quoted)?;
    Ok(unquoted)
}

fn extract_actual_url(google_url: &str) -> String {
    if google_url.starts_with("/url?q=") {
        if let Ok(parsed) = url::Url::parse(&format!("https://www.google.com{}", google_url)) {
            if let Some(q) = parsed.query_pairs().find(|(k, _)| k == "q").map(|(_, v)| v) {
                return q.to_string();
            }
        }
    }
    google_url.to_string()
}
