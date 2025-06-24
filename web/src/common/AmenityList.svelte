<script lang="ts">
  import type { Feature, FeatureCollection, Point } from "geojson";
  import { describeAmenity, hideAmenityKinds, type Amenity } from "../stores";

  // Can contain things besides amenities
  export let gj: FeatureCollection;

  $: amenityFeatures = gj.features.filter(
    (f) => "amenity_kind" in f.properties!,
  ) as Feature<Point, Amenity>[];
  $: kinds = groupByKind(amenityFeatures);

  // Sorted by number of members
  function groupByKind(
    features: Feature<Point, Amenity>[],
  ): [string, Feature<Point, Amenity>[]][] {
    let map = new Map();
    for (let f of features) {
      if (!map.has(f.properties.amenity_kind)) {
        map.set(f.properties.amenity_kind, []);
      }
      map.get(f.properties.amenity_kind).push(f);
    }

    let list = [...map.entries()];
    list.sort((a, b) => b[1].length - a[1].length);
    return list;
  }

  $: updateKinds(kinds);
  function updateKinds(kinds: [string, any][]) {
    for (let [kind, _] of kinds) {
      if (!$hideAmenityKinds.has(kind)) {
        $hideAmenityKinds.set(kind, false);
      }
    }
    $hideAmenityKinds = $hideAmenityKinds;
  }
</script>

{#each kinds as [kind, list]}
  <details>
    <summary
      ><input
        type="checkbox"
        checked={!$hideAmenityKinds.get(kind)}
        on:change={() => {
          $hideAmenityKinds.set(kind, !$hideAmenityKinds.get(kind));
          $hideAmenityKinds = $hideAmenityKinds;
        }}
      />
      {kind} ({list.length})</summary
    >
    <ol>
      {#each list as f}
        <li>
          <a href={f.properties.osm_id} target="_blank">{describeAmenity(f)}</a>
        </li>
      {/each}
    </ol>
  </details>
{/each}
