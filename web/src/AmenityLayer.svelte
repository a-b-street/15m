<script lang="ts">
  import { notNull } from "./common";
  import { CircleLayer, hoverStateFilter } from "svelte-maplibre";
  import { PropertiesTable, Popup } from "svelte-utils";
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
>
  <Popup openOn="hover" let:props>
    <PropertiesTable properties={props} />
  </Popup>
</CircleLayer>
