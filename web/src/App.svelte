<script lang="ts">
  import "@picocss/pico/css/pico.jade.min.css";
  import init, { MapModel } from "backend";
  import { Geocoder } from "svelte-utils";
  import type { Map } from "maplibre-gl";
  import { onMount } from "svelte";
  import { FillLayer, GeoJSON, MapLibre } from "svelte-maplibre";
  import { Layout } from "./common";
  import DebugMode from "./DebugMode.svelte";
  import IsochroneMode from "./IsochroneMode.svelte";
  import {
    mapContents,
    map as mapStore,
    mode,
    model,
    sidebarContents,
    maptilerApiKey,
  } from "./stores";
  import TitleMode from "./title/TitleMode.svelte";

  let wasmReady = false;
  onMount(async () => {
    await init();
    wasmReady = true;
  });

  let map: Map;
  $: if (map) {
    mapStore.set(map);
  }

  function zoomToFit() {
    if (map && $model) {
      map.fitBounds(
        Array.from($model.getBounds()) as [number, number, number, number],
        { animate: false },
      );
    }
  }

  function gotModel(_m: MapModel | null) {
    if (!$model) {
      return;
    }
    console.log("New map model loaded");
    zoomToFit();
    $mode = "isochrone";
  }
  $: gotModel($model);

  let sidebarDiv: HTMLDivElement;
  let mapDiv: HTMLDivElement;
  $: if (sidebarDiv && $sidebarContents) {
    sidebarDiv.innerHTML = "";
    sidebarDiv.appendChild($sidebarContents);
  }
  $: if (mapDiv && $mapContents) {
    mapDiv.innerHTML = "";
    mapDiv.appendChild($mapContents);
  }
</script>

<Layout>
  <div slot="left">
    <h1>15-minute neighbourhood tool</h1>
    <div bind:this={sidebarDiv} />

    {#if $mode != "title"}
      <hr />
      <div><button on:click={zoomToFit}>Zoom to fit</button></div>
    {/if}
  </div>
  <div slot="main" style="position:relative; width: 100%; height: 100vh;">
    <MapLibre
      style={`https://api.maptiler.com/maps/dataviz/style.json?key=${maptilerApiKey}`}
      standardControls
      hash
      bind:map
    >
      <Geocoder {map} apiKey={maptilerApiKey} />
      <div bind:this={mapDiv} />

      {#if $mode == "title"}
        <TitleMode {wasmReady} />
      {/if}
      {#if $model}
        <GeoJSON data={JSON.parse($model.getInvertedBoundary())}>
          <FillLayer paint={{ "fill-color": "black", "fill-opacity": 0.3 }} />
        </GeoJSON>
        {#if $mode == "debug"}
          <DebugMode />
        {:else if $mode == "isochrone"}
          <IsochroneMode />
        {/if}
      {/if}
    </MapLibre>
  </div>
</Layout>
