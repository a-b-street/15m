<script lang="ts">
  import type { FeatureCollection } from "geojson";
  import type { MapMouseEvent } from "maplibre-gl";
  import {
    GeoJSON,
    hoverStateFilter,
    LineLayer,
    MapEvents,
    Marker,
  } from "svelte-maplibre";
  import { notNull, PropertiesTable, SequentialLegend } from "svelte-utils";
  import { constructMatchExpression, Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BufferLayer from "./BufferLayer.svelte";
  import { colorScale } from "./colors";
  import { NavBar, PickProfile } from "./common";
  import {
    backend,
    bufferMins,
    mode,
    profile,
    routeA,
    routeB,
    showRouteBuffer,
    showRouteBufferPopulation,
    startTime,
    useHeuristic,
    type Profile,
  } from "./stores";

  let gj: FeatureCollection | null = null;
  let totalPopulationInBuffer = 0;
  let err = "";

  async function update(
    start: { lng: number; lat: number },
    end: { lng: number; lat: number },
    profile: Profile,
    _z: boolean,
    _t: string,
    _b: number,
    _sb: boolean,
  ) {
    try {
      totalPopulationInBuffer = 0;
      if ($showRouteBuffer) {
        gj = await $backend!.bufferRoute({
          start: $routeA!,
          end: [$routeB!.lng, $routeB!.lat],
          profile: $profile,
          useHeuristic: $useHeuristic,
          startTime: $startTime,
          maxSeconds: $bufferMins * 60,
        });
        totalPopulationInBuffer = gj.total_population;
      } else {
        gj = await $backend!.route({
          start,
          end: [end.lng, end.lat],
          profile: $profile,
          debugSearch: false,
          useHeuristic: $useHeuristic,
          startTime: $startTime,
        });
      }
      err = "";
    } catch (error: any) {
      gj = null;
      err = error.toString();
    }
  }
  $: update(
    $routeA!,
    $routeB!,
    $profile,
    $useHeuristic,
    $startTime,
    $bufferMins,
    $showRouteBuffer,
  );

  function onRightClick(e: CustomEvent<MapMouseEvent>) {
    // Move the first marker, for convenience
    $routeA = e.detail.lngLat;
  }

  async function debugRoute() {
    try {
      let debugGj = await $backend!.route({
        start: $routeA!,
        end: [$routeB!.lng, $routeB!.lat],
        profile: $profile,
        debugSearch: true,
        useHeuristic: $useHeuristic,
        startTime: $startTime,
      });
      $mode = {
        kind: "debug-route",
        debugGj,
        start: $routeA!,
        end: $routeB!,
        routeGj: gj!,
      };
    } catch (error: any) {
      err = error.toString();
    }
  }

  $: limits = Array.from(Array(6).keys()).map(
    (i) => (($bufferMins * 60) / (6 - 1)) * i,
  );
</script>

<SplitComponent>
  <div slot="top"><NavBar /></div>
  <div slot="sidebar">
    <h2>Route mode</h2>

    <PickProfile bind:profile={$profile} />

    <label>
      <input
        type="checkbox"
        bind:checked={$useHeuristic}
        disabled={$profile != "transit"}
      />
      Use heuristic (PT only)
    </label>

    <label>
      Start time (PT only)
      <input
        type="time"
        bind:value={$startTime}
        disabled={$profile != "transit"}
      />
    </label>

    <label>
      <input type="checkbox" bind:checked={$showRouteBuffer} />
      Buffer around route (minutes)
      <input type="number" bind:value={$bufferMins} min="1" max="60" />
    </label>
    {#if $showRouteBuffer}
      <label>
        {totalPopulationInBuffer.toLocaleString()} people live in this buffer. Show:
        <input type="checkbox" bind:checked={$showRouteBufferPopulation} />
      </label>
      <SequentialLegend
        {colorScale}
        labels={{ limits: limits.map((l) => l / 60) }}
      />
    {/if}

    <p>
      Move the <b>A</b> and <b>B</b> pins to find a route. (Hint: right-click to
      set the first pin somewhere.)
    </p>

    {#if err}
      <p>{err}</p>
    {:else if gj}
      <button on:click={debugRoute} disabled={$profile != "transit"}
        >Watch how this route was found (PT only)</button
      >

      {#if !$showRouteBuffer}
        <ol>
          {#each gj.features as f}
            {@const props = notNull(f.properties)}
            {#if props.kind == "road"}
              {#if props.time1}
                <li>[{props.time1} - {props.time2}] Walk</li>
              {:else}
                <li>Walk / cycle / drive</li>
              {/if}
            {:else}
              <li>
                [{props.time1} - {props.time2}] Take {props.route} for {props.num_stops}
                stops
              </li>
            {/if}
          {/each}
        </ol>
      {/if}
    {/if}
  </div>
  <div slot="map">
    <MapEvents on:contextmenu={onRightClick} />

    {#if $routeA && $routeB}
      <Marker bind:lngLat={$routeA} draggable><span class="dot">A</span></Marker
      >
      <Marker bind:lngLat={$routeB} draggable><span class="dot">B</span></Marker
      >
    {/if}

    {#if gj}
      <GeoJSON data={gj} generateId>
        {#if $showRouteBuffer}
          <BufferLayer {totalPopulationInBuffer} {limits} />
        {:else}
          <LineLayer
            id="route"
            paint={{
              "line-width": 20,
              "line-color": constructMatchExpression(
                ["get", "kind"],
                { road: "cyan", transit: "purple" },
                "red",
              ),
              "line-opacity": hoverStateFilter(0.5, 1.0),
            }}
            manageHoverState
          >
            <Popup openOn="hover" let:props>
              <PropertiesTable properties={props} />
            </Popup>
          </LineLayer>
        {/if}
      </GeoJSON>
    {/if}
  </div>
</SplitComponent>

<style>
  .dot {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: flex;
    justify-content: center;
    align-items: center;

    color: white;
    background-color: blue;
    font-weight: bold;
  }
</style>
