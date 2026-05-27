use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Image {
    pub title: String,
    pub image: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinkSource {
    pub link: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Owner {
    pub id: String,
    pub name: String,
    pub link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Address {
    pub borough: String,
    pub street: String,
    pub city: String,
    pub postal_code: String,
    pub state: String,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OptionItem {
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct About {
    pub id: String,
    pub name: String,
    pub options: Vec<OptionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Review {
    pub name: String,
    pub profile_picture: String,
    pub rating: i32,
    pub description: String,
    pub images: Vec<String>,
    pub when: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Entry {
    pub input_id: String,
    pub link: String,
    pub cid: String,
    pub title: String,
    pub categories: Vec<String>,
    pub category: String,
    pub address: String,
    pub open_hours: HashMap<String, Vec<String>>,
    pub popular_times: HashMap<String, HashMap<i32, i32>>,
    pub website: String,
    pub phone: String,
    pub plus_code: String,
    pub review_count: i32,
    pub review_rating: f64,
    pub reviews_per_rating: HashMap<i32, i32>,
    pub latitude: f64,
    pub longitude: f64,
    pub status: String,
    pub description: String,
    pub reviews_link: String,
    pub thumbnail: String,
    pub timezone: String,
    pub price_range: String,
    pub data_id: String,
    pub place_id: String,
    pub images: Vec<Image>,
    pub reservations: Vec<LinkSource>,
    pub order_online: Vec<LinkSource>,
    pub menu: LinkSource,
    pub owner: Owner,
    pub complete_address: Address,
    pub about: Vec<About>,
    pub user_reviews: Vec<Review>,
    pub user_reviews_extended: Vec<Review>,
    pub emails: Vec<String>,
}
