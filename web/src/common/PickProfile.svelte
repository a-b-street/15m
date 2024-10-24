<script lang="ts">
  import type { Profile } from "../stores";

  export let profile: Profile;
</script>

<label
  >Profile:
  <select bind:value={profile}>
    <option value="car">Car</option>
    <option value="bicycle">Bicycle</option>
    <option value="foot">Foot</option>
    <option value="transit">Public transit</option>
  </select>
</label>
