<script lang="ts">
  import { onMount } from "svelte";
  import { Modal } from "svelte-utils";
  import { backend } from "../stores";

  let kinds: Map<string, { enabled: boolean; num: number }> = new Map();
  let show = false;

  export let enabled: string[];
  $: enabled = getEnabled(kinds);

  onMount(async () => {
    let gj = await $backend!.renderAmenities();
    for (let f of gj.features) {
      let kind = f.properties.amenity_kind;
      if (kinds.has(kind)) {
        kinds.get(kind)!.num += 1;
      } else {
        kinds.set(kind, { enabled: false, num: 1 });
      }
    }

    kinds = sortMap(kinds, (x) => x.num);
  });

  // Descending
  function sortMap<K, V>(map: Map<K, V>, cmp: (value: V) => number) {
    let pairs: [K, V][] = [...map.entries()];
    pairs.sort((a, b) => cmp(b[1]) - cmp(a[1]));

    let result = new Map();
    for (let [k, v] of pairs) {
      result.set(k, v);
    }
    return result;
  }

  function getEnabled(_x: any): string[] {
    return [...kinds.entries()]
      .filter((pair) => pair[1].enabled)
      .map((pair) => pair[0]);
  }
</script>

<Modal bind:show>
  <h2>Pick types of amenity to search from</h2>

  <fieldset>
    <legend>Amenities:</legend>

    {#each kinds.entries() as [key, value]}
      <label>
        <input type="checkbox" bind:checked={value.enabled} />
        {key} ({value.num})
      </label>
    {/each}
  </fieldset>

  <center><button on:click={() => (show = false)}>Start!</button></center>
</Modal>

<p>Amenities: {enabled.join(", ")}</p>
<button on:click={() => (show = true)}>Choose</button>
