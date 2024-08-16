<script lang="ts">
  import "@picocss/pico/css/pico.jade.min.css";
  import cycleParking from "../assets/bicycle_parking.png?url";
  import logoDark from "../assets/logo_dark.svg?url";
  import About from "./About.svelte";
  import { notNull } from "svelte-utils";
  import { Geocoder } from "svelte-utils/map";
  import type { Map } from "maplibre-gl";
  import { onMount } from "svelte";
  import { FillLayer, GeoJSON, MapLibre } from "svelte-maplibre";
  import {
    Layout,
    mapContents,
    sidebarContents,
    topContents,
  } from "svelte-utils/top_bar_layout";
  import DebugMode from "./DebugMode.svelte";
  import IsochroneMode from "./IsochroneMode.svelte";
  import RouteMode from "./RouteMode.svelte";
  import DebugRouteMode from "./DebugRouteMode.svelte";
  import BufferRouteMode from "./BufferRouteMode.svelte";
  import ScoreMode from "./ScoreMode.svelte";
  import {
    map as mapStore,
    mode,
    backend,
    maptilerApiKey,
    isLoaded,
    showAbout,
    routeA,
    routeB,
    showPopulation,
  } from "./stores";
  import TitleMode from "./title/TitleMode.svelte";
  import workerWrapper from "./worker?worker";
  import { type Backend } from "./worker";
  import * as Comlink from "comlink";
  import { PopulationLayer } from "./common";

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

      let bbox = await $backend!.getBounds();
      $routeA = {
        lng: lerp(0.4, bbox[0], bbox[2]),
        lat: lerp(0.4, bbox[1], bbox[3]),
      };
      $routeB = {
        lng: lerp(0.6, bbox[0], bbox[2]),
        lat: lerp(0.6, bbox[1], bbox[3]),
      };

      await zoomToFit();
      $mode = { kind: "isochrone" };
    }
  }
  $: gotModel($isLoaded);

  function lerp(pct: number, a: number, b: number): number {
    return a + pct * (b - a);
  }

  let topDiv: HTMLSpanElement;
  let sidebarDiv: HTMLDivElement;
  let mapDiv: HTMLDivElement;
  $: if (topDiv && $topContents) {
    topDiv.innerHTML = "";
    topDiv.appendChild($topContents);
  }
  $: if (sidebarDiv && $sidebarContents) {
    sidebarDiv.innerHTML = "";
    sidebarDiv.appendChild($sidebarContents);
  }
  $: if (mapDiv && $mapContents) {
    mapDiv.innerHTML = "";
    mapDiv.appendChild($mapContents);
  }
</script>

<About />
<Layout>
  <div slot="top" style="display: flex">
    <button class="outline" on:click={() => ($showAbout = true)}>
      <img src={logoDark} style="height: 6vh;" alt="A/B Street logo" />
    </button>
    <span bind:this={topDiv} style="width: 100%" />
  </div>
  <div slot="left">
    <h1>15-minute neighbourhood tool</h1>
    <div bind:this={sidebarDiv} />

    {#if $mode.kind != "title"}
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
      images={[{ id: "cycle_parking", url: cycleParking }]}
      on:error={(e) => {
        // @ts-expect-error ErrorEvent isn't exported
        console.log(e.detail.error);
      }}
    >
      <Geocoder {map} apiKey={maptilerApiKey} />
      <div bind:this={mapDiv} />

      {#if $mode.kind == "title"}
        <TitleMode />
      {/if}
      {#if $isLoaded}
        {#await notNull($backend).getInvertedBoundary() then data}
          <GeoJSON {data}>
            <FillLayer paint={{ "fill-color": "black", "fill-opacity": 0.3 }} />
          </GeoJSON>
        {/await}
        {#if $mode.kind == "debug"}
          <DebugMode />
        {:else if $mode.kind == "isochrone"}
          <IsochroneMode />
        {:else if $mode.kind == "route"}
          <RouteMode />
        {:else if $mode.kind == "score"}
          <ScoreMode />
        {:else if $mode.kind == "debug-route"}
          <DebugRouteMode
            debugGj={$mode.debugGj}
            start={$mode.start}
            end={$mode.end}
            routeGj={$mode.routeGj}
          />
        {:else if $mode.kind == "buffer-route"}
          <BufferRouteMode gj={$mode.gj} />
        {/if}

        {#if $showPopulation}
          {#await notNull($backend).renderZones() then gj}
            <PopulationLayer {gj} />
          {/await}
        {/if}
      {/if}
    </MapLibre>
  </div>
</Layout>

<style>
  :global(.maplibregl-popup-content) {
    background-color: var(--pico-background-color);
  }
</style>
