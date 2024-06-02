<script lang="ts">
  import {
    StopsLayer,
    PickTravelMode,
    AmenityList,
    AmenityLayer,
    NavBar,
  } from "./common";
  import { GeoJSON, hoverStateFilter, LineLayer } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { PropertiesTable, notNull } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { backend, travelMode, filterForMode } from "./stores";
  import { onMount } from "svelte";
  import type { FeatureCollection } from "geojson";

  let gj: FeatureCollection | null = null;
  onMount(async () => {
    gj = await $backend!.renderDebug();
  });
</script>

<SplitComponent>
  <div slot="top"><NavBar /></div>
  <div slot="sidebar">
    <h2>Debug mode</h2>
    <p>Hover to see a segment's properties, and click to open OSM</p>

    <PickTravelMode bind:travelMode={$travelMode} />

    {#if gj}
      <AmenityList {gj} />
    {/if}
  </div>
  <div slot="map">
    {#if gj}
      <GeoJSON data={gj} generateId>
        <LineLayer
          id="network"
          paint={{
            "line-width": hoverStateFilter(5, 7),
            "line-color": "black",
          }}
          filter={filterForMode($travelMode)}
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

        <AmenityLayer popups />
        <StopsLayer />
      </GeoJSON>
    {/if}
  </div>
</SplitComponent>
