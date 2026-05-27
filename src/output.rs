use crate::models::Entry;
use anyhow::Result;
use csv::Writer;
use std::fs::File;
use std::io::{BufWriter, Write};

pub enum OutputWriter {
    Json(BufWriter<Box<dyn Write>>),
    Csv(Writer<Box<dyn Write>>),
}

impl OutputWriter {
    pub fn new(path: Option<&str>, json: bool) -> Result<Self> {
        let writer: Box<dyn Write> = match path {
            Some(p) => Box::new(File::create(p)?),
            None => Box::new(std::io::stdout()),
        };
        if json {
            Ok(Self::Json(BufWriter::new(writer)))
        } else {
            let mut wtr = Writer::from_writer(writer);
            wtr.write_record(csv_headers())?;
            Ok(Self::Csv(wtr))
        }
    }

    pub fn write_entry(&mut self, entry: &Entry) -> Result<()> {
        match self {
            OutputWriter::Json(w) => {
                let line = serde_json::to_string(entry)?;
                writeln!(w, "{}", line)?;
            }
            OutputWriter::Csv(w) => {
                w.write_record(csv_row(entry))?;
            }
        }
        Ok(())
    }
}

fn csv_headers() -> Vec<&'static str> {
    vec![
        "input_id",
        "link",
        "title",
        "category",
        "address",
        "open_hours",
        "popular_times",
        "website",
        "phone",
        "plus_code",
        "review_count",
        "review_rating",
        "reviews_per_rating",
        "latitude",
        "longitude",
        "cid",
        "status",
        "descriptions",
        "reviews_link",
        "thumbnail",
        "timezone",
        "price_range",
        "data_id",
        "place_id",
        "images",
        "reservations",
        "order_online",
        "menu",
        "owner",
        "complete_address",
        "about",
        "user_reviews",
        "user_reviews_extended",
        "emails",
    ]
}

fn csv_row(entry: &Entry) -> Vec<String> {
    vec![
        entry.input_id.clone(),
        entry.link.clone(),
        entry.title.clone(),
        entry.category.clone(),
        entry.address.clone(),
        to_json(&entry.open_hours),
        to_json(&entry.popular_times),
        entry.website.clone(),
        entry.phone.clone(),
        entry.plus_code.clone(),
        entry.review_count.to_string(),
        entry.review_rating.to_string(),
        to_json(&entry.reviews_per_rating),
        entry.latitude.to_string(),
        entry.longitude.to_string(),
        entry.cid.clone(),
        entry.status.clone(),
        entry.description.clone(),
        entry.reviews_link.clone(),
        entry.thumbnail.clone(),
        entry.timezone.clone(),
        entry.price_range.clone(),
        entry.data_id.clone(),
        entry.place_id.clone(),
        to_json(&entry.images),
        to_json(&entry.reservations),
        to_json(&entry.order_online),
        to_json(&entry.menu),
        to_json(&entry.owner),
        to_json(&entry.complete_address),
        to_json(&entry.about),
        to_json(&entry.user_reviews),
        to_json(&entry.user_reviews_extended),
        entry.emails.join(", "),
    ]
}

fn to_json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_headers_contains_input_id() {
        let headers = csv_headers();
        assert!(headers.contains(&"input_id"));
        assert!(headers.contains(&"place_id"));
    }
}
