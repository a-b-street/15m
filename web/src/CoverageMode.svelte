<script lang="ts">
  import type { FeatureCollection } from "geojson";
  import { FillLayer, GeoJSON, LineLayer, SymbolLayer } from "svelte-maplibre";
  import { notNull, SequentialLegend } from "svelte-utils";
  import { isLine, isPolygon, makeColorRamp, Popup } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { colorScale } from "./colors";
  import { NavBar, PickProfile } from "./common";
  import {
    backend,
    coverageMins,
    profile,
    startTime,
    type Profile,
  } from "./stores";

  let fromAmenity = "bicycle_parking";
  // TODO Generalize; show source amenity
  let showParking = true;

  let style = "Roads";

  let isochroneGj: FeatureCollection | null = null;
  let err = "";

  async function updateIsochrone(
    _x: string,
    _y: Profile,
    _z: string,
    _t: string,
    _im: number,
  ) {
    try {
      isochroneGj = await $backend!.isochroneFromAmenity({
        fromAmenity,
        profile: $profile,
        style,
        startTime: $startTime,
        maxSeconds: 60 * $coverageMins,
      });
      err = "";
    } catch (err: any) {
      isochroneGj = null;
      err = err.toString();
    }
  }
  $: updateIsochrone(fromAmenity, $profile, style, $startTime, $coverageMins);

  $: limits = Array.from(Array(6).keys()).map(
    (i) => (($coverageMins * 60) / (6 - 1)) * i,
  );
</script>

<SplitComponent>
  <div slot="top">
    <NavBar />
  </div>

  <div slot="sidebar">
    <h2>Coverage mode</h2>

    <label>
      <input type="checkbox" bind:checked={showParking} />
      Show parking
    </label>

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
      <input type="number" bind:value={$coverageMins} min="1" max="30" />
    </label>
    <SequentialLegend {colorScale} limits={limits.map((l) => l / 60)} />
    {#if err}
      <p>{err}</p>
    {/if}
  </div>

  <div slot="map">
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
      </GeoJSON>

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
    {/if}
  </div>
</SplitComponent>
