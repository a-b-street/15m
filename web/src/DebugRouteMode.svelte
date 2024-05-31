<script lang="ts">
  import { GeoJSON, CircleLayer, LineLayer, Marker } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { map, mode } from "./stores";
  import type { FeatureCollection } from "geojson";
  import { onMount, onDestroy } from "svelte";
  import { notNull } from "svelte-utils";
  import { constructMatchExpression } from "svelte-utils/map";

  export let debugGj: FeatureCollection;
  export let start: { lng: number; lat: number };
  export let end: { lng: number; lat: number };
  export let routeGj: FeatureCollection;

  let numNodes = debugGj.features.length / 2;

  let showSteps = 1;
  $: display = {
    type: "FeatureCollection" as const,
    features: debugGj.features.slice(0, 2 * showSteps),
  };

  function onKeyDown(e: KeyboardEvent) {
    if (e.key == "ArrowLeft" && showSteps > 1) {
      e.stopPropagation();
      showSteps--;
    }
    if (e.key == "ArrowRight" && showSteps < numNodes) {
      e.stopPropagation();
      showSteps++;
    }
  }

  onMount(() => {
    $map?.keyboard.disable();
  });
  onDestroy(() => {
    $map?.keyboard.enable();
  });
</script>

<svelte:window on:keydown={onKeyDown} />

<SplitComponent>
  <div slot="top">
    <button on:click={() => ($mode = { kind: "route" })}>Back</button>
  </div>
  <div slot="sidebar">
    <h2>Debugging a route</h2>

    <p>{numNodes} total nodes searched</p>

    <input type="range" min="1" max={numNodes} bind:value={showSteps} />

    <p>
      Search is currently at {notNull(
        display.features[display.features.length - 1].properties,
      ).time}
    </p>
  </div>
  <div slot="map">
    <Marker lngLat={start}><span class="dot">A</span></Marker>
    <Marker lngLat={end}><span class="dot">B</span></Marker>

    <GeoJSON data={routeGj}>
      <LineLayer
        paint={{
          "line-width": 20,
          "line-color": constructMatchExpression(
            ["get", "kind"],
            { road: "cyan", transit: "purple" },
            "red",
          ),
          "line-opacity": 0.5,
        }}
      />
    </GeoJSON>

    <GeoJSON data={display} generateId>
      <CircleLayer
        paint={{
          "circle-radius": 5,
          "circle-color": "black",
        }}
      />

      <LineLayer
        paint={{
          "line-width": 5,
          "line-color": constructMatchExpression(
            ["get", "kind"],
            { road: "black", transit: "orange" },
            "red",
          ),
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>

<style>
  .dot {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: flex;
    justify-content: center;
    align-items: center;

    color: white;
    background-color: blue;
    font-weight: bold;
  }
</style>
