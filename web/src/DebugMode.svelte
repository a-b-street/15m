<script lang="ts">
  import { GeoJSON, hoverStateFilter, LineLayer } from "svelte-maplibre";
  import { notNull } from "./common";
  import SplitComponent from "./SplitComponent.svelte";
  import { PropertiesTable, Popup } from "svelte-utils";
  import { mode, model } from "./stores";
</script>

<SplitComponent>
  <div slot="sidebar">
    <h2>Debug mode</h2>
    <div>
      <button on:click={() => ($mode = "title")}>Change study area</button>
      <button on:click={() => ($mode = "isochrone")}>Isochrones</button>
    </div>
    <p>Hover to see a segment's properties, and click to open OSM</p>
  </div>
  <div slot="map">
    <GeoJSON data={JSON.parse(notNull($model).render())} generateId>
      <LineLayer
        id="network"
        paint={{
          "line-width": hoverStateFilter(5, 7),
          "line-color": "black",
        }}
        manageHoverState
        on:click={(e) =>
          window.open(notNull(e.detail.features[0].properties).way, "_blank")}
        hoverCursor="pointer"
      >
        <Popup openOn="hover" let:props>
          <PropertiesTable properties={props} />
        </Popup>
      </LineLayer>
    </GeoJSON>
  </div>
</SplitComponent>
