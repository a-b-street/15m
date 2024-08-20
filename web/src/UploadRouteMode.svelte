<script lang="ts">
  import { NavBar, PickTravelMode } from "./common";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import type { FeatureCollection } from "geojson";
  import {
    backend,
    travelMode,
    bufferMins,
    showRouteBuffer,
    showRouteBufferPopulation,
    startTime,
    type TravelMode,
  } from "./stores";
  import { GeoJSON, LineLayer, hoverStateFilter } from "svelte-maplibre";
  import BufferLayer from "./BufferLayer.svelte";
  import { SequentialLegend } from "svelte-utils";
  import { colorScale } from "./colors";

  let input: FeatureCollection | null = null;
  let output: FeatureCollection | null = null;
  let totalPopulationInBuffer = 0;

  let fileInput: HTMLInputElement;
  async function loadFile(e: Event) {
    try {
      let gj = JSON.parse(
        await fileInput.files![0].text(),
      ) as FeatureCollection;
      gj.features = gj.features.filter((f) => f.geometry.type == "LineString");
      input = gj;
    } catch (err) {
      window.alert(`Couldn't snap routes from file: ${err}`);
    }
  }

  async function update(
    input: FeatureCollection | null,
    mode: TravelMode,
    _t: string,
    _b: number,
    _sb: boolean,
  ) {
    totalPopulationInBuffer = 0;
    output = null;
    if (!input) {
      return;
    }

    try {
      output = await $backend!.snapAndBufferRoute({
        input,
        mode: $travelMode,
        startTime: $startTime,
        maxSeconds: $bufferMins * 60,
      });
      totalPopulationInBuffer = output.total_population;
    } catch (err) {
      window.alert(`Problem: ${err}`);
    }
  }
  $: update(input, $travelMode, $startTime, $bufferMins, $showRouteBuffer);

  $: limits = Array.from(Array(6).keys()).map(
    (i) => (($bufferMins * 60) / (6 - 1)) * i,
  );
</script>

<SplitComponent>
  <div slot="top"><NavBar /></div>
  <div slot="sidebar">
    <h2>Upload Route mode</h2>
    <p>
      This is an experimental tool to snap routes drawn elsewhere to this
      network for analysis.
    </p>
    <label>
      Select a GeoJSON file with LineStrings:
      <input bind:this={fileInput} on:change={loadFile} type="file" />
    </label>

    <PickTravelMode bind:travelMode={$travelMode} />

    {#if input}
      <label>
        <input type="checkbox" bind:checked={$showRouteBuffer} />
        Buffer around route (minutes)
        <input type="number" bind:value={$bufferMins} min="1" max="30" />
      </label>
      {#if $showRouteBuffer}
        <label>
          {totalPopulationInBuffer.toLocaleString()} people live in this buffer.
          Show:
          <input type="checkbox" bind:checked={$showRouteBufferPopulation} />
        </label>
        <SequentialLegend {colorScale} limits={limits.map((l) => l / 60)} />
      {/if}
    {/if}
  </div>
  <div slot="map">
    {#if input}
      <GeoJSON data={input} generateId>
        <LineLayer
          paint={{
            "line-width": 20,
            "line-color": "cyan",
            "line-opacity": hoverStateFilter(0.5, 1.0),
          }}
          manageHoverState
        />
      </GeoJSON>
    {/if}

    {#if output}
      <GeoJSON data={output} generateId>
        {#if $showRouteBuffer}
          <BufferLayer {totalPopulationInBuffer} {limits} />
        {:else}
          <LineLayer
            filter={["==", ["get", "kind"], "route"]}
            paint={{
              "line-width": 20,
              "line-color": "red",
              "line-opacity": hoverStateFilter(0.5, 1.0),
            }}
            manageHoverState
          />
        {/if}
      </GeoJSON>
    {/if}
  </div>
</SplitComponent>
