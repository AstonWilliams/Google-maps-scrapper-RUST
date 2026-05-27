# Google Maps Scraper (Rust)

<!-- Header Image Placeholder -->
<!-- Add your hero image/banner here -->

[![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![CI](https://img.shields.io/github/actions/workflow/status/AstonWilliams/Google-maps-scrapper-RUST/rust-scraper-build.yml?branch=main)](https://github.com/AstonWilliams/Google-maps-scrapper-RUST/actions)
[![License](https://img.shields.io/github/license/AstonWilliams/Google-maps-scrapper-RUST)](LICENSE)
[![Stars](https://img.shields.io/github/stars/AstonWilliams/Google-maps-scrapper-RUST?style=social)](https://github.com/AstonWilliams/Google-maps-scrapper-RUST/stargazers)

High‑performance, cross‑platform Google Maps scraper written in Rust. Collects business listings with ratings, contact details, and location data. Supports fast HTTP mode for speed and full Chromium mode for richer fields.

Support the project:
- Star the repo: https://github.com/AstonWilliams/Google-maps-scrapper-RUST
- Donation link: https://usefulthings.gumroad.com/coffee

---

## Images

<!-- Screenshot Placeholder -->
<!-- Add terminal output screenshot here -->

<!-- Diagram Placeholder -->
<!-- Add architecture diagram here -->

---

## Features

| Feature | Description |
|---|---|
| Fast mode (HTTP) | Quick result extraction with lightweight HTTP requests |
| Full mode (Chromium) | Richer fields by parsing APP_INITIALIZATION_STATE |
| Concurrency | Parallel workers for faster throughput |
| Output formats | CSV and JSONL supported |
| Geo + radius | Target by coordinates and radius |
| Rating filters | Min/max rating filters and result limit |
| Proxies | HTTP/SOCKS proxy support |
| Email extraction | Optional website email scraping |
| Extra reviews | DOM-based extended review extraction |
| Auto Chromium install | Optional install prompt when missing |

---

## Build Instructions

```bash
git clone https://github.com/AstonWilliams/Google-maps-scrapper-RUST.git
cd Google-maps-scrapper-RUST/rust-scraper
cargo build --release
```

Binary output:
- target/release/gmaps-scraper-rs

---

## Example Commands

### Fast mode (CSV)
```bash
./target/release/gmaps-scraper-rs \
  --input example-queries.txt \
  --results results.csv \
  --fast-mode \
  --geo "48.8566,2.3522" \
  --zoom 13 \
  --radius 10000
```

### Full mode (Chromium)
```bash
./target/release/gmaps-scraper-rs \
  --input example-queries.txt \
  --results results.jsonl \
  --depth 5 \
  --concurrency 2 \
  --auto-install-chromium
```

### Filters (ratings + limit)
```bash
./target/release/gmaps-scraper-rs \
  --input example-queries.txt \
  --results results.csv \
  --fast-mode \
  --geo "48.8566,2.3522" \
  --zoom 13 \
  --radius 10000 \
  --min-rating 4.5 \
  --max-rating 5.0 \
  --max-results 20
```

### Proxies + concurrency
```bash
./target/release/gmaps-scraper-rs \
  --input example-queries.txt \
  --results results.jsonl \
  --concurrency 4 \
  --proxies "http://user:pass@host:port,socks5://host:port"
```

---

## Output Formats

- JSONL: use `--json`
- CSV: omit `--json`

---

## Changelog

| Version | Date | Notes |
|---|---|---|
| 0.1.0 | 2026-05-27 | Initial Rust CLI with fast mode, full mode, CSV/JSONL, filters, and proxies |

---

## Notes

- Fast mode returns fewer fields than full mode.
- Full mode uses Chromium to extract APP_INITIALIZATION_STATE.
- Use `--auto-install-chromium` to install Chromium on demand.

---

## Contributing

PRs are welcome. Please open an issue first for major changes.

---

## License

MIT
