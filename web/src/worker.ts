import * as Comlink from "comlink";
import init, { MapModel } from "backend";
import type { TravelMode } from "./stores";
import type { Position, Feature, Polygon, FeatureCollection } from "geojson";

export class Backend {
  inner: MapModel | null;

  constructor() {
    this.inner = null;
  }

  async loadFile(osmBytes: Uint8Array, progressCb: (msg: string) => void) {
    // TODO Do we need to do this only once?
    await init();

    this.inner = new MapModel(osmBytes, progressCb);
  }

  isLoaded(): boolean {
    return this.inner != null;
  }

  unset() {
    this.inner = null;
  }

  getBounds(): [number, number, number, number] {
    if (!this.inner) {
      throw new Error("Backend used without a file loaded");
    }

    return Array.from(this.inner.getBounds()) as [
      number,
      number,
      number,
      number,
    ];
  }

  getInvertedBoundary(): Feature<Polygon> {
    if (!this.inner) {
      throw new Error("Backend used without a file loaded");
    }

    return JSON.parse(this.inner.getInvertedBoundary());
  }

  render(): FeatureCollection {
    if (!this.inner) {
      throw new Error("Backend used without a file loaded");
    }

    return JSON.parse(this.inner.render());
  }

  isochrone(req: {
    // TODO LngLatLike doesn't work?
    start: { lng: number; lat: number };
    mode: TravelMode;
    contours: boolean;
  }): FeatureCollection {
    if (!this.inner) {
      throw new Error("Backend used without a file loaded");
    }

    return JSON.parse(
      this.inner.isochrone({
        x: req.start.lng,
        y: req.start.lat,
        mode: req.mode,
        contours: req.contours,
      }),
    );
  }

  route(req: {
    // TODO LngLatLike doesn't work?
    start: { lng: number; lat: number };
    end: Position;
    mode: TravelMode;
  }): FeatureCollection {
    if (!this.inner) {
      throw new Error("Backend used without a file loaded");
    }

    return JSON.parse(
      this.inner.route({
        x1: req.start.lng,
        y1: req.start.lat,
        x2: req.end[0],
        y2: req.end[1],
        mode: req.mode,
      }),
    );
  }
}

Comlink.expose(Backend);
