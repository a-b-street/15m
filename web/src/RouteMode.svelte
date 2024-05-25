<script lang="ts">
  import type { MapMouseEvent } from "maplibre-gl";
  import {
    MapEvents,
    GeoJSON,
    LineLayer,
    Marker,
    hoverStateFilter,
  } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/two_column_layout";
  import { mode, backend, type TravelMode } from "./stores";
  import PickTravelMode from "./PickTravelMode.svelte";
  import { Popup, constructMatchExpression } from "svelte-utils/map";
  import { notNull, PropertiesTable } from "svelte-utils";
  import { onMount } from "svelte";
  import type { FeatureCollection } from "geojson";

  let travelMode: TravelMode = "foot";

  let start: { lng: number; lat: number } | null = null;
  let end: { lng: number; lat: number } | null = null;
  onMount(async () => {
    // TODO Maybe need to do this when the file changes
    let bbox = await $backend!.getBounds();
    start = {
      lng: lerp(0.4, bbox[0], bbox[2]),
      lat: lerp(0.4, bbox[1], bbox[3]),
    };
    end = {
      lng: lerp(0.6, bbox[0], bbox[2]),
      lat: lerp(0.6, bbox[1], bbox[3]),
    };
  });

  let gj: FeatureCollection | null = null;
  let err = "";

  async function updateRoute(
    _x: { lng: number; lat: number } | null,
    _y: { lng: number; lat: number } | null,
    mode: TravelMode,
  ) {
    if (start && end) {
      try {
        gj = await $backend!.route({
          start,
          end: [end.lng, end.lat],
          mode,
        });
        err = "";
      } catch (error: any) {
        gj = null;
        err = error.toString();
      }
    }
  }
  $: updateRoute(start, end, travelMode);

  function onRightClick(e: CustomEvent<MapMouseEvent>) {
    // Move the first marker, for convenience
    start = e.detail.lngLat;
  }

  function lerp(pct: number, a: number, b: number): number {
    return a + pct * (b - a);
  }
</script>

<SplitComponent>
  <div slot="sidebar">
    <h2>Route mode</h2>
    <div>
      <button on:click={() => ($mode = "title")}>Change study area</button>
      <button on:click={() => ($mode = "isochrone")}>Isochrone mode</button>
    </div>
    <PickTravelMode bind:travelMode />
    <p>
      Move the <b>A</b> and <b>B</b> pins to find a route. (Hint: right-click to
      set the first pin somewhere.)
    </p>

    {#if err}
      <p>{err}</p>
    {:else if gj}
      <ol>
        {#each gj.features as f}
          {@const props = notNull(f.properties)}
          {#if props.kind == "road"}
            <li>Walk</li>
          {:else}
            <li>
              Take transit (trip {props.trip}) for {props.num_stops} stops
            </li>
          {/if}
        {/each}
      </ol>
    {/if}
  </div>
  <div slot="map">
    <MapEvents on:contextmenu={onRightClick} />

    {#if start}
      <Marker bind:lngLat={start} draggable><span class="dot">A</span></Marker>
    {/if}
    {#if end}
      <Marker bind:lngLat={end} draggable><span class="dot">B</span></Marker>
    {/if}

    {#if gj}
      <GeoJSON data={gj} generateId>
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
