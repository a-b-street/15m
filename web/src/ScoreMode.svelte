<script lang="ts">
  import * as Comlink from "comlink";
  import { Loading, NavBar, PickAmenityKinds, PickProfile } from "./common";
  import type { Feature, FeatureCollection, Point } from "geojson";
  import { colorScale } from "./colors";
  import {
    GeoJSON,
    CircleLayer,
    LineLayer,
    hoverStateFilter,
    SymbolLayer,
  } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { backend, profile, type ScoreProps } from "./stores";
  import { SequentialLegend, notNull } from "svelte-utils";
  import { makeColorRamp } from "svelte-utils/map";

  let loading: string[] = [];
  let maxSeconds = 600;
  let poiKinds: string[] = [];
  let showParking = true;

  let gj: FeatureCollection<Point, ScoreProps> | null = null;

  $: updateScores(poiKinds, maxSeconds, $profile);

  async function updateScores(_x: string[], _y: number, _z: string) {
    loading = [...loading, "Calculating scores"];
    gj = await $backend!.score(
      {
        profile: $profile,
        poiKinds,
        maxSeconds,
      },
      Comlink.proxy(progressCb),
    );
    loading = [];
  }
  function progressCb(msg: string) {
    loading = [...loading, msg];
  }

  let routeGj: FeatureCollection | null = null;

  $: limits = Array.from(Array(6).keys()).map(
    (i) => (maxSeconds / (6 - 1)) * i,
  );

  let hoveredAmenity: Feature<Point, ScoreProps> | null;

  async function updateRoute(_x: Feature<Point, ScoreProps> | null) {
    if (hoveredAmenity) {
      try {
        routeGj = await $backend!.route({
          start: {
            lng: hoveredAmenity.geometry.coordinates[0],
            lat: hoveredAmenity.geometry.coordinates[1],
          },
          end: [
            hoveredAmenity.properties.closest_lon,
            hoveredAmenity.properties.closest_lat,
          ],
          profile: $profile,
          debugSearch: false,
          useHeuristic: false,
          startTime: "07:00",
        });
      } catch (err) {
        console.log(`No route: ${err}`);
        routeGj = null;
      }
    } else {
      routeGj = null;
    }
  }
  $: updateRoute(hoveredAmenity);
</script>

{#if gj == null}
  <Loading {loading} />
{/if}

<SplitComponent>
  <div slot="top">
    <NavBar />
  </div>
  <div slot="sidebar">
    <h2>Score mode</h2>

    <PickProfile bind:profile={$profile} />

    {#if hoveredAmenity}
      <span
        >From {hoveredAmenity.properties.poi}, it's {hoveredAmenity.properties
          .cost} seconds to the nearest parking</span
      >
    {/if}

    <PickAmenityKinds bind:enabled={poiKinds} />

    <SequentialLegend {colorScale} {limits} />

    <label>
      <input type="number" bind:value={maxSeconds} />
      Max time (seconds)
    </label>

    <label>
      <input type="checkbox" bind:checked={showParking} />
      Show parking
    </label>

    <p>
      This is an early experiment of a mode to show an "access score". Right
      now, it's starting from every POI chosen and walking up to some time to
      the nearest bicycle parking. This is a simple way of showing POIs without
      any nearby parking. Note the granularity of results is poor; the search
      begins and ends at the nearest intersection, and the time to walk doesn't
      take into account the side of the road or walking partly down some road.
    </p>
    <p>
      Parking icon from <a
        href="https://github.com/gravitystorm/openstreetmap-carto"
        target="_blank">OpenStreetMap Carto</a
      >
    </p>
  </div>
  <div slot="map">
    {#if gj}
      <GeoJSON data={gj} generateId>
        <CircleLayer
          paint={{
            "circle-radius": 15,
            "circle-color": makeColorRamp(["get", "cost"], limits, colorScale),
            "circle-stroke-width": hoverStateFilter(1, 3),
            "circle-stroke-color": "black",
          }}
          manageHoverState
          bind:hovered={hoveredAmenity}
        />
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

    {#await notNull($backend).renderAmenities() then data}
      <GeoJSON {data}>
        <SymbolLayer
          filter={["==", ["get", "amenity_kind"], "bicycle_parking"]}
          layout={{
            "icon-image": "cycle_parking",
            "icon-size": 1.0,
            "icon-allow-overlap": true,
            visibility: showParking ? "visible" : "none",
          }}
        />
      </GeoJSON>
    {/await}
  </div>
</SplitComponent>
