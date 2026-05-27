use anyhow::{anyhow, Result};
use headless_chrome::{Browser, LaunchOptions};
use serde_json::Value;
use std::ffi::OsString;

use crate::models::Review;

pub struct BrowserClient {
    browser: Browser,
}

impl BrowserClient {
    pub fn new(headless: bool, proxy: Option<&str>) -> Result<Self> {
        let mut builder = LaunchOptions::default_builder();
        builder.headless(headless);
        if let Some(p) = proxy {
            let arg = OsString::from(format!("--proxy-server={}", p));
            let leaked: &'static std::ffi::OsStr = Box::leak(arg.into_boxed_os_str());
            builder.args(vec![leaked]);
        }
        let options = builder.build().map_err(|e| anyhow!("launch options: {e}"))?;
        let browser = Browser::new(options)?;
        Ok(Self { browser })
    }

    pub fn fetch_app_state(&self, url: &str) -> Result<String> {
        let tab = self.browser.new_tab()?;
        tab.navigate_to(url)?;
        tab.wait_until_navigated()?;

        let _ = tab.evaluate(
            r#"(() => {
                const consentForm = document.querySelector('form[action*="consent.google"]');
                if (consentForm) {
                    const btn = consentForm.querySelector('button, input[type="submit"]');
                    if (btn) { btn.click(); return true; }
                }
                const buttons = document.querySelectorAll('button, input[type="submit"]');
                for (const btn of buttons) {
                    const text = (btn.textContent || btn.value || '').toLowerCase();
                    if (text.includes('reject') || text.includes('decline') || text.includes('ablehnen')) {
                        btn.click();
                        return true;
                    }
                }
                return false;
            })()"#,
            true,
        );

        let js = r#"
            (function() {
                if (!window.APP_INITIALIZATION_STATE || !window.APP_INITIALIZATION_STATE[3]) {
                    return null;
                }
                const appState = window.APP_INITIALIZATION_STATE[3];
                for (const key of Object.keys(appState)) {
                    const arr = appState[key];
                    if (Array.isArray(arr)) {
                        for (const idx of [6, 5]) {
                            const item = arr[idx];
                            if (typeof item === 'string' && item.startsWith(")]}'")) {
                                return item;
                            }
                        }
                    }
                }
                return null;
            })()
        "#;

        let raw: Value = tab.evaluate(js, true)?.value.ok_or_else(|| anyhow!("no app state"))?;
        let s = raw.as_str().ok_or_else(|| anyhow!("unexpected app state type"))?;
        Ok(s.trim_start_matches(")]}'").trim().to_string())
    }

    pub fn fetch_place_links(&self, url: &str, depth: u32) -> Result<Vec<String>> {
        let tab = self.browser.new_tab()?;
        tab.navigate_to(url)?;
        tab.wait_until_navigated()?;

        let _ = tab.wait_for_element("div[role='feed']");

        for _ in 0..depth {
            let _ = tab.evaluate(
                r#"(() => {
                    const el = document.querySelector('div[role="feed"]');
                    if (el) {
                        el.scrollTop = el.scrollHeight;
                        return true;
                    }
                    window.scrollBy(0, 800);
                    return true;
                })()"#,
                true,
            );
            tab.wait_for_element("div[role='feed']").ok();
        }

        let js = r#"(() => {
            const out = [];
            const anchors = document.querySelectorAll('div[role="feed"] div[jsaction] > a');
            for (const a of anchors) {
                if (a.href) out.push(a.href);
            }
            return out;
        })()"#;

        let raw: Value = tab.evaluate(js, true)?.value.ok_or_else(|| anyhow!("no links"))?;
        let arr = raw.as_array().ok_or_else(|| anyhow!("unexpected links type"))?;
        let links = arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>();

        if !links.is_empty() {
            return Ok(links);
        }

        let current = tab.get_url();
        if current.contains("/maps/place/") {
            return Ok(vec![current]);
        }

        Ok(Vec::new())
    }

    pub fn fetch_dom_reviews(&self, url: &str, max_scrolls: u32) -> Result<Vec<Review>> {
        let tab = self.browser.new_tab()?;
        tab.navigate_to(url)?;
        tab.wait_until_navigated()?;

        let open_reviews_js = r#"(() => {
            const candidates = [
                'button[jsaction*="pane.reviewChart.moreReviews"]',
                'button[jsaction*="pane.review.more"]',
                'button[aria-label*="reviews"]',
                'button[aria-label*="Reviews"]',
                'a[href*="reviews"]'
            ];
            for (const sel of candidates) {
                const el = document.querySelector(sel);
                if (el) { el.click(); return true; }
            }
            return false;
        })()"#;
        let _ = tab.evaluate(open_reviews_js, true);

        for _ in 0..max_scrolls {
            let _ = tab.evaluate(
                r#"(() => {
                    const selectors = ['div[role="feed"]', '.m6QErb.DxyBCb.kA9KIf', '.DxyBCb.kA9KIf'];
                    for (const sel of selectors) {
                        const el = document.querySelector(sel);
                        if (el) { el.scrollBy(0, 800); return true; }
                    }
                    window.scrollBy(0, 800);
                    return true;
                })()"#,
                true,
            );
            tab.wait_until_navigated().ok();
        }

        let js = r#"(() => {
            const nodes = Array.from(document.querySelectorAll('div[data-review-id]'));
            const getText = (el, sel) => {
                const n = el.querySelector(sel);
                return n ? n.textContent.trim() : '';
            };
            const getRating = (el) => {
                const r = el.querySelector('span[aria-label*="star"]');
                if (!r) return 0;
                const m = r.getAttribute('aria-label').match(/([0-9.]+)/);
                return m ? parseFloat(m[1]) : 0;
            };
            return nodes.map(el => ({
                name: getText(el, '.d4r55'),
                description: getText(el, '.wiI7pd') || getText(el, '.MyEned'),
                rating: getRating(el),
                when: getText(el, '.rsqaWe'),
                profile_picture: (() => {
                    const img = el.querySelector('img');
                    return img ? img.src : '';
                })(),
            }));
        })()"#;

        let raw: Value = tab.evaluate(js, true)?.value.ok_or_else(|| anyhow!("no reviews"))?;
        let arr = raw.as_array().ok_or_else(|| anyhow!("unexpected reviews type"))?;

        let mut reviews = Vec::new();
        for item in arr {
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let description = item.get("description").and_then(|v| v.as_str()).unwrap_or_default();
            let when = item.get("when").and_then(|v| v.as_str()).unwrap_or_default();
            let profile_picture = item.get("profile_picture").and_then(|v| v.as_str()).unwrap_or_default();
            let rating = item.get("rating").and_then(|v| v.as_f64()).unwrap_or(0.0) as i32;
            reviews.push(Review {
                name: name.to_string(),
                profile_picture: profile_picture.to_string(),
                rating,
                description: description.to_string(),
                images: Vec::new(),
                when: when.to_string(),
            });
        }

        Ok(reviews)
    }
}
