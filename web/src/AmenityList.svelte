<script lang="ts">
  import type { Feature, Point, FeatureCollection } from "geojson";

  // Can contain things besides amenities
  export let gj: FeatureCollection<Point, Amenity>;

  interface Amenity {
    amenity_kind: string;
    osm_id: string;
    name?: string;
    brand?: string;
    cuisine?: string;
  }

  $: amenityFeatures = gj.features.filter(
    (f) => "amenity_kind" in f.properties,
  );

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

  function describe(f: Feature<Point, Amenity>): string {
    let label = f.properties.name || `a ${f.properties.amenity_kind}`;
    if (f.properties.brand) {
      label += ` (${f.properties.brand})`;
    }
    if (f.properties.cuisine) {
      label += ` (${f.properties.cuisine})`;
    }
    return label;
  }
</script>

{#each groupByKind(amenityFeatures) as [kind, list]}
  <details>
    <summary>{kind} ({list.length})</summary>
    <ol>
      {#each list as f}
        <li><a href={f.properties.osm_id} target="_blank">{describe(f)}</a></li>
      {/each}
    </ol>
  </details>
{/each}
