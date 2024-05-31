<script lang="ts">
  import { notNull, Modal } from "svelte-utils";
  import { showAbout } from "./stores";
</script>

{#if $showAbout}
  <Modal on:close={() => ($showAbout = false)} let:dialog>
    <h1>15-minute neighborhood tool</h1>
    <p>TODO. Extremely early in development.</p>
    <p>
      This <a href="https://github.com/a-b-street/15m/" target="_blank"
        >open source</a
      >
      tool is created by
      <a href="https://github.com/dabreegster/" target="_blank"
        >Dustin Carlino</a
      >
      and relies heavily on
      <a href="https://www.openstreetmap.org/about" target="_blank"
        >OpenStreetMap</a
      > data.
    </p>

    <center
      ><button on:click={() => notNull(dialog).close()}>Start!</button></center
    >
  </Modal>
{/if}
