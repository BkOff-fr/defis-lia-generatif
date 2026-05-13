<script lang="ts">
  import ComingSoon from '$lib/components/ComingSoon.svelte';
  import { TrendingUp } from '@lucide/svelte';
</script>

<ComingSoon
  moduleId="M4"
  crumb="Simuler"
  title="Projetez l'impact d'un usage sur <em>5 ans</em>."
  subtitle="Construisez un scénario macro (population, taux d'adoption, fréquence, mix
modèles, datacenter) et obtenez la trajectoire CO₂eq · énergie · eau année par année
avec son intervalle d'incertitude."
  icon={TrendingUp}
  chantier="C10 — outillage moteur"
  efs="EF-M4-01 → EF-M4-06"
  pendingIpcs={[
    {
      name: 'run_scenario(req)',
      description:
        "Lance une projection temporelle Monte-Carlo sur N années à partir d'un ScenarioRequestDto (population, taux, modèles, période)."
    },
    {
      name: 'save_scenario(scenario)',
      description: 'Persiste un scénario en JSON dans le data root utilisateur.'
    },
    {
      name: 'list_scenarios()',
      description: 'Liste les scénarios sauvegardés pour reprise / comparaison.'
    }
  ]}
/>
