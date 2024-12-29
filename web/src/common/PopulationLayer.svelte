<script lang="ts">
  import type { FeatureCollection } from "geojson";
  import {
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    LineLayer,
  } from "svelte-maplibre";
  import { makeColorRamp, Popup } from "svelte-utils/map";
  import { populationColorScale } from "../colors";

  export let gj: FeatureCollection;

  // TODO Should the limits be fixed? But this varies so much regionally
  let limits = Array.from(Array(6).keys()).map(
    (i) => (gj.max_density / (6 - 1)) * i,
  );
</script>

<GeoJSON data={gj} generateId>
  <FillLayer
    manageHoverState
    paint={{
      "fill-color": makeColorRamp(
        ["get", "density"],
        limits,
        populationColorScale,
      ),
      "fill-opacity": hoverStateFilter(0.2, 0.8),
    }}
  >
    <Popup openOn="hover" let:props
      >{props.population.toLocaleString()} people live here ({Math.round(
        props.density,
      ).toLocaleString()} people / square kilometer)</Popup
    >
  </FillLayer>
  <LineLayer paint={{ "line-color": "black", "line-width": 1 }} />
</GeoJSON>
