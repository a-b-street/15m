<script lang="ts">
  import { NavBar, PickTravelMode } from "./common";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import type { FeatureCollection } from "geojson";
  import { backend, travelMode } from "./stores";
  import { GeoJSON, LineLayer, hoverStateFilter } from "svelte-maplibre";

  let input: FeatureCollection | null = null;
  let output: FeatureCollection | null = null;

  let fileInput: HTMLInputElement;
  async function loadFile(e: Event) {
    try {
      let gj = JSON.parse(
        await fileInput.files![0].text(),
      ) as FeatureCollection;
      gj.features = gj.features.filter((f) => f.geometry.type == "LineString");
      input = gj;

      output = await $backend!.snapRoute({
        input: input!,
        mode: $travelMode,
      });
    } catch (err) {
      window.alert(`Couldn't snap routes from file: ${err}`);
    }
  }
</script>

<SplitComponent>
  <div slot="top"><NavBar /></div>
  <div slot="sidebar">
    <h2>Upload Route mode</h2>
    <p>
      This is an experimental tool to snap routes drawn elsewhere to this
      network for analysis.
    </p>
    <label>
      Select a GeoJSON file with LineStrings:
      <input bind:this={fileInput} on:change={loadFile} type="file" />
    </label>

    <PickTravelMode bind:travelMode={$travelMode} />
  </div>
  <div slot="map">
    {#if input}
      <GeoJSON data={input} generateId>
        <LineLayer
          paint={{
            "line-width": 20,
            "line-color": "cyan",
            "line-opacity": hoverStateFilter(0.5, 1.0),
          }}
          manageHoverState
        />
      </GeoJSON>
    {/if}

    {#if output}
      <GeoJSON data={output} generateId>
        <LineLayer
          paint={{
            "line-width": 20,
            "line-color": "red",
            "line-opacity": hoverStateFilter(0.5, 1.0),
          }}
          manageHoverState
        />
      </GeoJSON>
    {/if}
  </div>
</SplitComponent>
