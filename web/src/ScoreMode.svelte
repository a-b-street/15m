<script lang="ts">
  import * as Comlink from "comlink";
  import { Loading, NavBar, PickAmenityKinds } from "./common";
  import type { Feature, FeatureCollection, Point } from "geojson";
  import { colorScale } from "./colors";
  import { GeoJSON, CircleLayer, LineLayer } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { backend, type ScoreProps } from "./stores";
  import { SequentialLegend } from "svelte-utils";
  import { Popup, makeColorRamp } from "svelte-utils/map";

  let loading: string[] = [];
  let poiKinds: string[] = [];

  let gj: FeatureCollection<Point, ScoreProps> | null = null;

  $: updateScores(poiKinds);

  async function updateScores(_x: string[]) {
    loading = [...loading, "Calculating scores"];
    gj = await $backend!.score(
      {
        poiKinds,
      },
      Comlink.proxy(progressCb),
    );
    loading = [];
  }
  function progressCb(msg: string) {
    loading = [...loading, msg];
  }

  let routeGj: FeatureCollection | null = null;

  let limits = Array.from(Array(6).keys()).map(
    (i) => ((60 * 10) / (6 - 1)) * i,
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
          mode: "foot",
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
  <div slot="top"><NavBar /></div>
  <div slot="sidebar">
    <h2>Score mode</h2>

    <PickAmenityKinds bind:enabled={poiKinds} />

    <SequentialLegend {colorScale} {limits} />

    <p>
      This is an early experiment of a mode to show an "access score". Right
      now, it's starting from every POI based on the types chosen below and
      walking up to 10 minutes to the nearest bicycle parking. This is a simple
      way of showing POIs without any nearby parking. Note the granularity of
      results is poor; the search begins and ends at the nearest intersection,
      and the time to walk doesn't take into account the side of the road or
      walking partly down some road.
    </p>
  </div>
  <div slot="map">
    {#if gj}
      <GeoJSON data={gj}>
        <CircleLayer
          paint={{
            "circle-radius": 15,
            "circle-color": makeColorRamp(["get", "cost"], limits, colorScale),
          }}
          manageHoverState
          bind:hovered={hoveredAmenity}
          eventsIfTopMost
        >
          <Popup openOn="hover" let:props>
            From {props.poi}, it's {props.cost} seconds to the nearest parking
          </Popup>
        </CircleLayer>
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
