import type { Map } from "maplibre-gl";
import { writable, type Writable } from "svelte/store";
import type { ExpressionSpecification } from "maplibre-gl";
import * as Comlink from "comlink";
import { type Backend } from "./worker";

export let maptilerApiKey = "MZEJTanw3WpxRvt7qDfo";

export let sidebarContents: Writable<HTMLDivElement | null> = writable(null);
export let mapContents: Writable<HTMLDivElement | null> = writable(null);

export type Mode = "title" | "debug" | "isochrone";

export let mode: Writable<Mode> = writable("title");
export let map: Writable<Map | null> = writable(null);
export let showAbout: Writable<boolean> = writable(true);

export type TravelMode = "car" | "bicycle" | "foot";

export function filterForMode(travelMode: TravelMode): ExpressionSpecification {
  return ["!=", ["get", `access_${travelMode}`], "None"];
}

// TODO Does this need to be a store?
export let backend: Writable<Comlink.Remote<Backend> | null> = writable(null);
// Indicates the backend is ready and a file is loaded
export let isLoaded = writable(false);
