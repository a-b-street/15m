<script lang="ts">
  import type { Feature, FeatureCollection, Point } from "geojson";
  import { onMount } from "svelte";
  import { FillLayer, GeoJSON, LineLayer, Marker } from "svelte-maplibre";
  import { SequentialLegend } from "svelte-utils";
  import { isLine, isPolygon, makeColorRamp, Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { colorScale } from "./colors";
  import { AmenityLayer, AmenityList, NavBar, PickProfile } from "./common";
  import {
    backend,
    describeAmenity,
    isochroneMins,
    profile,
    startTime,
    type Amenity,
    type Profile,
  } from "./stores";

  let start: { lng: number; lat: number } | null = null;
  onMount(async () => {
    // TODO Maybe need to do this when the file changes
    let bbox = await $backend!.getBounds();
    start = {
      lng: lerp(0.5, bbox[0], bbox[2]),
      lat: lerp(0.5, bbox[1], bbox[3]),
    };
  });
  let style = "Roads";

  let isochroneGj: FeatureCollection | null = null;
  let routeGj: FeatureCollection | null = null;
  let err = "";

  let hoveredAmenity: Feature<Point, Amenity> | null;

  async function updateIsochrone(
    _x: { lng: number; lat: number } | null,
    _y: Profile,
    _z: string,
    _t: string,
    _im: number,
  ) {
    if (start) {
      try {
        isochroneGj = await $backend!.isochrone({
          start,
          profile: $profile,
          style,
          startTime: $startTime,
          maxSeconds: 60 * $isochroneMins,
        });
        err = "";
      } catch (err: any) {
        isochroneGj = null;
        err = err.toString();
      }
    }
  }
  $: updateIsochrone(start, $profile, style, $startTime, $isochroneMins);

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
          profile: $profile,
          debugSearch: false,
          useHeuristic: false,
          startTime: $startTime,
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
  $: updateRoute(start, hoveredAmenity, $startTime);

  function lerp(pct: number, a: number, b: number): number {
    return a + pct * (b - a);
  }

  $: limits = Array.from(Array(6).keys()).map(
    (i) => (($isochroneMins * 60) / (6 - 1)) * i,
  );
</script>

<SplitComponent>
  <div slot="top">
    <NavBar />
  </div>
  <div slot="sidebar">
    <h2>Isochrone mode</h2>

    <p>
      Move the pin to calculate an isochrone from that start. The cost is time
      in seconds.
    </p>
    {#if hoveredAmenity}
      <span
        >{describeAmenity(hoveredAmenity)} ({hoveredAmenity.properties
          .amenity_kind})</span
      >
    {/if}

    <PickProfile bind:profile={$profile} />

    <label>
      Start time (PT only)
      <input
        type="time"
        bind:value={$startTime}
        disabled={$profile != "transit"}
      />
    </label>

    <label
      >Draw:
      <select bind:value={style}>
        <option value="Roads">Roads</option>
        <option value="Grid">Grid</option>
        <option value="Contours">Contours</option>
      </select>
    </label>

    <label
      >Minutes away
      <input type="number" bind:value={$isochroneMins} min="1" max="30" />
    </label>
    <SequentialLegend {colorScale} limits={limits.map((l) => l / 60)} />
    {#if err}
      <p>{err}</p>
    {/if}

    {#if isochroneGj}
      <hr />
      <AmenityList gj={isochroneGj} />
    {/if}
  </div>
  <div slot="map">
    {#if start}
      <Marker bind:lngLat={start} draggable><span class="dot">X</span></Marker>
    {/if}

    {#if isochroneGj}
      <GeoJSON data={isochroneGj} generateId>
        <LineLayer
          id="isochrone"
          filter={isLine}
          paint={{
            "line-width": 2,
            "line-color": makeColorRamp(
              ["get", "cost_seconds"],
              limits,
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
              limits,
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
