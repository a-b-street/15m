<script lang="ts">
  import {
    CircleLayer,
    GeoJSON,
    hoverStateFilter,
    LineLayer,
  } from "svelte-maplibre";
  import { notNull, PickTravelMode } from "./common";
  import SplitComponent from "./SplitComponent.svelte";
  import { PropertiesTable, Popup } from "svelte-utils";
  import { mode, model, type TravelMode, filterForMode } from "./stores";

  let travelMode: TravelMode = "foot";
</script>

<SplitComponent>
  <div slot="sidebar">
    <h2>Debug mode</h2>
    <div>
      <button on:click={() => ($mode = "title")}>Change study area</button>
      <button on:click={() => ($mode = "isochrone")}>Isochrones</button>
    </div>
    <p>Hover to see a segment's properties, and click to open OSM</p>

    <PickTravelMode bind:travelMode />
  </div>
  <div slot="map">
    <GeoJSON data={JSON.parse(notNull($model).render())} generateId>
      <LineLayer
        id="network"
        paint={{
          "line-width": hoverStateFilter(5, 7),
          "line-color": "black",
        }}
        filter={filterForMode(travelMode)}
        manageHoverState
        on:click={(e) =>
          window.open(notNull(e.detail.features[0].properties).way, "_blank")}
        hoverCursor="pointer"
        eventsIfTopMost
      >
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </LineLayer>

      <CircleLayer
        id="amenities"
        paint={{
          "circle-radius": 10,
          "circle-color": "cyan",
        }}
        manageHoverState
        on:click={(e) =>
          window.open(
            notNull(e.detail.features[0].properties).osm_id,
            "_blank",
          )}
        hoverCursor="pointer"
        eventsIfTopMost
      >
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </CircleLayer>
    </GeoJSON>
  </div>
</SplitComponent>
