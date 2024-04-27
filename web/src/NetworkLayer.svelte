<script lang="ts">
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { notNull } from "./common";
  import { model } from "./stores";
</script>

<GeoJSON data={JSON.parse(notNull($model).render())}>
  <LineLayer
    id="network"
    paint={{
      "line-width": 5,
      "line-color": "black",
    }}
  />
</GeoJSON>
