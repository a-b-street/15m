<script lang="ts">
  import { PickTravelMode, NavBar } from "./common";
  import type { MapMouseEvent } from "maplibre-gl";
  import {
    MapEvents,
    GeoJSON,
    LineLayer,
    Marker,
    hoverStateFilter,
  } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import {
    mode,
    backend,
    travelMode,
    type TravelMode,
    startTime,
    useHeuristic,
    routeA,
    routeB,
    bufferMins,
    showRouteBuffer,
    showRouteBufferPopulation,
  } from "./stores";
  import { Popup, constructMatchExpression } from "svelte-utils/map";
  import { notNull, SequentialLegend, PropertiesTable } from "svelte-utils";
  import type { FeatureCollection } from "geojson";
  import { colorScale } from "./colors";
  import BufferLayer from "./BufferLayer.svelte";

  let gj: FeatureCollection | null = null;
  let totalPopulationInBuffer = 0;
  let err = "";

  async function update(
    start: { lng: number; lat: number },
    end: { lng: number; lat: number },
    mode: TravelMode,
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
          mode: $travelMode,
          useHeuristic: $useHeuristic,
          startTime: $startTime,
          maxSeconds: $bufferMins * 60,
        });
        totalPopulationInBuffer = gj.total_population;
      } else {
        gj = await $backend!.route({
          start,
          end: [end.lng, end.lat],
          mode,
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
    $travelMode,
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
        mode: $travelMode,
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

    <PickTravelMode bind:travelMode={$travelMode} />

    <label>
      <input
        type="checkbox"
        bind:checked={$useHeuristic}
        disabled={$travelMode != "transit"}
      />
      Use heuristic (PT only)
    </label>

    <label>
      Start time (PT only)
      <input
        type="time"
        bind:value={$startTime}
        disabled={$travelMode != "transit"}
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
      <SequentialLegend {colorScale} limits={limits.map((l) => l / 60)} />
    {/if}

    <p>
      Move the <b>A</b> and <b>B</b> pins to find a route. (Hint: right-click to
      set the first pin somewhere.)
    </p>

    {#if err}
      <p>{err}</p>
    {:else if gj}
      <button on:click={debugRoute} disabled={$travelMode != "transit"}
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
