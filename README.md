# 15-minute neighborhood tool

This is an experimental rewrite of <15m.abstreet.org>. Stay tuned.

## Developer docs

To build and run the web app locally, `cd web; npm run wasm; npm run dev`. You need to re-run `npm run wasm` when the Rust code in `backend` changes.

The GTFS data used is built from the [UK BODS](https://data.bus-data.dft.gov.uk/). After downloading and unzipping, you can build using `cd cli; cargo run --release build-gtfs /path/to/gtfs`. The file is encoded using [geomedea](https://github.com/michaelkirk/geomedea).
