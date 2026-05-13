<script lang="ts">
  import ComingSoon from '$lib/components/ComingSoon.svelte';
  import { Globe } from '@lucide/svelte';
</script>

<ComingSoon
  moduleId="M9 / M12"
  crumb="Territoire & géolocalisation"
  title="Cartographie IRIS de l'impact <em>territorial</em>."
  subtitle="Croisement ComparIA (5 GB, conversations LLMs) × RTE/NaTran/Teréga IRIS
(consommation industrielle France, ~50 000 mailles) → choroplèthe interactive, détection
des IRIS candidats à héberger un datacenter, scénarios régionaux 5 ans, comparaison
benchmarks internationaux."
  icon={Globe}
  chantier="C12 — Tier 1 défi data.gouv.fr"
  efs="EF-M9-01 → EF-M9-05 + EF-M12-01 → EF-M12-07"
  pendingIpcs={[
    {
      name: 'list_iris(bbox)',
      description: 'Renvoie les IRIS dans une bounding box (geojson polygones + métadonnées INSEE).'
    },
    {
      name: 'load_iris_consumption(year)',
      description:
        'Charge la couche Gold consommation industrielle élec + gaz par IRIS (source RTE/NaTran/Teréga via ODRÉ).'
    },
    {
      name: 'detect_datacenters_candidates(threshold_mwh, ratio_elec_gaz)',
      description:
        'Heuristique IRIS avec consommation électrique disproportionnée par rapport au gaz — candidats DC.'
    },
    {
      name: 'project_region_scenario(region, years, model_mix)',
      description: "Projection régionale d'impact LLM sur N années pour une région administrative."
    },
    {
      name: 'detect_user_zone()',
      description:
        'Géolocalisation IP locale (offline, sans appel externe) → région INSEE + mix élec local.'
    }
  ]}
/>
