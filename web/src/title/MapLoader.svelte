<script lang="ts">
  import * as Comlink from "comlink";
  import type { Feature, Polygon } from "geojson";
  import { onMount } from "svelte";
  import { OverpassSelector } from "svelte-utils/overpass";
  import { Loading } from "../common";
  import { backend, isLoaded, map } from "../stores";

  let example = "";
  let loading: string[] = [];
  let useLocalVite = false;
  let exampleAreas: [string, [string, string][]][] = [];

  onMount(async () => {
    // When running locally if a vite public/ directory is set up, load from that for speed
    try {
      let resp = await fetch("/osm/areas.json");
      if (resp.ok) {
        useLocalVite = true;
        console.log("Using local cache, not od2net.org");
        exampleAreas = await resp.json();
      } else {
        let resp = await fetch(
          `https://assets.od2net.org/severance_pbfs/areas.json`,
        );
        exampleAreas = await resp.json();
      }

      // For quicker dev
      //example = "kowloon";
    } catch (err) {}
  });

  let osmFileInput: HTMLInputElement;
  async function loadOsmFile(e: Event) {
    try {
      await loadModel(await osmFileInput.files![0].arrayBuffer());
      example = "";
    } catch (err) {
      window.alert(`Couldn't open this file: ${err}`);
    }
    loading = [];
  }

  let modelFileInput: HTMLInputElement;
  async function loadModelFile(e: Event) {
    try {
      loading = ["Loading pre-built file"];
      let buffer = await modelFileInput.files![0].arrayBuffer();
      console.time("load");
      await $backend!.loadModelFile(new Uint8Array(buffer));
      console.timeEnd("load");
      $isLoaded = true;

      example = "";
    } catch (err) {
      window.alert(`Couldn't open this file: ${err}`);
    }
    loading = [];
  }

  async function loadModel(buffer: ArrayBuffer) {
    let gtfsUrl = useLocalVite
      ? `http://${window.location.host}/15m/gtfs.gmd`
      : "https://assets.od2net.org/gtfs.gmd";
    let populationUrl = useLocalVite
      ? `http://${window.location.host}/15m/population.fgb`
      : "https://assets.od2net.org/population.fgb";
    loading = ["Building map model from OSM input"];
    console.time("load");
    await $backend!.loadOsmFile(
      new Uint8Array(buffer),
      gtfsUrl,
      populationUrl,
      Comlink.proxy(progressCb),
    );
    console.timeEnd("load");
    $isLoaded = true;
  }

  function progressCb(msg: string) {
    loading = [...loading, msg];
  }

  async function gotXml(
    e: CustomEvent<{ xml: string; boundary: Feature<Polygon> }>,
  ) {
    try {
      // TODO Can we avoid turning into bytes?
      await loadModel(new TextEncoder().encode(e.detail.xml));
      example = "";
    } catch (err) {
      window.alert(`Couldn't import from Overpass: ${err}`);
    }
    loading = [];
  }

  async function loadExample(example: string) {
    if (example != "") {
      if (useLocalVite) {
        await loadFromUrl(`/osm/${example}.pbf`);
      } else {
        await loadFromUrl(
          `https://assets.od2net.org/severance_pbfs/${example}.pbf`,
        );
      }
    }
  }
  $: loadExample(example);

  async function loadFromUrl(url: string) {
    try {
      loading = [`Downloading ${url}`];
      let resp = await fetch(url);
      await loadModel(await resp.arrayBuffer());
    } catch (err) {
      window.alert(`Couldn't open from URL ${url}: ${err}`);
    }
    loading = [];
  }
</script>

<Loading {loading} />

<div>
  <label>
    Load an example:
    <select bind:value={example}>
      <option value="">Custom file loaded</option>
      {#each exampleAreas as [country, areas]}
        <optgroup label={country}>
          {#each areas as [value, label]}
            <option {value}>{label}</option>
          {/each}
        </optgroup>
      {/each}
    </select>
  </label>
</div>

<i>or...</i>

<div>
  <label>
    Load an osm.xml or a .pbf file:
    <input bind:this={osmFileInput} on:change={loadOsmFile} type="file" />
  </label>
</div>

<i>or...</i>

<OverpassSelector
  map={$map}
  on:gotXml={gotXml}
  on:loading={(e) => (loading = [...loading, e.detail])}
  on:error={(e) => window.alert(e.detail)}
/>

<i>or...</i>

<div>
  <label>
    Load a pre-built model.bin file:
    <input bind:this={modelFileInput} on:change={loadModelFile} type="file" />
  </label>
</div>
