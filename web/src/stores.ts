import { MapModel } from "backend";
import type { Map } from "maplibre-gl";
import { writable, type Writable } from "svelte/store";
import type { ExpressionSpecification } from "maplibre-gl";

export let maptilerApiKey = "MZEJTanw3WpxRvt7qDfo";

export let sidebarContents: Writable<HTMLDivElement | null> = writable(null);
export let mapContents: Writable<HTMLDivElement | null> = writable(null);

export type Mode = "title" | "debug" | "isochrone";

export let mode: Writable<Mode> = writable("title");
export let model: Writable<MapModel | null> = writable(null);
export let map: Writable<Map | null> = writable(null);
export let showAbout: Writable<boolean> = writable(true);

export type TravelMode = "car" | "bicycle" | "foot";

export function filterForMode(travelMode: TravelMode): ExpressionSpecification {
  return ["!=", ["get", `access_${travelMode}`], "None"];
}
