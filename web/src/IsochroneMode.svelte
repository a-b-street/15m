<script lang="ts">
  import AmenityLayer from "./AmenityLayer.svelte";
  import AmenityList from "./AmenityList.svelte";
  import { colorScale } from "./colors";
  import type { FeatureCollection } from "geojson";
  import { GeoJSON, LineLayer, Marker } from "svelte-maplibre";
  import SplitComponent from "./SplitComponent.svelte";
  import { mode, model, type TravelMode, filterForMode } from "./stores";
  import { makeColorRamp, PickTravelMode, notNull } from "./common";
  import { SequentialLegend, Popup } from "svelte-utils";

  let travelMode: TravelMode = "foot";

  // TODO Maybe need to do this when model changes
  let bbox: number[] = Array.from($model!.getBounds());
  let start = {
    lng: lerp(0.5, bbox[0], bbox[2]),
    lat: lerp(0.5, bbox[1], bbox[3]),
  };

  let gj: FeatureCollection | null = null;
  let err = "";

  $: if (start) {
    try {
      gj = JSON.parse(
        $model!.isochrone({
          x: start.lng,
          y: start.lat,
          mode: travelMode,
        }),
      );
      err = "";
    } catch (err: any) {
      gj = null;
      err = err.toString();
    }
  }

  function lerp(pct: number, a: number, b: number): number {
    return a + pct * (b - a);
  }

  let limits = [0, 200, 400, 600, 800, 1000];
</script>

<SplitComponent>
  <div slot="sidebar">
    <h2>Isochrone mode</h2>
    <div>
      <button on:click={() => ($mode = "title")}>Change study area</button>
      <button on:click={() => ($mode = "debug")}>Debug OSM</button>
    </div>

    <p>
      Move the pin to calculate an isochrone from that start. The cost is
      distance in meters.
    </p>

    <PickTravelMode bind:travelMode />

    <SequentialLegend {colorScale} {limits} />
    {#if err}
      <p>{err}</p>
    {/if}

    {#if gj}
      <AmenityList {gj} />
    {/if}
  </div>
  <div slot="map">
    <GeoJSON data={JSON.parse(notNull($model).render())}>
      <LineLayer
        id="network"
        paint={{
          "line-width": 5,
          "line-color": "black",
          "line-opacity": ["case", filterForMode(travelMode), 1, 0.5],
        }}
      />
    </GeoJSON>

    <Marker bind:lngLat={start} draggable><span class="dot">X</span></Marker>
    {#if gj}
      <GeoJSON data={gj}>
        <LineLayer
          id="isochrone"
          paint={{
            "line-width": 20,
            "line-color": makeColorRamp(
              ["get", "cost_meters"],
              limits,
              colorScale,
            ),
            "line-opacity": 0.5,
          }}
          eventsIfTopMost
        >
          <Popup openOn="hover" let:props>
            {props.cost_meters} m away
          </Popup>
        </LineLayer>

        <AmenityLayer />
      </GeoJSON>
    {/if}
  </div>
</SplitComponent>

<style>
  .dot {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: inline-block;
    background-color: blue;
    text-align: center;
    color: white;
    font-weight: bold;
  }
</style>