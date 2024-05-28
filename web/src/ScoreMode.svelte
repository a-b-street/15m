<script lang="ts">
  import type { FeatureCollection, Point } from "geojson";
  import { colorScale } from "./colors";
  import { GeoJSON, CircleLayer } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/two_column_layout";
  import { mode, backend } from "./stores";
  import { SequentialLegend } from "svelte-utils";
  import { Popup, makeColorRamp } from "svelte-utils/map";
  import { onMount } from "svelte";

  let gj: FeatureCollection<Point, { cost: number }> | null = null;
  onMount(async () => {
    gj = await $backend!.score();
    console.log(gj);
  });

  let limits = Array.from(Array(6).keys()).map(
    (i) => ((60 * 10) / (6 - 1)) * i,
  );
</script>

<SplitComponent>
  <div slot="sidebar">
    <h2>Score mode</h2>
    <div>
      <button on:click={() => ($mode = { kind: "title" })}
        >Change study area</button
      >
      <button on:click={() => ($mode = { kind: "isochrone" })}>Isochrone</button
      >
      <button on:click={() => ($mode = { kind: "route" })}>Route</button>
    </div>

    <p>
      This is an early experiment of a mode to show an "access score". Right
      now, it's starting from every POI of a few fixed types (cafe, pub,
      restaurant, bank, nightclub) and walking up to one minute to the nearest
      bicycle parking. This is a simple way of showing POIs without any nearby
      parking. Note the granularity of results is poor; the search begins and
      ends at the nearest intersection, and the time to walk doesn't take into
      account the side of the road or walking partly down some road.
    </p>

    <SequentialLegend {colorScale} {limits} />
  </div>
  <div slot="map">
    {#if gj}
      <GeoJSON data={gj}>
        <CircleLayer
          paint={{
            "circle-radius": 5,
            "circle-color": makeColorRamp(["get", "cost"], limits, colorScale),
          }}
          eventsIfTopMost
        >
          <Popup openOn="hover" let:props>
            {props.cost} seconds to the nearest parking
          </Popup>
        </CircleLayer>
      </GeoJSON>
    {/if}
  </div>
</SplitComponent>
