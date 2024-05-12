<script lang="ts">
  import * as Comlink from "comlink";
  import { onMount } from "svelte";
  import { OverpassSelector } from "svelte-utils";
  import { map, backend, isLoaded } from "../stores";
  import Loading from "./Loading.svelte";

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

  let fileInput: HTMLInputElement;
  async function loadFile(e: Event) {
    try {
      await loadModel(await fileInput.files![0].arrayBuffer());
      example = "";
    } catch (err) {
      window.alert(`Couldn't open this file: ${err}`);
    }
    loading = [];
  }

  async function loadModel(buffer: ArrayBuffer) {
    loading = ["Building map model from OSM input"];
    console.time("load");
    await $backend!.loadFile(new Uint8Array(buffer), Comlink.proxy(progressCb));
    console.timeEnd("load");
    $isLoaded = true;
  }

  function progressCb(msg: string) {
    loading = [...loading, msg];
  }

  async function gotXml(e: CustomEvent<string>) {
    try {
      // TODO Can we avoid turning into bytes?
      await loadModel(new TextEncoder().encode(e.detail));
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
    <input bind:this={fileInput} on:change={loadFile} type="file" />
  </label>
</div>

<i>or...</i>

<OverpassSelector
  map={$map}
  on:gotXml={gotXml}
  on:loading={(e) => (loading = [...loading, e.detail])}
  on:error={(e) => window.alert(e.detail)}
/>
