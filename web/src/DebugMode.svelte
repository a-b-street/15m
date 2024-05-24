<script lang="ts">
  import { GeoJSON, hoverStateFilter, LineLayer } from "svelte-maplibre";
  import PickTravelMode from "./PickTravelMode.svelte";
  import { SplitComponent } from "svelte-utils/two_column_layout";
  import AmenityList from "./AmenityList.svelte";
  import AmenityLayer from "./AmenityLayer.svelte";
  import StopsLayer from "./StopsLayer.svelte";
  import { PropertiesTable, notNull } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { mode, backend, type TravelMode, filterForMode } from "./stores";
  import { onMount } from "svelte";
  import type { FeatureCollection } from "geojson";

  let travelMode: TravelMode = "foot";

  let gj: FeatureCollection | null = null;
  onMount(async () => {
    gj = await $backend!.renderDebug();
  });
</script>

{#if gj}
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
        <StopsLayer />
      </GeoJSON>
    </div>
  </SplitComponent>
{/if}
