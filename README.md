# 15-minute neighborhood tool

This is an experimental rewrite of <15m.abstreet.org>. Stay tuned.

## Developer docs

### Installation

You'll need:
[npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm),
[wasm-pack](https://github.com/rustwasm/wasm-pack), and
[cargo](https://www.rust-lang.org/tools/install).

`cd web`, and then:

- `npm ci` to install dependencies (`ci` to make sure the versions in
  `package-lock.json` are used)
- `npm run wasm` to rebuild the Rust backend
  - vite doesn't automatically rebuild when you edit things
- `npm run dev` to run locally
  - Changes to the Svelte/CSS usually auto-reload in your browser
- `npm run fmt` to auto-format code
- `npm run check` to see TypeScript errors

### GTFS

The GTFS data used is built from the [UK BODS](https://data.bus-data.dft.gov.uk/). After downloading and unzipping, you can build using `cd cli; cargo run --release build-gtfs /path/to/gtfs`. The file is encoded using [geomedea](https://github.com/michaelkirk/geomedea).
