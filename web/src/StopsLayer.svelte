<script lang="ts">
  import { CircleLayer, hoverStateFilter } from "svelte-maplibre";
  import { notNull, Modal } from "svelte-utils";
  import { Popup } from "svelte-utils/map";

  let next_steps: [any][] | null = null;
</script>

<CircleLayer
  id="stops"
  paint={{
    "circle-radius": 5,
    "circle-color": hoverStateFilter("cyan", "blue"),
  }}
  manageHoverState
  filter={["has", "next_steps"]}
  on:click={(e) =>
    (next_steps = JSON.parse(
      notNull(e.detail.features[0].properties).next_steps,
    ))}
  hoverCursor="pointer"
  eventsIfTopMost
>
  <Popup openOn="hover" let:props
    >{props.name} has {JSON.parse(props.next_steps).length} next steps (arrivals)</Popup
  >
</CircleLayer>

{#if next_steps}
  <Modal on:close={() => (next_steps = null)}>
    {#each next_steps as x}
      <p>{JSON.stringify(x)}</p>
    {/each}
  </Modal>
{/if}
