<script lang="ts">
  import "@picocss/pico/css/pico.jade.min.css";
  import { Geocoder, notNull } from "svelte-utils";
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
    backend,
    sidebarContents,
    maptilerApiKey,
    isLoaded,
  } from "./stores";
  import TitleMode from "./title/TitleMode.svelte";
  import workerWrapper from "./worker?worker";
  import { type Backend } from "./worker";
  import * as Comlink from "comlink";

  onMount(async () => {
    // If you get "import declarations may only appear at top level of a
    // module", then you need a newer browser.
    // https://caniuse.com/mdn-api_worker_worker_ecmascript_modules
    //
    // In Firefox 112, go to about:config and enable dom.workers.modules.enabled
    //
    // Note this should work fine in older browsers when doing 'npm run build'.
    // It's only a problem during local dev mode.
    interface WorkerConstructor {
      new (): Backend;
    }

    const MyWorker: Comlink.Remote<WorkerConstructor> = Comlink.wrap(
      new workerWrapper(),
    );
    let backendWorker = await new MyWorker();
    backend.set(backendWorker);
  });

  let map: Map;
  $: if (map) {
    mapStore.set(map);
  }

  async function zoomToFit() {
    if (map && $isLoaded) {
      map.fitBounds(await $backend!.getBounds(), { animate: false });
    }
  }

  async function gotModel(ready: boolean) {
    if (ready) {
      console.log("New map model loaded");
      await zoomToFit();
      $mode = "isochrone";
    }
  }
  $: gotModel($isLoaded);

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
        <TitleMode />
      {/if}
      {#if $isLoaded}
        {#await notNull($backend).getInvertedBoundary() then data}
          <GeoJSON {data}>
            <FillLayer paint={{ "fill-color": "black", "fill-opacity": 0.3 }} />
          </GeoJSON>
        {/await}
        {#if $mode == "debug"}
          <DebugMode />
        {:else if $mode == "isochrone"}
          <IsochroneMode />
        {/if}
      {/if}
    </MapLibre>
  </div>
</Layout>
