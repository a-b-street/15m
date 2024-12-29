<script lang="ts">
  import type { FeatureCollection } from "geojson";
  import { onMount } from "svelte";
  import { GeoJSON, hoverStateFilter, LineLayer } from "svelte-maplibre";
  import { notNull, PropertiesTable } from "svelte-utils";
  import { Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import {
    AmenityLayer,
    AmenityList,
    NavBar,
    PickProfile,
    StopsLayer,
  } from "./common";
  import { backend, filterForProfile, profile } from "./stores";

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

    <PickProfile bind:profile={$profile} />

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
          filter={filterForProfile($profile)}
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
