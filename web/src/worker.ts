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
    populationUrl: string | undefined,
    progressCb: (msg: string) => void,
  ) {
    // TODO Do we need to do this only once?
    await init();

    this.inner = await new MapModel(
      osmBytes,
      gtfsUrl,
      populationUrl,
      progressCb,
    );
  }

  async loadModelFile(graphBytes: Uint8Array) {
    // TODO Do we need to do this only once?
    await init();

    this.inner = await MapModel.loadFile(graphBytes);
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

  renderZones(): FeatureCollection {
    if (!this.inner) {
      throw new Error("Backend used without a file loaded");
    }

    return JSON.parse(this.inner.renderZones());
  }

  isochrone(req: {
    // TODO LngLatLike doesn't work?
    start: { lng: number; lat: number };
    mode: TravelMode;
    style: string;
    startTime: string;
    maxSeconds: number;
  }): FeatureCollection {
    if (!this.inner) {
      throw new Error("Backend used without a file loaded");
    }

    return JSON.parse(
      this.inner.isochrone({
        x: req.start.lng,
        y: req.start.lat,
        mode: req.mode,
        style: req.style,
        start_time: req.startTime,
        max_seconds: req.maxSeconds,
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

  bufferRoute(req: {
    // TODO LngLatLike doesn't work?
    start: { lng: number; lat: number };
    end: Position;
    mode: TravelMode;
    useHeuristic: boolean;
    startTime: string;
    maxSeconds: number;
  }): FeatureCollection {
    if (!this.inner) {
      throw new Error("Backend used without a file loaded");
    }

    return JSON.parse(
      this.inner.bufferRoute({
        x1: req.start.lng,
        y1: req.start.lat,
        x2: req.end[0],
        y2: req.end[1],
        mode: req.mode,
        use_heuristic: req.useHeuristic,
        start_time: req.startTime,
        max_seconds: req.maxSeconds,
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

  snapAndBufferRoute(
    req: {
      input: FeatureCollection;
      mode: TravelMode;
      startTime: string;
      maxSeconds: number;
    },
    progressCb: (msg: string) => void,
  ): FeatureCollection {
    if (!this.inner) {
      throw new Error("Backend used without a file loaded");
    }

    return JSON.parse(
      this.inner.snapAndBufferRoute(
        {
          input: JSON.stringify(req.input),
          mode: req.mode,
          start_time: req.startTime,
          max_seconds: req.maxSeconds,
        },
        progressCb,
      ),
    );
  }
}

Comlink.expose(Backend);
