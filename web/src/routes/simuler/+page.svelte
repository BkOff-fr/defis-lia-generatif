<script lang="ts">
  // Module M13 — Simulateur « Et si...? » (CDC §4.1).
  // L'UI est encore un stub (ComingSoon) — le moteur Rust pour les 7
  // leviers temps réel arrivera au chantier C11. La garde de route ci-dessous
  // est posée dès C10 (cf. ADR-0010 + brief C10 §3.5).
  import ComingSoon from '$lib/components/ComingSoon.svelte';
  import { TrendingUp } from '@lucide/svelte';
  import { preferences, type ModuleId } from '$lib/preferences';

  const MODULE_ID: ModuleId = 'm13';

  $effect(() => {
    if ($preferences.loaded && !$preferences.enabled_modules.includes(MODULE_ID)) {
      // `window.location.replace` plutôt que `goto`/`$app/navigation`
      // (cf. note dans +layout.svelte).
      window.location.replace('/?disabled=' + MODULE_ID);
    }
  });
</script>

<ComingSoon
  moduleId="M13"
  crumb="Simuler"
  title="Projetez l'impact d'un scénario avec <em>7 leviers</em> temps réel."
  subtitle="Construisez un scénario macro (population, taux d'adoption, fréquence, mix
modèles, datacenter, batch size, contexte) et obtenez la trajectoire CO₂eq · énergie ·
eau année par année avec son intervalle d'incertitude."
  icon={TrendingUp}
  chantier="C11 — moteur simulateur"
  efs="EF-M13-01 → EF-M13-07"
  pendingIpcs={[
    {
      name: 'run_scenario(req)',
      description:
        "Lance une projection Monte-Carlo sur N années à partir d'un ScenarioRequestDto (population, taux, modèles, période, leviers)."
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
