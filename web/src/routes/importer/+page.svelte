<script lang="ts">
  import ComingSoon from '$lib/components/ComingSoon.svelte';
  import { Upload } from '@lucide/svelte';
</script>

<ComingSoon
  moduleId="M10"
  crumb="Importer un journal d'usage"
  title="Importez un journal d'usage entreprise — <em>anonymisé localement</em>."
  subtitle="CSV / JSONL / Parquet (exports OpenAI, Anthropic, Mistral, génériques) →
mappage interactif des colonnes → estimation par lot → rapport RSE prêt pour CSRD.
Aucun envoi externe, tout reste sur la machine."
  icon={Upload}
  chantier="C11 — entreprise / RSE"
  efs="EF-M10-01 → EF-M10-05"
  pendingIpcs={[
    {
      name: 'parse_usage_log(path, format)',
      description:
        'Détecte le schéma, anonymise les colonnes user_hash, normalise les timestamps. Renvoie un échantillon + métadonnées.'
    },
    {
      name: 'apply_mapping(path, mapping)',
      description:
        'Applique le mappage interactif des colonnes (Sobr.ia ← votre fichier) et matérialise un Parquet normalisé sous data/silver/imports/.'
    },
    {
      name: 'estimate_batch(rows)',
      description:
        'Estime N requêtes en parallèle via le moteur Monte-Carlo, agrège par utilisateur / équipe / modèle / période.'
    },
    {
      name: 'export_rse_report(scenario, path)',
      description: 'Génère un PDF CSRD-ready 12 pages (livrable L4 du CDC).'
    }
  ]}
/>
