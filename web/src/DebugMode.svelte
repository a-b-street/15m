<script lang="ts">
  import { GeoJSON, hoverStateFilter, LineLayer } from "svelte-maplibre";
  import { notNull, PickTravelMode } from "./common";
  import SplitComponent from "./SplitComponent.svelte";
  import AmenityList from "./AmenityList.svelte";
  import AmenityLayer from "./AmenityLayer.svelte";
  import { PropertiesTable, Popup } from "svelte-utils";
  import { mode, model, type TravelMode, filterForMode } from "./stores";

  let travelMode: TravelMode = "foot";

  let gj = JSON.parse($model!.render());
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

    <AmenityList {gj} />
  </div>
  <div slot="map">
    <GeoJSON data={gj} generateId>
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

      <AmenityLayer />
    </GeoJSON>
  </div>
</SplitComponent>
