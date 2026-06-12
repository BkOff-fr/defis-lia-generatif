<script lang="ts">
  import {
    BookOpen,
    Sigma,
    ShieldCheck,
    FlaskConical,
    Library,
    Quote,
    Info,
    ArrowUpRight,
    HelpCircle,
    Lock
  } from '@lucide/svelte';

  // Sections (sommaire latéral / TOC) — `anchor` doit matcher l'`id`
  // posé sur le `<section>` cible pour le scroll natif `:target`.
  const sections = [
    { anchor: 'methode', label: 'Méthode', icon: Sigma },
    { anchor: 'validation', label: 'Validation', icon: ShieldCheck },
    { anchor: 'glossaire', label: 'Glossaire', icon: BookOpen },
    { anchor: 'references', label: 'Références', icon: Library },
    { anchor: 'biblio', label: 'Bibliographie', icon: Quote },
    { anchor: 'apropos', label: 'À propos', icon: Info }
  ] as const;

  type GlossaryEntry = {
    fr: string;
    en: string;
    def: string;
    source?: string;
  };

  // Sous-ensemble du glossaire canonique (docs/methodology/GLOSSAIRE.md).
  // À enrichir au fil de S0 (cible ≥ 40 termes). Quand on aura un import
  // markdown côté front, on tirera la table directement du fichier.
  const glossary: GlossaryEntry[] = [
    {
      fr: 'Amortissement (embodied)',
      en: 'Embodied amortization',
      def: 'Répartition de l’impact de fabrication du hardware sur sa durée d’usage et son volume de requêtes.',
      source: 'Gupta et al. 2022'
    },
    {
      fr: 'CO₂ équivalent (CO₂eq)',
      en: 'Carbon dioxide equivalent',
      def: 'Métrique unifiée des gaz à effet de serre, pondérés par leur GWP100.',
      source: 'GIEC'
    },
    {
      fr: 'CSRD',
      en: 'Corporate Sustainability Reporting Directive',
      def: 'Directive UE imposant la publication de données extra-financières pour les grandes entreprises.',
      source: 'UE 2022/2464'
    },
    {
      fr: 'EcoLogits',
      en: 'EcoLogits',
      def: 'Bibliothèque Python d’estimation d’impact des requêtes LLMs, méthodologie officielle ComparIA.',
      source: 'Data for Good'
    },
    {
      fr: 'Embodied carbon',
      en: 'Embodied carbon',
      def: 'Émissions liées à la fabrication, transport et fin de vie du matériel.',
      source: 'ITU-T L.1410'
    },
    {
      fr: 'Facteur d’émission (IF)',
      en: 'Emission Factor',
      def: 'Quantité de gCO₂eq par unité d’énergie consommée (gCO₂eq/kWh).',
      source: 'ADEME Base Empreinte'
    },
    {
      fr: 'Frugalité numérique',
      en: 'Digital frugality',
      def: 'Démarche de conception minimisant la consommation de ressources des systèmes numériques.',
      source: 'AFNOR SPEC 2314'
    },
    {
      fr: 'IRIS',
      en: 'IRIS (statistical unit)',
      def: 'Plus petite unité géographique INSEE en France, ~2 000 habitants.',
      source: 'INSEE'
    },
    {
      fr: 'LCA (ACV)',
      en: 'Life Cycle Assessment',
      def: 'Analyse environnementale couvrant le cycle de vie complet d’un produit ou service.',
      source: 'ISO 14040/44'
    },
    {
      fr: 'Médaillon (architecture)',
      en: 'Medallion architecture',
      def: 'Pattern de pipeline data en 3 couches Copper/Silver/Gold.',
      source: 'Databricks 2020'
    },
    {
      fr: 'Monte-Carlo',
      en: 'Monte Carlo simulation',
      def: 'Méthode statistique de propagation d’incertitude par échantillonnage.'
    },
    {
      fr: 'PUE',
      en: 'Power Usage Effectiveness',
      def: 'Ratio énergie totale datacenter / énergie IT, mesure d’efficacité énergétique.',
      source: 'The Green Grid'
    },
    {
      fr: 'Scope 1/2/3',
      en: 'Scope 1/2/3',
      def: 'Périmètres d’émissions définis par le GHG Protocol.',
      source: 'GHG Protocol'
    },
    {
      fr: 'Token',
      en: 'Token',
      def: 'Unité de découpage de texte utilisée par les LLMs (≈ 3,3 caractères en moyenne en FR, ~4 en EN).'
    },
    {
      fr: 'WUE',
      en: 'Water Usage Effectiveness',
      def: 'Litres d’eau consommés par kWh IT d’un datacenter.',
      source: 'The Green Grid'
    }
  ];

  type Reference = {
    title: string;
    org: string;
    year?: string;
    url: string;
  };

  const references: Reference[] = [
    {
      title: 'AFNOR SPEC 2314 — Référentiel général pour l’IA frugale',
      org: 'Ecolab / CGDD',
      year: '2024',
      url: 'https://normalisation.afnor.org/actualites/intelligence-artificielle-publication-de-la-spec-afnor-2314/'
    },
    {
      title: 'ISO/IEC 21031:2024 — Méthodologie environnementale ICT',
      org: 'ISO/IEC',
      year: '2024',
      url: 'https://www.iso.org/standard/89947.html'
    },
    {
      title: 'ITU-T L.1410 — LCA pour les TIC',
      org: 'UIT',
      year: '2014',
      url: 'https://www.itu.int/rec/T-REC-L.1410-201412-I'
    },
    {
      title: 'GHG Protocol — Scope 3 Standard',
      org: 'WBCSD / WRI',
      url: 'https://ghgprotocol.org/standards/scope-3-standard'
    },
    {
      title: 'Base Empreinte — facteurs d’émission',
      org: 'ADEME',
      url: 'https://base-empreinte.ademe.fr'
    }
  ];

  const biblio = [
    {
      label: 'Luccioni, A. S., et al. (2023) — Estimating the Carbon Footprint of BLOOM.',
      hint: 'arXiv:2211.02001'
    },
    {
      label: 'Patterson, D., et al. (2021) — Carbon Emissions and Large Neural Network Training.',
      hint: 'arXiv:2104.10350'
    },
    {
      label: 'EcoLogits Team (2024) — Methodology v1.4.',
      hint: 'genai-impact.org'
    },
    {
      label: 'Gupta, U., et al. (2022) — Chasing Carbon: Embodied Footprint of Computing.',
      hint: 'IEEE Micro'
    },
    {
      label: 'Ren, S., et al. (2023) — Making AI Less Thirsty.',
      hint: 'arXiv:2304.03271'
    }
  ];
