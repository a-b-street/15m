<script lang="ts">
  import type { Feature, Point } from "geojson";
  import AmenityLayer from "./AmenityLayer.svelte";
  import AmenityList from "./AmenityList.svelte";
  import { colorScale } from "./colors";
  import type { FeatureCollection } from "geojson";
  import { GeoJSON, FillLayer, LineLayer, Marker } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/two_column_layout";
  import { mode, backend, type TravelMode } from "./stores";
  import PickTravelMode from "./PickTravelMode.svelte";
  import { SequentialLegend } from "svelte-utils";
  import { Popup, makeColorRamp, isLine, isPolygon } from "svelte-utils/map";
  import { onMount } from "svelte";

  let travelMode: TravelMode = "foot";
  let startTime = "07:00";

  let start: { lng: number; lat: number } | null = null;
  onMount(async () => {
    // TODO Maybe need to do this when the file changes
    let bbox = await $backend!.getBounds();
    start = {
      lng: lerp(0.5, bbox[0], bbox[2]),
      lat: lerp(0.5, bbox[1], bbox[3]),
    };
  });
  let contours = true;

  let isochroneGj: FeatureCollection | null = null;
  let routeGj: FeatureCollection | null = null;
  let err = "";

  let hoveredAmenity: Feature<Point> | null;

  async function updateIsochrone(
    _x: { lng: number; lat: number } | null,
    _y: TravelMode,
    _z: boolean,
    _t: string,
  ) {
    if (start) {
      try {
        isochroneGj = await $backend!.isochrone({
          start,
          mode: travelMode,
          contours,
          startTime,
        });
        err = "";
      } catch (err: any) {
        isochroneGj = null;
        err = err.toString();
      }
    }
  }
  $: updateIsochrone(start, travelMode, contours, startTime);

  async function updateRoute(
    x: { lng: number; lat: number } | null,
    _y: Feature<Point> | null,
    _t: string,
  ) {
    if (start && hoveredAmenity) {
      try {
        routeGj = await $backend!.route({
          start,
          end: hoveredAmenity.geometry.coordinates,
          mode: travelMode,
          debugSearch: false,
          useHeuristic: false,
          startTime,
        });
        err = "";
      } catch (err: any) {
        routeGj = null;
        err = err.toString();
      }
    } else {
      routeGj = null;
    }
  }
  $: updateRoute(start, hoveredAmenity, startTime);

  function lerp(pct: number, a: number, b: number): number {
    return a + pct * (b - a);
  }

  let limitsMinutes = [0, 3, 6, 9, 12, 15];
  let limitsSeconds = limitsMinutes.map((x) => x * 60);
</script>

<SplitComponent>
  <div slot="sidebar">
    <h2>Isochrone mode</h2>
    <div>
      <button on:click={() => ($mode = { kind: "title" })}
        >Change study area</button
      >
      <button on:click={() => ($mode = { kind: "route" })}>Route</button>
      <button on:click={() => ($mode = { kind: "debug" })}>Debug OSM</button>
    </div>

    <p>
      Move the pin to calculate an isochrone from that start. The cost is time
      in seconds.
    </p>

    <PickTravelMode bind:travelMode />

    <label>
      Start time (PT only)
      <input type="time" bind:value={startTime} />
    </label>

    <label><input type="checkbox" bind:checked={contours} />Contours</label>

    <SequentialLegend {colorScale} limits={limitsMinutes} />
    {#if err}
      <p>{err}</p>
    {/if}

    {#if isochroneGj}
      <AmenityList gj={isochroneGj} />
    {/if}
  </div>
  <div slot="map">
    {#if start}
      <Marker bind:lngLat={start} draggable><span class="dot">X</span></Marker>
    {/if}

    {#if isochroneGj}
      <GeoJSON data={isochroneGj}>
        <LineLayer
          id="isochrone"
          filter={isLine}
          paint={{
            "line-width": 20,
            "line-color": makeColorRamp(
              ["get", "cost_seconds"],
              limitsSeconds,
              colorScale,
            ),
            "line-opacity": 0.5,
          }}
          eventsIfTopMost
        >
          <Popup openOn="hover" let:props>
            {(props.cost_seconds / 60).toFixed(1)} minutes away
          </Popup>
        </LineLayer>

        <FillLayer
          id="isochrone-contours"
          filter={isPolygon}
          paint={{
            "fill-color": makeColorRamp(
              ["get", "min_seconds"],
              limitsSeconds,
              colorScale,
            ),
            "fill-opacity": 0.5,
          }}
        />

        <AmenityLayer bind:hovered={hoveredAmenity} />
      </GeoJSON>
    {/if}

    {#if routeGj}
      <GeoJSON data={routeGj}>
        <LineLayer
          id="route"
          paint={{
            "line-width": 10,
            "line-color": "red",
          }}
        />
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
