<script lang="ts">
  import { CircleLayer, hoverStateFilter } from "svelte-maplibre";
  import { notNull, Modal } from "svelte-utils";
  import { Popup } from "svelte-utils/map";

  let arrivals: [string, string][] | null = null;
</script>

<CircleLayer
  id="stops"
  paint={{
    "circle-radius": 5,
    "circle-color": hoverStateFilter("cyan", "blue"),
  }}
  manageHoverState
  filter={["has", "arrivals"]}
  on:click={(e) =>
    (arrivals = JSON.parse(notNull(e.detail.features[0].properties).arrivals))}
  hoverCursor="pointer"
  eventsIfTopMost
>
  <Popup openOn="hover" let:props
    >{props.name} has {JSON.parse(props.arrivals).length} arrivals</Popup
  >
</CircleLayer>

{#if arrivals}
  <Modal on:close={() => (arrivals = null)}>
    {#each arrivals as x}
      <p>{JSON.stringify(x)}</p>
    {/each}
  </Modal>
{/if}
