<script lang="ts">
  import { PolygonToolLayer } from "maplibre-draw-polygon";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { Modal, notNull } from "svelte-utils";
  import { map, backend, showAbout, isLoaded } from "../stores";
  import MapLoader from "./MapLoader.svelte";
  import { onMount } from "svelte";

  // When other modes reset here, they can't clear without a race condition
  onMount(async () => {
    $isLoaded = false;
    await $backend!.unset();
  });
</script>

<SplitComponent>
  <div slot="top">modes</div>
  <div slot="sidebar">
    {#if $showAbout}
      <Modal on:close={() => ($showAbout = false)} let:dialog>
        <h1>15-minute neighborhood tool</h1>
        <p>TODO. Extremely early in development.</p>
        <p>
          This <a href="https://github.com/a-b-street/15m/" target="_blank"
            >open source</a
          >
          tool is created by
          <a href="https://github.com/dabreegster/" target="_blank"
            >Dustin Carlino</a
          >
          and relies heavily on
          <a href="https://www.openstreetmap.org/about" target="_blank"
            >OpenStreetMap</a
          > data.
        </p>

        <center
          ><button on:click={() => notNull(dialog).close()}>Start!</button
          ></center
        >
      </Modal>
    {/if}

    <h2>Choose your study area</h2>
    <button on:click={() => ($showAbout = true)}>About this tool</button>
    <hr />

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
