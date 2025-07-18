<script lang="ts">
  import * as Comlink from "comlink";
  import type { FeatureCollection } from "geojson";
  import { onDestroy, onMount } from "svelte";
  import { GeoJSON, hoverStateFilter, LineLayer } from "svelte-maplibre";
  import { SequentialLegend } from "svelte-utils";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BufferLayer from "./BufferLayer.svelte";
  import { colorScale } from "./colors";
  import { Loading, NavBar, PickProfile } from "./common";
  import {
    backend,
    bufferMins,
    map,
    profile,
    showRouteBuffer,
    showRouteBufferPopulation,
    startTime,
    type Profile,
  } from "./stores";

  let loading: string[] = [];

  let allInput: FeatureCollection | null = null;
  let input: FeatureCollection | null = null;
  let output: FeatureCollection | null = null;
  let totalPopulationInBuffer = 0;

  let showInput = true;
  let showOutput = true;
  let showOneInput = false;
  let oneFeatureId: number | null = null;

  $: if (showOneInput) {
    if (oneFeatureId == null) {
      oneFeatureId = 0;
    }
  } else {
    oneFeatureId = null;
  }
  $: if (allInput) {
    if (showOneInput && oneFeatureId != null) {
      input = {
        type: "FeatureCollection",
        features: [allInput.features[oneFeatureId]],
      };
    } else {
      input = allInput;
    }
  } else {
    input = null;
  }

  let fileInput: HTMLInputElement;
  async function loadFile(e: Event) {
    try {
      let gj = JSON.parse(
        await fileInput.files![0].text(),
      ) as FeatureCollection;
      gj.features = gj.features.filter((f) => f.geometry.type == "LineString");
      allInput = gj;
      showOneInput = false;
    } catch (err) {
      window.alert(`Couldn't snap routes from file: ${err}`);
    }
  }

  async function update(
    input: FeatureCollection | null,
    profile: Profile,
    _t: string,
    _b: number,
    _sb: boolean,
  ) {
    totalPopulationInBuffer = 0;
    output = null;
    if (!input) {
      return;
    }

    loading = ["Snapping routes"];
    try {
      output = await $backend!.snapAndBufferRoute(
        {
          input,
          profile: $profile,
          startTime: $startTime,
          maxSeconds: $bufferMins * 60,
        },
        Comlink.proxy(progressCb),
      );
      totalPopulationInBuffer = output.total_population;
    } catch (err) {
      window.alert(`Problem: ${err}`);
    }
    loading = [];
  }
  $: update(input, $profile, $startTime, $bufferMins, $showRouteBuffer);
  function progressCb(msg: string) {
    // TODO All of the work happens, then the logs get sent. Why?!
    //loading = [msg];
  }

  $: limits = Array.from(Array(6).keys()).map(
    (i) => (($bufferMins * 60) / (6 - 1)) * i,
  );

  onMount(() => {
    $map?.keyboard.disable();
  });
  onDestroy(() => {
    $map?.keyboard.enable();
  });

  function onKeyDown(e: KeyboardEvent) {
    if (oneFeatureId == null || allInput == null) {
      return;
    }
    if (e.key == "ArrowLeft") {
      e.stopPropagation();
      if (oneFeatureId > 0) {
        oneFeatureId--;
      }
    }
    if (e.key == "ArrowRight") {
      e.stopPropagation();
      if (oneFeatureId != allInput.features.length - 1) {
        oneFeatureId++;
      }
    }
  }
</script>

<svelte:window on:keydown={onKeyDown} />

<Loading {loading} />

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

    {#if input}
      <label>
        <input type="checkbox" bind:checked={showOneInput} />
        Show one input at a time
      </label>

      {#if oneFeatureId != null}
        <div>
          <button on:click={() => oneFeatureId--} disabled={oneFeatureId == 0}
            >Prev</button
          >
          {oneFeatureId + 1} / {allInput.features.length}
          <button
            on:click={() => oneFeatureId++}
            disabled={oneFeatureId == allInput.features.length - 1}>Next</button
          >
        </div>
      {/if}
    {/if}

    <label style:color="cyan">
      <input type="checkbox" bind:checked={showInput} />
      Show input
    </label>
    <label style:color="red">
      <input type="checkbox" bind:checked={showOutput} />
      Show snapped output
    </label>

    <PickProfile bind:profile={$profile} />

    {#if input}
      <label>
        <input type="checkbox" bind:checked={$showRouteBuffer} />
        Buffer around route (minutes)
        <input type="number" bind:value={$bufferMins} min="1" max="60" />
      </label>
      {#if $showRouteBuffer}
        <label>
          {totalPopulationInBuffer.toLocaleString()} people live in this buffer.
          Show:
          <input type="checkbox" bind:checked={$showRouteBufferPopulation} />
        </label>
        <SequentialLegend
          {colorScale}
          labels={{ limits: limits.map((l) => l / 60) }}
        />
      {/if}
    {/if}
  </div>
  <div slot="map">
    {#if input}
      <GeoJSON data={input} generateId>
        <LineLayer
          layout={{
            visibility: showInput ? "visible" : "none",
          }}
          paint={{
            "line-width": 20,
            "line-color": "cyan",
            "line-opacity": hoverStateFilter(0.5, 1.0),
          }}
          manageHoverState
          hoverCursor={showOneInput ? "inherit" : "pointer"}
          on:click={(e) => {
            showOneInput = true;
            oneFeatureId = e.detail.features[0].id;
          }}
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
            layout={{
              visibility: showOutput ? "visible" : "none",
            }}
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
