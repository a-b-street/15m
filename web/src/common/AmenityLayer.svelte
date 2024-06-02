<script lang="ts">
  import { CircleLayer, hoverStateFilter } from "svelte-maplibre";
  import { PropertiesTable, notNull } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import type { Feature, Point } from "geojson";
  import type { Amenity } from "../stores";

  export let hovered: Feature<Point, Amenity> | null = null;
  export let popups = false;
</script>

<CircleLayer
  id="amenities"
  paint={{
    "circle-radius": 5,
    "circle-opacity": 0,
    "circle-stroke-width": 2,
    "circle-stroke-color": hoverStateFilter("orange", "red"),
  }}
  manageHoverState
  filter={["has", "amenity_kind"]}
  on:click={(e) =>
    window.open(notNull(e.detail.features[0].properties).osm_id, "_blank")}
  hoverCursor="pointer"
  eventsIfTopMost
  bind:hovered
>
  {#if popups}
    <Popup openOn="hover" let:props>
      <PropertiesTable properties={props} />
    </Popup>
  {/if}
</CircleLayer>
