<script lang="ts">
  import { Modal, notNull } from "../common";
  import { PolygonToolLayer } from "maplibre-draw-polygon";
  import SplitComponent from "../SplitComponent.svelte";
  import { map, model, showAbout } from "../stores";
  import MapLoader from "./MapLoader.svelte";

  export let wasmReady: boolean;

  // When other modes reset here, they can't clear the model without a race condition
  $model = null;
</script>

<SplitComponent>
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

    {#if $map && wasmReady}
      <MapLoader />
    {:else}
      <p>Waiting for MapLibre and WASM to load...</p>
    {/if}
  </div>

  <div slot="map">
    <PolygonToolLayer />
  </div>
</SplitComponent>
