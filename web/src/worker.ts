import * as Comlink from "comlink";
import init, { MapModel } from "backend";
import type { TravelMode, ScoreProps, Amenity } from "./stores";
import type {
  Position,
  Feature,
  Point,
  Polygon,
  FeatureCollection,
} from "geojson";

export class Backend {
  inner: MapModel | null;

  constructor() {
    this.inner = null;
  }

  async loadOsmFile(
    osmBytes: Uint8Array,
    gtfsUrl: string | undefined,
    progressCb: (msg: string) => void,
  ) {
    // TODO Do we need to do this only once?
    await init();

    // TODO Can we await here?
    this.inner = await new MapModel(osmBytes, true, gtfsUrl, progressCb);
  }

  async loadGraphFile(graphBytes: Uint8Array) {
    // TODO Do we need to do this only once?
    await init();

    // No progress worth reporting for this
    // TODO Can we await here?
    this.inner = await new MapModel(graphBytes, false, undefined, undefined);
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

  renderDebug(): FeatureCollection {
    if (!this.inner) {
      throw new Error("Backend used without a file loaded");
    }

    return JSON.parse(this.inner.renderDebug());
  }

  renderAmenities(): FeatureCollection<Point, Amenity> {
    if (!this.inner) {
      throw new Error("Backend used without a file loaded");
    }

    return JSON.parse(this.inner.renderAmenities());
  }

  isochrone(req: {
    // TODO LngLatLike doesn't work?
    start: { lng: number; lat: number };
    mode: TravelMode;
    contours: boolean;
    startTime: string;
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
        start_time: req.startTime,
      }),
    );
  }

  route(req: {
    // TODO LngLatLike doesn't work?
    start: { lng: number; lat: number };
    end: Position;
    mode: TravelMode;
    debugSearch: boolean;
    useHeuristic: boolean;
    startTime: string;
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
        debug_search: req.debugSearch,
        use_heuristic: req.useHeuristic,
        start_time: req.startTime,
      }),
    );
  }

  score(
    req: {
      poiKinds: string[];
      maxSeconds: number;
    },
    progressCb: (msg: string) => void,
  ): FeatureCollection<Point, ScoreProps> {
    if (!this.inner) {
      throw new Error("Backend used without a file loaded");
    }

    return JSON.parse(
      this.inner.score(
        {
          poi_kinds: req.poiKinds,
          max_seconds: req.maxSeconds,
        },
        progressCb,
      ),
    );
  }
}

Comlink.expose(Backend);
