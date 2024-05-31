<script lang="ts">
  import { PolygonToolLayer } from "maplibre-draw-polygon";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { map, backend, isLoaded } from "../stores";
  import MapLoader from "./MapLoader.svelte";
  import { onMount } from "svelte";

  // When other modes reset here, they can't clear without a race condition
  onMount(async () => {
    $isLoaded = false;
    await $backend!.unset();
  });
</script>

<SplitComponent>
  <div slot="top"></div>
  <div slot="sidebar">
    <h2>Choose your study area</h2>

    {#if $map}
      <MapLoader />
    {:else}
      <p>Waiting for MapLibre and WASM to load...</p>
    {/if}
  </div>

  <div slot="map">
    <PolygonToolLayer />
  </div>
</SplitComponent>
