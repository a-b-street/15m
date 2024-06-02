<script lang="ts">
  import { backend } from "../stores";
  import { onMount } from "svelte";
  import { Modal, notNull } from "svelte-utils";

  let kinds: { [name: string]: boolean } = {};
  let show = false;

  export let enabled: string[];
  $: enabled = getEnabled(kinds);

  onMount(async () => {
    let gj = await $backend!.renderDebug();
    let allKinds: Set<string> = new Set();
    for (let f of gj.features) {
      let kind: string | undefined = f.properties!.amenity_kind;
      if (kind) {
        allKinds.add(kind);
      }
    }

    // Make the order work
    for (let kind of [...allKinds].sort()) {
      kinds[kind] = false;
    }
    kinds = kinds;
  });

  function getEnabled(kinds: { [name: string]: boolean }): string[] {
    return Object.entries(kinds)
      .filter((pair) => pair[1])
      .map((pair) => pair[0]);
  }
</script>

{#if show}
  <Modal on:close={() => (show = false)} let:dialog>
    <h2>Pick types of amenity to search from</h2>

    <fieldset>
      <legend>Amenities:</legend>

      {#each Object.keys(kinds) as key}
        <label>
          <input type="checkbox" bind:checked={kinds[key]} />
          {key}
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
