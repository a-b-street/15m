import type { Map } from "maplibre-gl";
import { writable, type Writable } from "svelte/store";
import type { ExpressionSpecification } from "maplibre-gl";
import * as Comlink from "comlink";
import { type Backend } from "./worker";
import type { FeatureCollection } from "geojson";

export let maptilerApiKey = "MZEJTanw3WpxRvt7qDfo";

export type Mode =
  | { kind: "title" }
  | { kind: "debug" }
  | { kind: "isochrone" }
  | { kind: "route" }
  | { kind: "score" }
  | {
      kind: "debug-route";
      debugGj: FeatureCollection;
      start: { lng: number; lat: number };
      end: { lng: number; lat: number };
      routeGj: FeatureCollection;
    };

export let mode: Writable<Mode> = writable({ kind: "title" });
export let map: Writable<Map | null> = writable(null);
export let showAbout: Writable<boolean> = writable(true);

export type TravelMode = "car" | "bicycle" | "foot" | "transit";

export function filterForMode(travelMode: TravelMode): ExpressionSpecification {
  return ["!=", ["get", `access_${travelMode}`], "None"];
}

export let travelMode: Writable<TravelMode> = writable("foot");
export let startTime: Writable<string> = writable("07:00");

// Only used in RouteMode
export let routeA: Writable<{ lng: number; lat: number } | null> =
  writable(null);
export let routeB: Writable<{ lng: number; lat: number } | null> =
  writable(null);
export let useHeuristic = writable(true);

// TODO Does this need to be a store?
export let backend: Writable<Comlink.Remote<Backend> | null> = writable(null);
// Indicates the backend is ready and a file is loaded
export let isLoaded = writable(false);

export interface ScoreProps {
  cost: number;
  poi: string;
  closest_lon: number;
  closest_lat: number;
}
