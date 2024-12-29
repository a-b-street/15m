import init, { MapModel } from "backend";
import * as Comlink from "comlink";
import type {
  Feature,
  FeatureCollection,
  Point,
  Polygon,
  Position,
} from "geojson";
import type { Amenity, Profile, ScoreProps } from "./stores";

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
    profile: Profile;
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
        from_amenity: "",
        profile: req.profile == "transit" ? "foot" : req.profile,
        transit: req.profile == "transit",
        style: req.style,
        start_time: req.startTime,
        max_seconds: req.maxSeconds,
      }),
    );
  }

  isochroneFromAmenity(req: {
    fromAmenity: string;
    profile: Profile;
    style: string;
    startTime: string;
    maxSeconds: number;
  }): FeatureCollection {
    if (!this.inner) {
      throw new Error("Backend used without a file loaded");
    }

    return JSON.parse(
      this.inner.isochrone({
        x: 0,
        y: 0,
        from_amenity: req.fromAmenity,
        profile: req.profile == "transit" ? "foot" : req.profile,
        transit: req.profile == "transit",
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
    profile: Profile;
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
        profile: req.profile == "transit" ? "foot" : req.profile,
        transit: req.profile == "transit",
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
    profile: Profile;
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
        profile: req.profile == "transit" ? "foot" : req.profile,
        transit: req.profile == "transit",
        use_heuristic: req.useHeuristic,
        start_time: req.startTime,
        max_seconds: req.maxSeconds,
      }),
    );
  }

  score(
    req: {
      profile: Profile;
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
          // TODO Note transit won't work here
          profile: req.profile,
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
      profile: Profile;
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
          // TODO Note transit won't work here
          profile: req.profile,
          start_time: req.startTime,
          max_seconds: req.maxSeconds,
        },
        progressCb,
      ),
    );
  }
}

Comlink.expose(Backend);