</script>

<svelte:head>
  <title>Sobr.ia · Méthodologie</title>
</svelte:head>

<div class="canvas-inner">
  <!-- TopBar -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Atelier <span class="sep">/</span>
      <span class="current">Méthodologie</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill" title="Documentation servie depuis le binaire local">
      <Lock size={12} strokeWidth={1.8} />
      Doc 100 % locale
    </span>
    <a class="icon-btn" href="/" aria-label="Retour à l'atelier">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- Hero -->
  <section class="hero">
    <div class="hero-eyebrow">
      <span class="pulse" aria-hidden="true"></span>
      Méthodologie · alignement AFNOR SPEC 2314
    </div>
    <h1 class="hero-h1">
      Une méthode <em>sourcée</em>, des hypothèses lisibles, des intervalles assumés.
    </h1>
    <p class="hero-sub">
      Sobr.ia mesure l'empreinte des LLMs par Monte-Carlo sur 10 000 tirages à partir de paramètres
      publiés ou extrapolés, en suivant le référentiel français AFNOR SPEC 2314 et les facteurs
      d'émission ADEME. Toute valeur est restituée avec son intervalle P5–P95 et ses sources
      cliquables.
    </p>
    <aside class="page-crosslink" aria-label="Choisir une méthodologie">
      <strong>Cette page documente l'approche AFNOR SPEC 2314 (Sobr.ia).</strong>
      Pour <em>choisir</em> quelle méthodologie scientifique vous utilisez au quotidien (AFNOR
      Sobr.ia, EcoLogits 2026-01, etc.) :
      <a class="crosslink-cta" href="/methodologies">
        → Catalogue des méthodologies <ArrowUpRight size={12} strokeWidth={2} />
      </a>
    </aside>
  </section>

  <div class="page-grid">
    <!-- TOC latérale -->
    <aside class="toc" aria-label="Sommaire">
      {#each sections as s (s.anchor)}
        {@const Icon = s.icon}
        <a class="toc-item" href={`#${s.anchor}`}>
          <Icon size={14} strokeWidth={1.8} />
          {s.label}
        </a>
      {/each}
    </aside>

    <div class="page-main">
      <!-- ─── Méthode ─────────────────────────────────────── -->
      <section id="methode" class="card">
        <header class="card-head">
          <Sigma size={18} strokeWidth={1.6} />
          <h2>Formule de référence</h2>
        </header>
        <p>
          Pour un prompt unitaire (T<sub>in</sub> tokens en entrée, T<sub>out</sub>
          tokens estimés en sortie), Sobr.ia calcule l'empreinte CO₂eq comme la somme du coût opérationnel
          et de l'embodied amorti :
        </p>
        <pre class="formula"><code
            >CO₂eq = [ E_compute × PUE × IF_électrique
        + E_embodied / N_amortissement ]
       avec  E_compute = (T_in × ε_prefill + T_out × ε_decode) × η_modèle</code
          ></pre>
        <ul class="formula-legend">
          <li>
            <span class="sym">ε_prefill</span><span class="sep">·</span>
            énergie de lecture du prompt, ≈ 0,4 × ε_decode (batché GPU).
          </li>
          <li>
            <span class="sym">ε_decode</span><span class="sep">·</span>
            énergie de génération autorégressive par token, ≈ k_decode × N<sub>params</sub>
            (HF AI Energy Score 2026).
          </li>
          <li>
            <span class="sym">PUE</span><span class="sep">·</span>
            Power Usage Effectiveness du datacenter (1,1–1,5 selon zone).
          </li>
          <li>
            <span class="sym">IF</span><span class="sep">·</span>
            facteur d'émission électrique en gCO₂eq/kWh (RTE/ADEME).
          </li>
          <li>
            <span class="sym">embodied</span><span class="sep">·</span>
            empreinte fabrication GPU amortie sur ~10⁹ requêtes (Gupta 2022).
          </li>
        </ul>
        <p class="card-foot">
          Détail complet et code source dans
          <a
            href="https://github.com/BkOff-fr/defis-lia-generatif/blob/main/crates/sobria-core/src/estimation.rs"
            target="_blank"
            rel="noopener noreferrer"
          >
            <code>sobria-core::estimation</code><ArrowUpRight size={12} strokeWidth={2} />
          </a>.
        </p>
      </section>

      <!-- ─── Validation ─────────────────────────────────── -->
      <section id="validation" class="card">
        <header class="card-head">
          <ShieldCheck size={18} strokeWidth={1.6} />
          <h2>Propagation d'incertitude &amp; validation</h2>
        </header>
        <p>
          Chaque paramètre est une <strong>distribution</strong> (uniforme, normale ou log-normale
          selon nature), pas une valeur scalaire. Le moteur tire 10 000 trajectoires Monte-Carlo et
          restitue les percentiles P5, P50, P95 — c'est l'intervalle que vous voyez sur l'écran
          <a href="/">Estimer</a>.
        </p>
        <div class="status-grid">
          <div class="status">
            <FlaskConical size={14} strokeWidth={1.8} />
            <div>
              <div class="status-label">Plausibilité</div>
              <div class="status-val">CI permanente</div>
              <div class="status-note">P50 ∈ plage attendue, 10²–10³ d'écart toléré.</div>
            </div>
          </div>
          <div class="status">
            <ShieldCheck size={14} strokeWidth={1.8} />
            <div>
              <div class="status-label">Reproduction</div>
              <div class="status-val">±20–25 % usage-only</div>
              <div class="status-note">
                3 cas Llama 70B / Mistral Large 2 contre EcoLogits 2026-01
                (DOI:10.21105/joss.07471). Embodied comparé séparément (méthodologies divergentes
                par construction).
              </div>
            </div>
          </div>
          <div class="status">
            <BookOpen size={14} strokeWidth={1.8} />
            <div>
              <div class="status-label">Relecture</div>
              <div class="status-val">mentor Ecolab/ADEME</div>
              <div class="status-note">Revue méthodologie + valeurs critiques avant tag v1.0.</div>
            </div>
          </div>
        </div>
      </section>

      <!-- ─── Glossaire ────────────────────────────────── -->
      <section id="glossaire" class="card">
        <header class="card-head">
          <BookOpen size={18} strokeWidth={1.6} />
          <h2>Glossaire FR&nbsp;/&nbsp;EN</h2>
          <span class="card-meta">{glossary.length} termes</span>
        </header>
        <table class="glossary">
          <thead>
            <tr>
              <th>FR</th>
              <th>EN</th>
              <th>Définition</th>
              <th>Source</th>
            </tr>
          </thead>
          <tbody>
            {#each glossary as g (g.fr)}
              <tr>
                <td class="gl-fr">{g.fr}</td>
                <td class="gl-en">{g.en}</td>
                <td class="gl-def">{g.def}</td>
                <td class="gl-src">{g.source ?? '—'}</td>
              </tr>
            {/each}
          </tbody>
        </table>
        <p class="card-foot">
          Source canonique :
          <a
            href="https://github.com/BkOff-fr/defis-lia-generatif/blob/main/docs/methodology/GLOSSAIRE.md"
            target="_blank"
            rel="noopener noreferrer"
          >
            <code>docs/methodology/GLOSSAIRE.md</code><ArrowUpRight size={12} strokeWidth={2} />
          </a>
          (à enrichir en S0, cible ≥ 40 termes).
        </p>
      </section>

      <!-- ─── Références normatives ─────────────────────── -->
      <section id="references" class="card">
        <header class="card-head">
          <Library size={18} strokeWidth={1.6} />
          <h2>Références normatives</h2>
        </header>
        <ul class="ref-list">
          {#each references as r (r.title)}
            <li>
              <a href={r.url} target="_blank" rel="noopener noreferrer">
                <span class="ref-title">{r.title}</span>
                <span class="ref-meta">{r.org}{r.year ? ` · ${r.year}` : ''}</span>
                <ArrowUpRight size={12} strokeWidth={2} />
              </a>
            </li>
          {/each}
        </ul>
      </section>

      <!-- ─── Bibliographie ─────────────────────────────── -->
      <section id="biblio" class="card">
        <header class="card-head">
          <Quote size={18} strokeWidth={1.6} />
          <h2>Bibliographie sélective</h2>
        </header>
        <ul class="biblio-list">
          {#each biblio as b (b.label)}
            <li>
              <span class="biblio-label">{b.label}</span>
              <span class="biblio-hint mono">{b.hint}</span>
            </li>
          {/each}
        </ul>
        <p class="card-foot">
          La bibliographie complète (≥ 30 entrées en cible) sera publiée au tag v1.0 avec le rapport
          méthodologique PDF (livrable L5).
        </p>
      </section>

      <!-- ─── À propos ──────────────────────────────────── -->
      <section id="apropos" class="card">
        <header class="card-head">
          <Info size={18} strokeWidth={1.6} />
          <h2>À propos de cette version</h2>
        </header>
        <dl class="about-grid">
          <dt>Application</dt>
          <dd>Sobr.ia <span class="mono">v0.2.0</span> · interface immersive</dd>

          <dt>Référentiel</dt>
          <dd class="mono">2026.05 (CalVer) · 8 presets de modèles validés au défi</dd>

          <dt>Méthodologie</dt>
          <dd>
            AFNOR SPEC 2314 (Ecolab, 2024) + facteurs ADEME Base Empreinte + EcoLogits (Data for
            Good) pour la couche modèle.
          </dd>

          <dt>Licences</dt>
          <dd>
            <span class="badge-pill">MIT</span> code ·
            <span class="badge-pill">Etalab 2.0</span> données ·
            <span class="badge-pill">CC-BY 4.0</span> docs ·
            <span class="badge-pill">SIL OFL 1.1</span>
            <a href="/methodo#fonts" class="link">polices</a>
          </dd>

          <dt>Sources</dt>
          <dd>
            ADEME · RTE · Hugging Face · Data for Good (EcoLogits) · ComparIA · ODRÉ
            (RTE/NaTran/Teréga IRIS) · Luccioni 2023 · Patterson 2021 · Gupta 2022 · Ren 2023.
          </dd>

          <dt>Local-first</dt>
          <dd>
            Toutes les estimations, le ledger d'audit et le référentiel sont stockés sur votre
            machine. Aucune donnée n'est envoyée vers un serveur distant — pas même pour la
            documentation que vous lisez actuellement.
          </dd>
        </dl>
      </section>
    </div>
  </div>
</div>

<style>
  .canvas-inner {
    max-width: 1240px;
    margin: 0 auto;
    padding: 40px 56px 80px;
  }

  /* TopBar (clone allégé) */
  .topbar {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 28px;
  }
  .breadcrumb {
    font: 400 13px/1 var(--font-ui);
    color: var(--ivory-3);
  }
  .breadcrumb .sep {
    color: var(--ivory-4);
    margin: 0 8px;
  }
  .breadcrumb .current {
    color: var(--ivory-2);
  }
  .spacer {
    flex: 1;
  }
  .local-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 12px;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: 999px;
    font: 500 12px/1 var(--font-ui);
    color: var(--lime);
  }
  .icon-btn {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--ivory-2);
    cursor: pointer;
    text-decoration: none;
    transition: all var(--dur-base) var(--ease);
  }
  .icon-btn:hover {
    background: var(--surface-hi);
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  /* Hero */
  .hero {
    padding-bottom: 24px;
    margin-bottom: 32px;
    border-bottom: 1px solid var(--border);
  }
  .hero-eyebrow {
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.16em;
    color: var(--ivory-3);
    margin-bottom: 14px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .hero-eyebrow .pulse {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--lime);
    box-shadow: 0 0 10px var(--lime);
  }
  .hero-h1 {
    font: 400 42px/1.1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.02em;
    max-width: 760px;
    margin: 0 0 10px;
  }
  .hero-h1 em {
    font-style: normal;
    color: var(--lime);
  }
  .hero-sub {
    font: 400 15px/1.6 var(--font-ui);
    color: var(--ivory-2);
    max-width: 720px;
    margin: 0;
  }

  /* Polish B — cross-link vers /methodologies (catalogue) */
  .page-crosslink {
    margin-top: 18px;
    padding: 12px 16px;
    background: var(--surface);
    border: 1px dashed var(--border-hi);
    border-radius: var(--radius-md);
    font: 400 13px/1.55 var(--font-ui);
    color: var(--ivory-2);
    max-width: 720px;
  }
  .page-crosslink strong {
    display: block;
    color: var(--ivory);
    font-weight: 500;
    margin-bottom: 4px;
  }
  .page-crosslink em {
    font-style: italic;
  }
  .crosslink-cta {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-top: 6px;
    color: var(--lime);
    text-decoration: none;
    font-weight: 500;
  }
  .crosslink-cta:hover {
    text-decoration: underline;
  }

  /* Layout 2 colonnes : TOC + main */
  .page-grid {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 32px;
    align-items: flex-start;
  }
  .toc {
    position: sticky;
    top: 24px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 12px 0;
    border-left: 1px solid var(--border);
  }
  .toc-item {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    font: 500 12px/1 var(--font-ui);
    color: var(--ivory-3);
    text-decoration: none;
    border-bottom: none;
    transition: all var(--dur-base) var(--ease);
    margin-left: -1px;
    border-left: 2px solid transparent;
  }
  .toc-item:hover {
    color: var(--ivory);
    border-left-color: var(--border-hi);
  }
  .toc-item:target,
  .toc-item:focus-visible {
    color: var(--lime);
    border-left-color: var(--lime);
  }

  .page-main {
    display: flex;
    flex-direction: column;
    gap: 24px;
    min-width: 0;
  }

  /* Cards */
  .card {
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 26px 30px;
    scroll-margin-top: 24px;
  }
  .card-head {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }
  .card-head :global(svg) {
    color: var(--lime);
    flex-shrink: 0;
  }
  .card-head h2 {
    font: 400 22px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 0;
    flex: 1;
  }
  .card-meta {
    font: 500 12px/1 var(--font-mono);
    color: var(--ivory-3);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    padding: 4px 10px;
  }
  .card p {
    font: 400 14px/1.6 var(--font-ui);
    color: var(--ivory-2);
    margin: 0 0 14px;
  }
  .card a {
    color: var(--lime);
    text-decoration: none;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.4);
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }
  .card a:hover {
    border-bottom-color: var(--lime);
  }
  .card-foot {
    margin: 16px 0 0 !important;
    font-size: 12px !important;
    color: var(--ivory-3) !important;
  }

  /* Formule */
  .formula {
    background: var(--ink-2);
    border: 1px solid var(--border);
    border-left: 3px solid var(--lime);
    border-radius: var(--radius-sm);
    padding: 14px 16px;
    margin: 0 0 16px;
    overflow-x: auto;
  }
  .formula code {
    font: 500 13px/1.6 var(--font-mono);
    color: var(--ivory);
    white-space: pre;
  }
  .formula-legend {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .formula-legend li {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-2);
  }
  .formula-legend .sym {
    font: 500 12px/1 var(--font-mono);
    color: var(--lime);
  }
  .formula-legend .sep {
    color: var(--ivory-4);
    margin: 0 6px;
  }

  /* Status grid (validation) */
  .status-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 12px;
    margin-top: 6px;
  }
  .status {
    display: flex;
    gap: 12px;
    padding: 14px 16px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }
  .status :global(svg) {
    color: var(--lime);
    flex-shrink: 0;
    margin-top: 3px;
  }
  .status-label {
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
  }
  .status-val {
    font: 400 16px/1.2 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 4px 0 4px;
  }
  .status-note {
    font: 400 12px/1.5 var(--font-ui);
    color: var(--ivory-3);
  }

  /* Glossaire */
  .glossary {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
    font: 400 13px/1.5 var(--font-ui);
  }
  .glossary thead th {
    text-align: left;
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
  }
  .glossary thead th:nth-child(1) {
    width: 160px;
  }
  .glossary thead th:nth-child(2) {
    width: 160px;
  }
  .glossary thead th:nth-child(4) {
    width: 130px;
  }
  .glossary tbody td {
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
    vertical-align: top;
    color: var(--ivory-2);
  }
  .glossary tbody tr:last-child td {
    border-bottom: none;
  }
  .glossary tbody tr:hover td {
    background: rgba(255, 255, 255, 0.02);
  }
  .gl-fr {
    color: var(--ivory);
    font-weight: 500;
  }
  .gl-en {
    color: var(--ivory-3);
    font-style: italic;
  }
  .gl-src {
    font: 400 12px/1.4 var(--font-mono);
    color: var(--ivory-3);
  }

  /* Références */
  .ref-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .ref-list li a {
    display: flex;
    align-items: baseline;
    gap: 12px;
    padding: 12px 14px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    border-bottom: 1px solid var(--border);
    transition: all var(--dur-base) var(--ease);
    text-decoration: none;
  }
  .ref-list li a:hover {
    border-color: rgba(197, 240, 74, 0.3);
    background: rgba(197, 240, 74, 0.04);
  }
  .ref-title {
    font: 500 14px/1.3 var(--font-ui);
    color: var(--ivory);
    flex: 1;
  }
  .ref-meta {
    font: 400 12px/1 var(--font-mono);
    color: var(--ivory-3);
    flex-shrink: 0;
  }
  .ref-list :global(svg) {
    color: var(--ivory-3);
    flex-shrink: 0;
  }
  .ref-list a:hover :global(svg) {
    color: var(--lime);
  }

  /* Bibliographie */
  .biblio-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .biblio-list li {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 14px;
    border-left: 2px solid var(--border);
  }
  .biblio-list li:hover {
    border-left-color: var(--lime);
  }
  .biblio-label {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory);
  }
  .biblio-hint {
    font: 400 12px/1 var(--font-mono);
    color: var(--ivory-3);
  }

  /* À propos */
  .about-grid {
    display: grid;
    grid-template-columns: 130px 1fr;
    gap: 12px 20px;
    margin: 0;
  }
  .about-grid dt {
    font: 500 12px/1.2 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    padding-top: 4px;
  }
  .about-grid dd {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory);
    margin: 0;
  }
  .badge-pill {
    display: inline-flex;
    padding: 2px 8px;
    margin-right: 4px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    font: 500 12px/1.4 var(--font-mono);
    color: var(--ivory-2);
  }
  .link {
    color: var(--lime);
    text-decoration: none;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.4);
  }

  @media (max-width: 960px) {
    .page-grid {
      grid-template-columns: 1fr;
    }
    .toc {
      position: static;
      flex-direction: row;
      flex-wrap: wrap;
      gap: 4px;
      border-left: none;
      border-bottom: 1px solid var(--border);
      padding-bottom: 12px;
    }
    .toc-item {
      border-left: none;
      border-bottom: 2px solid transparent;
    }
    .toc-item:hover {
      border-left: none;
      border-bottom-color: var(--border-hi);
    }
  }
  @media (max-width: 720px) {
    .canvas-inner {
      padding: 24px 16px 60px;
    }
    .hero-h1 {
      font-size: 32px;
    }
    .card {
      padding: 20px;
    }
    .about-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
