<script lang="ts">
  import { GeoJSON, CircleLayer, LineLayer } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/two_column_layout";
  import { map, mode } from "./stores";
  import type { FeatureCollection } from "geojson";
  import { onMount, onDestroy } from "svelte";
  import { notNull } from "svelte-utils";

  export let gj: FeatureCollection;

  let numNodes = gj.features.length / 2;

  let showSteps = 1;
  $: display = {
    type: "FeatureCollection" as const,
    features: gj.features.slice(0, 2 * showSteps),
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
  <div slot="sidebar">
    <h2>Debugging a route</h2>
    <div>
      <button on:click={() => ($mode = { kind: "route" })}>Back</button>
    </div>

    <p>{gj.features.length / 2} total nodes searched</p>

    <input
      type="range"
      min="1"
      max={gj.features.length / 2}
      bind:value={showSteps}
    />

    <p>
      Search is currently at {notNull(
        display.features[display.features.length - 1].properties,
      ).time}
    </p>
  </div>
  <div slot="map">
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
          "line-color": "black",
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>
