<script lang="ts">
  import type { Feature, Point } from "geojson";
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

  let isochroneGj: FeatureCollection | null = null;
  let routeGj: FeatureCollection | null = null;
  let err = "";

  let hoveredAmenity: Feature<Point> | null;

  $: if (start) {
    try {
      isochroneGj = JSON.parse(
        $model!.isochrone({
          x: start.lng,
          y: start.lat,
          mode: travelMode,
        }),
      );
      err = "";
    } catch (err: any) {
      isochroneGj = null;
      err = err.toString();
    }
  }

  $: if (start && hoveredAmenity) {
    try {
      routeGj = JSON.parse(
        $model!.route({
          x1: start.lng,
          y1: start.lat,
          x2: hoveredAmenity.geometry.coordinates[0],
          y2: hoveredAmenity.geometry.coordinates[1],
          mode: travelMode,
        }),
      );
      err = "";
    } catch (err: any) {
      routeGj = null;
      err = err.toString();
    }
  } else {
    routeGj = null;
  }

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
      <button on:click={() => ($mode = "title")}>Change study area</button>
      <button on:click={() => ($mode = "debug")}>Debug OSM</button>
    </div>

    <p>
      Move the pin to calculate an isochrone from that start. The cost is time
      in seconds.
    </p>

    <PickTravelMode bind:travelMode />

    <SequentialLegend {colorScale} limits={limitsMinutes} />
    {#if err}
      <p>{err}</p>
    {/if}

    {#if isochroneGj}
      <AmenityList gj={isochroneGj} />
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

    {#if isochroneGj}
      <GeoJSON data={isochroneGj}>
        <LineLayer
          id="isochrone"
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
    display: inline-block;
    background-color: blue;
    text-align: center;
    color: white;
    font-weight: bold;
  }
</style>
