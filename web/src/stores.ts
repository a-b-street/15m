import * as Comlink from "comlink";
import type { Feature, FeatureCollection, Point } from "geojson";
import type { ExpressionSpecification, Map as MaplibreMap } from "maplibre-gl";
import { writable, type Writable } from "svelte/store";
import { type Backend } from "./worker";

export let maptilerApiKey = "MZEJTanw3WpxRvt7qDfo";

export type Mode =
  | { kind: "title" }
  | { kind: "debug" }
  | { kind: "isochrone" }
  | { kind: "route" }
  | { kind: "score" }
  | { kind: "coverage" }
  | {
      kind: "debug-route";
      debugGj: FeatureCollection;
      start: { lng: number; lat: number };
      end: { lng: number; lat: number };
      routeGj: FeatureCollection;
    }
  | { kind: "upload-route" };

export let mode: Writable<Mode> = writable({ kind: "title" });
export let map: Writable<MaplibreMap | null> = writable(null);
export let showAbout: Writable<boolean> = writable(true);
export let showPopulation: Writable<boolean> = writable(false);

export type Profile = "car" | "bicycle" | "foot" | "transit";

export function filterForProfile(profile: Profile): ExpressionSpecification {
  return ["!=", ["get", `access_${profile}`], "None"];
}

export let profile: Writable<Profile> = writable("foot");
export let startTime: Writable<string> = writable("07:00");

// Only used in RouteMode
export let routeA: Writable<{ lng: number; lat: number } | null> =
  writable(null);
export let routeB: Writable<{ lng: number; lat: number } | null> =
  writable(null);
export let useHeuristic = writable(true);
export let showRouteBuffer = writable(false);
export let showRouteBufferPopulation = writable(false);
export let isochroneMins = writable(15);
export let coverageMins = writable(1);
export let bufferMins = writable(5);

// TODO Does this need to be a store?
export let backend: Writable<Comlink.Remote<Backend> | null> = writable(null);
// Indicates the backend is ready and a file is loaded
export let isLoaded = writable(false);

export let hideAmenityKinds: Writable<Map<string, boolean>> = writable(
  new Map(),
);

// ----
// TODO Move to another file

export interface ScoreProps {
  cost: number;
  poi: string;
  closest_lon: number;
  closest_lat: number;
}

export interface Amenity {
  amenity_kind: string;
  osm_id: string;
  name?: string;
  brand?: string;
  cuisine?: string;
}

export function describeAmenity(f: Feature<Point, Amenity>): string {
  let label = f.properties.name || `a ${f.properties.amenity_kind}`;
  if (f.properties.brand) {
    label += ` (${f.properties.brand})`;
  }
  if (f.properties.cuisine) {
    label += ` (${f.properties.cuisine})`;
  }
  return label;
}
