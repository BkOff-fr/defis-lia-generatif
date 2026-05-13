<script lang="ts">
  type Props = {
    size?: number;
    /** Disable per-instance animations (vein flow + node pulse).
     * Le souffle d'ensemble (.breath) reste piloté par l'appelant. */
    static?: boolean;
    /** ARIA accessible label — default "Sobr.ia". */
    label?: string;
    /** Generate unique IDs for gradients when several marks coexist on the
     * same page (sinon les `url(#…)` collisionnent). */
    uid?: string;
  };
  const { size = 44, static: isStatic = false, label = 'Sobr.ia', uid = 'sob' }: Props = $props();

  const leafG = $derived(`${uid}-leaf`);
  const veinG = $derived(`${uid}-vein`);
  const nodeG = $derived(`${uid}-node`);
</script>

<svg
  class="mark"
  class:animated={!isStatic}
  width={size}
  height={size}
  viewBox="0 0 100 100"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
  role="img"
  aria-label={label}
>
  <defs>
    <linearGradient id={leafG} x1="20" y1="15" x2="85" y2="90" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#e0ff80" />
      <stop offset="60%" stop-color="#c5f04a" />
      <stop offset="100%" stop-color="#7a9a32" />
    </linearGradient>
    <linearGradient id={veinG} x1="0" x2="100" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#c5f04a" stop-opacity=".25" />
      <stop offset="50%" stop-color="#c5f04a" />
      <stop offset="100%" stop-color="#c5f04a" stop-opacity=".25" />
    </linearGradient>
    <radialGradient id={nodeG} cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#ffffff" />
      <stop offset="30%" stop-color="#e0ff80" />
      <stop offset="100%" stop-color="#c5f04a" />
    </radialGradient>
  </defs>

  <!-- Outer leaf (S-curve : la première lettre du wordmark) -->
  <path
    class="leaf-out"
    d="M 18 78 C 12 50, 32 18, 64 22 C 78 24, 86 36, 84 52 C 81 78, 52 88, 22 84"
    stroke="url(#{leafG})"
    stroke-width="3.2"
    stroke-linecap="round"
    fill="none"
  />
  <!-- Inner leaf (referme la S en feuille) -->
  <path
    class="leaf-in"
    d="M 22 84 C 30 70, 38 56, 48 50 C 60 42, 74 38, 84 52"
    stroke="url(#{leafG})"
    stroke-width="2.2"
    stroke-linecap="round"
    fill="none"
    opacity=".7"
  />
  <!-- Central vein → circuit conductor -->
  <path
    class="vein"
    d="M 22 80 Q 38 64, 48 50 T 80 28"
    stroke="url(#{veinG})"
    stroke-width="1.4"
    stroke-linecap="round"
    stroke-dasharray="6 4"
    fill="none"
  />
  <!-- 3 datapoint nodes — IA -->
  <circle class="node node-a" cx="32" cy="72" r="3.4" fill="url(#{nodeG})" />
  <circle class="node node-b" cx="50" cy="52" r="4.2" fill="url(#{nodeG})" />
  <circle class="node node-c" cx="74" cy="32" r="3" fill="url(#{nodeG})" />
  <!-- Subtle tip accent -->
  <circle cx="82" cy="26" r="1.2" fill="#c5f04a" />
</svg>

<style>
  .mark {
    display: block;
    max-width: 100%;
    max-height: 100%;
  }

  /* Tracé progressif au mount — uniquement quand animé. */
  .mark.animated .leaf-out {
    stroke-dasharray: 280;
    stroke-dashoffset: 280;
    animation: mark-draw 1.6s cubic-bezier(0.65, 0, 0.35, 1) forwards;
  }
  .mark.animated .leaf-in {
    stroke-dasharray: 280;
    stroke-dashoffset: 280;
    animation: mark-draw 1.8s 0.25s cubic-bezier(0.65, 0, 0.35, 1) forwards;
  }
  /* Flux électrique le long de la nervure. */
  .mark.animated .vein {
    animation: mark-vein 2s linear infinite;
  }
  /* Pulse des trois nœuds, désynchronisées. */
  .mark.animated .node {
    animation: mark-pulse 2.4s ease-in-out infinite;
  }
  .mark.animated .node-b {
    animation-delay: 0.4s;
  }
  .mark.animated .node-c {
    animation-delay: 0.8s;
  }

  @keyframes mark-draw {
    to {
      stroke-dashoffset: 0;
    }
  }
  @keyframes mark-vein {
    to {
      stroke-dashoffset: -40;
    }
  }
  @keyframes mark-pulse {
    0%,
    100% {
      opacity: 0.6;
    }
    50% {
      opacity: 1;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .mark.animated .leaf-out,
    .mark.animated .leaf-in {
      stroke-dasharray: none;
      stroke-dashoffset: 0;
    }
    .mark.animated .vein,
    .mark.animated .node {
      animation: none;
    }
  }
</style>
