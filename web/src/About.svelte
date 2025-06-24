<script lang="ts">
  import { Modal, notNull } from "svelte-utils";
  import { showAbout } from "./stores";
</script>

{#if $showAbout}
  <Modal on:close={() => ($showAbout = false)} let:dialog>
    <h1>15-minute neighborhood tool</h1>
    <p>
      This is an <b>experimental</b> tool to explore how transportation and land-use
      interacts. From a start point, how far can somebody reach in 15 minutes by
      walking, cycling, or with public transit? Within that area, what points of
      interest are around?
    </p>
    <p>
      This is a newer version of an <a
        href="https://a-b-street.github.io/docs/software/fifteen_min.html"
        target="_blank">older tool</a
      >. It's still under development. If you have any questions or ideas about
      it, please
      <a href="https://github.com/a-b-street/15m/issues/new" target="_blank"
        >file an issue</a
      >
      or email
      <a href="mailto:dabreegster@gmail.com">dabreeegster@gmail.com</a>.
    </p>
    <p>
      This tool contains data from
      <a href="https://www.openstreetmap.org/about" target="_blank"
        >OpenStreetMap</a
      >, the UK
      <a href="https://www.bus-data.dft.gov.uk/" target="_blank"
        >Bus Open Data Service</a
      >, and census data from several countries using
      <a
        href="https://github.com/Urban-Analytics-Technology-Platform/popgetter"
        target="_blank">popgetter</a
      >.
    </p>
    <p>
      This <a href="https://github.com/a-b-street/15m/" target="_blank"
        >open source</a
      >
      tool is created by
      <a href="https://abstreet.uk" target="_blank"
        >Dustin Carlino</a
      >.
    </p>

    <center
      ><button on:click={() => notNull(dialog).close()}>Start!</button></center
    >
  </Modal>
{/if}
