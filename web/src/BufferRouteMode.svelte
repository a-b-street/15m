<script lang="ts">
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { mode } from "./stores";
  import type { FeatureCollection } from "geojson";
  import { makeColorRamp, Popup } from "svelte-utils/map";
  import { SequentialLegend } from "svelte-utils";
  import { colorScale } from "./colors";

  export let gj: FeatureCollection;

  let limitsMinutes = [0, 1, 2, 3, 4, 5];
  let limitsSeconds = limitsMinutes.map((x) => x * 60);
</script>

<SplitComponent>
  <div slot="top">
    <button on:click={() => ($mode = { kind: "route" })}>Back</button>
  </div>
  <div slot="sidebar">
    <h2>Buffer around a route</h2>
    <SequentialLegend {colorScale} limits={limitsMinutes} />
  </div>
  <div slot="map">
    <GeoJSON data={gj}>
      <LineLayer
        paint={{
          "line-width": 20,
          "line-color": [
            "case",
            ["==", ["get", "kind"], "route"],
            "red",
            makeColorRamp(["get", "cost_seconds"], limitsSeconds, colorScale),
          ],
          "line-opacity": 0.5,
        }}
      >
        <Popup openOn="hover" let:props>
          {#if props.kind == "buffer"}
            {(props.cost_seconds / 60).toFixed(1)} minutes away
          {:else}
            part of the route
          {/if}
        </Popup>
      </LineLayer>
    </GeoJSON>
  </div>
</SplitComponent>
