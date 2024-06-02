<script lang="ts">
  import { backend } from "../stores";
  import { onMount } from "svelte";
  import { Modal, notNull } from "svelte-utils";

  let kinds: Map<string, { enabled: boolean; num: number }> = new Map();
  let show = false;

  export let enabled: string[];
  $: enabled = getEnabled(kinds);

  onMount(async () => {
    let gj = await $backend!.renderDebug();
    for (let f of gj.features) {
      let kind: string | undefined = f.properties!.amenity_kind;
      if (kind) {
        if (kinds.has(kind)) {
          kinds.get(kind)!.num += 1;
        } else {
          kinds.set(kind, { enabled: false, num: 1 });
        }
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

{#if show}
  <Modal on:close={() => (show = false)} let:dialog>
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

    <center
      ><button on:click={() => notNull(dialog).close()}>Start!</button></center
    >
  </Modal>
{:else}
  <p>Amenities: {enabled.join(", ")}</p>
  <button on:click={() => (show = true)}>Choose</button>
{/if}
