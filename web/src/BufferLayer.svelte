<script lang="ts">
  import { LineLayer, FillLayer, hoverStateFilter } from "svelte-maplibre";
  import { showRouteBufferPopulation } from "./stores";
  import { Popup, makeColorRamp } from "svelte-utils/map";
  import { colorScale } from "./colors";

  export let totalPopulationInBuffer: number;
  export let limits: number[];
</script>

<LineLayer
  filter={["in", ["get", "kind"], ["literal", ["route", "buffer"]]]}
  paint={{
    "line-width": ["case", ["==", ["get", "kind"], "route"], 20, 3],
    "line-color": [
      "case",
      ["==", ["get", "kind"], "route"],
      "red",
      makeColorRamp(["get", "cost_seconds"], limits, colorScale),
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

{#if $showRouteBufferPopulation}
  {#if totalPopulationInBuffer == 0}
    <FillLayer
      filter={["==", ["get", "kind"], "hull"]}
      paint={{
        "fill-color": "black",
        "fill-opacity": 0.5,
      }}
    />
  {:else}
    <FillLayer
      filter={["==", ["get", "kind"], "zone_overlap"]}
      manageHoverState
      paint={{
        "fill-color": "black",
        "fill-opacity": hoverStateFilter(0.2, 0.8),
      }}
    >
      <Popup openOn="hover" let:props
        >{props.population.toLocaleString()} people live here (zone overlaps with
        buffer {Math.round(props.pct * 100)}%)</Popup
      >
    </FillLayer>
    <LineLayer
      filter={["==", ["get", "kind"], "zone_overlap"]}
      paint={{ "line-color": "black", "line-width": 1 }}
    />
  {/if}
{/if}
