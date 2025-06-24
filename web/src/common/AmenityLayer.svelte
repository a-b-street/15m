<script lang="ts">
  import type { Feature, Point } from "geojson";
  import { CircleLayer, hoverStateFilter } from "svelte-maplibre";
  import { notNull, PropertiesTable } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { hideAmenityKinds, type Amenity } from "../stores";

  export let hovered: Feature<Point, Amenity> | null = null;
  export let popups = false;

  // TODO This should be a method on the store
  function hideKinds(kinds: Map<string, boolean>): string[] {
    return [...kinds.entries()]
      .filter(([_, hide]) => hide)
      .map(([kind, _]) => kind);
  }
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
  filter={[
    "all",
    ["has", "amenity_kind"],
    [
      "!",
      [
        "in",
        ["get", "amenity_kind"],
        ["literal", hideKinds($hideAmenityKinds)],
      ],
    ],
  ]}
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
