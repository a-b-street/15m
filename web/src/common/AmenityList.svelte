<script lang="ts">
  import type { Feature, Point, FeatureCollection } from "geojson";
  import { type Amenity, describeAmenity } from "../stores";

  // Can contain things besides amenities
  export let gj: FeatureCollection;

  $: amenityFeatures = gj.features.filter(
    (f) => "amenity_kind" in f.properties!,
  ) as Feature<Point, Amenity>[];

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
</script>

{#each groupByKind(amenityFeatures) as [kind, list]}
  <details>
    <summary>{kind} ({list.length})</summary>
    <ol>
      {#each list as f}
        <li>
          <a href={f.properties.osm_id} target="_blank">{describeAmenity(f)}</a>
        </li>
      {/each}
    </ol>
  </details>
{/each}
