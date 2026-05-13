<script lang="ts">
  import ComingSoon from '$lib/components/ComingSoon.svelte';
  import { Download } from '@lucide/svelte';
</script>

<ComingSoon
  moduleId="M6"
  crumb="Rapports & exports"
  title="Générez un rapport <em>sourcé et auditable</em>."
  subtitle="PDF synthétique 1 page (grand public), rapport détaillé 4-8 pages
(direction), rapport CSRD-ready 12 pages (entreprises), notebook Quarto
reproductible, JSON-LD pour audit réglementaire, Observable Notebook embeddable."
  icon={Download}
  chantier="C10 — outillage"
  efs="EF-M6-01 → EF-M6-05"
  pendingIpcs={[
    {
      name: 'render_report_pdf(template, payload, path)',
      description:
        "Compose un PDF (template Quarto) à partir d'un payload d'estimation, scénario ou comparaison. Templates : synth 1p, détaillé 4-8p, CSRD 12p."
    },
    {
      name: 'render_notebook_qmd(scenario, path)',
      description:
        'Génère un notebook Quarto autonome (sources + calculs + plots) qui peut être rendu offline en HTML / PDF.'
    },
    {
      name: 'export_dataset_parquet(filter, path)',
      description:
        'Matérialise un Parquet enrichi de datasheet (Gebru et al.) — filtre par modèle / période / utilisateur.'
    },
    {
      name: 'export_jsonld_csrd(payload, path)',
      description:
        'Export JSON-LD compatible audit CSRD (catégories scope 1/2/3 mappées sur le GHG Protocol).'
    }
  ]}
/>
