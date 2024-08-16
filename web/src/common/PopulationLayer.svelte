<script lang="ts">
  import {
    GeoJSON,
    LineLayer,
    FillLayer,
    hoverStateFilter,
  } from "svelte-maplibre";
  import { notNull } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { backend } from "../stores";
</script>

{#await notNull($backend).renderZones() then data}
  <GeoJSON {data} generateId>
    <FillLayer
      manageHoverState
      paint={{
        "fill-color": "red",
        "fill-opacity": hoverStateFilter(0.2, 0.8),
      }}
    >
      <Popup openOn="hover" let:props>{props.population.toLocaleString()}</Popup
      >
    </FillLayer>
    <LineLayer paint={{ "line-color": "black", "line-width": 1 }} />
  </GeoJSON>
{/await}
