<script lang="ts">
  import type { TravelMode } from "../stores";

  export let travelMode: TravelMode;
</script>

<label
  >Mode:
  <select bind:value={travelMode}>
    <option value="car">Car</option>
    <option value="bicycle">Bicycle</option>
    <option value="foot">Foot</option>
    <option value="transit">Public transit</option>
  </select>
</label>
