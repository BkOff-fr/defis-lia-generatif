<script lang="ts">
  import { onMount } from 'svelte';
  import { hasWebGL, prefersReducedMotion } from '$lib/three-utils';

  type Method = 'afnor' | 'ecologits';
  let method: Method = $state('afnor');
  let container: HTMLDivElement | null = $state(null);
  let disabled = $state(false);
  let stats = $state({ p5: 0, p50: 0, p95: 0 });

  // Paramètres illustratifs (ordre de grandeur cohérent avec Mistral Large 2 ~1.14 g/prompt).
  const PARAMS: Record<Method, { mean: number; sigma: number; label: string }> = {
    afnor: { mean: 1.4, sigma: 0.45, label: 'AFNOR SPEC 2314 / Sobr.ia' },
    ecologits: { mean: 1.15, sigma: 0.3, label: 'EcoLogits 2024' },
  };

  function gaussRand(): number {
    const u = Math.random() || 1e-9;
    const v = Math.random() || 1e-9;
    return Math.sqrt(-2 * Math.log(u)) * Math.cos(2 * Math.PI * v);
  }

  function percentiles(values: number[]) {
    const sorted = [...values].sort((a, b) => a - b);
    const at = (q: number) => sorted[Math.floor(q * (sorted.length - 1))];
    return { p5: at(0.05), p50: at(0.5), p95: at(0.95) };
  }

  let regenTargets: ((m: Method) => void) | null = null;

  // Re-run when method changes (after onMount has wired regenTargets).
  $effect(() => {
    regenTargets?.(method);
  });

  onMount(() => {
    if (!container) return;
    if (!hasWebGL() || prefersReducedMotion()) {
      disabled = true;
      return;
    }
    let canceled = false;
    let stop: (() => void) | null = null;

    (async () => {
      const THREE = await import('three');
      if (canceled || !container) return;

      const N = 10000;
      const W = container.clientWidth;
      const H = container.clientHeight;

      const scene = new THREE.Scene();
      const camera = new THREE.PerspectiveCamera(40, W / H, 0.1, 100);
      camera.position.set(0, 0.6, 6);
      camera.lookAt(0, 0, 0);

      const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
      renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
      renderer.setSize(W, H);
      renderer.setClearColor(0x000000, 0);
      container.appendChild(renderer.domElement);

      const geom = new THREE.SphereGeometry(0.015, 4, 4);
      const mat = new THREE.MeshBasicMaterial({
        color: 0xc5f04a,
        transparent: true,
        opacity: 0.55,
      });
      const mesh = new THREE.InstancedMesh(geom, mat, N);
      mesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
      scene.add(mesh);

      const axisGeom = new THREE.BoxGeometry(5, 0.005, 0.005);
      const axisMat = new THREE.MeshBasicMaterial({ color: 0x46443f });
      const axis = new THREE.Mesh(axisGeom, axisMat);
      axis.position.y = -1;
      scene.add(axis);

      const bandMat = new THREE.MeshBasicMaterial({
        color: 0xf5b769,
        transparent: true,
        opacity: 0.18,
        side: THREE.DoubleSide,
      });
      const bandGeom = new THREE.PlaneGeometry(0.025, 2.4);
      const bandP5 = new THREE.Mesh(bandGeom, bandMat);
      const bandP95 = new THREE.Mesh(bandGeom, bandMat);
      scene.add(bandP5, bandP95);

      const dummy = new THREE.Object3D();
      const positions = new Float32Array(N * 3);
      const targets = new Float32Array(N * 3);
      const samples: number[] = new Array(N);

      const doRegen = (m: Method) => {
        const { mean, sigma } = PARAMS[m];
        for (let i = 0; i < N; i++) {
          const sample = mean + sigma * gaussRand();
          samples[i] = sample;
          const x = ((sample - mean) / (3 * sigma)) * 2;
          const angle = Math.random() * Math.PI * 2;
          const r = Math.random() * 0.15 + 0.02;
          targets[i * 3] = x;
          targets[i * 3 + 1] = Math.sin(angle) * r - 0.5;
          targets[i * 3 + 2] = Math.cos(angle) * r;
        }
        const p = percentiles(samples);
        stats = p;
        const toX = (s: number) => ((s - mean) / (3 * sigma)) * 2;
        bandP5.position.x = toX(p.p5);
        bandP95.position.x = toX(p.p95);
      };

      // Initialize at scrambled positions (dance phase)
      for (let i = 0; i < N; i++) {
        positions[i * 3] = (Math.random() - 0.5) * 5;
        positions[i * 3 + 1] = (Math.random() - 0.5) * 2.5 - 0.5;
        positions[i * 3 + 2] = (Math.random() - 0.5) * 2;
      }

      regenTargets = doRegen;
      doRegen(method);

      let converging = false;
      const io = new IntersectionObserver(
        (entries) => {
          for (const e of entries) {
            if (e.isIntersecting) {
              converging = true;
              io.disconnect();
              break;
            }
          }
        },
        { threshold: 0.3 },
      );
      io.observe(container);

      const ro = new ResizeObserver(() => {
        if (!container) return;
        const w = container.clientWidth;
        const h = container.clientHeight;
        camera.aspect = w / h;
        camera.updateProjectionMatrix();
        renderer.setSize(w, h);
      });
      ro.observe(container);

      let raf = 0;
      let last = performance.now();
      const animate = (now: number) => {
        const dt = Math.min((now - last) / 1000, 0.05);
        last = now;

        const lerp = converging ? Math.min(dt * 1.8, 1) : Math.min(dt * 0.4, 1);
        for (let i = 0; i < N; i++) {
          const ix = i * 3;
          positions[ix] += (targets[ix] - positions[ix]) * lerp;
          positions[ix + 1] += (targets[ix + 1] - positions[ix + 1]) * lerp;
          positions[ix + 2] += (targets[ix + 2] - positions[ix + 2]) * lerp;
          dummy.position.set(positions[ix], positions[ix + 1], positions[ix + 2]);
          dummy.updateMatrix();
          mesh.setMatrixAt(i, dummy.matrix);
        }
        mesh.instanceMatrix.needsUpdate = true;
        scene.rotation.y = Math.sin(now * 0.00015) * 0.08;
        renderer.render(scene, camera);
        raf = requestAnimationFrame(animate);
      };
      raf = requestAnimationFrame(animate);

      stop = () => {
        cancelAnimationFrame(raf);
        ro.disconnect();
        io.disconnect();
        renderer.dispose();
        geom.dispose();
        mat.dispose();
        axisGeom.dispose();
        axisMat.dispose();
        bandGeom.dispose();
        bandMat.dispose();
        regenTargets = null;
        if (container && renderer.domElement.parentNode === container) {
          container.removeChild(renderer.domElement);
        }
      };
    })();

    return () => {
      canceled = true;
      stop?.();
    };
  });

  const METHODS: Method[] = ['afnor', 'ecologits'];
</script>

<div class="grid gap-6 lg:grid-cols-[1fr_320px] items-center">
  <div
    bind:this={container}
    class="w-full h-[420px] rounded-lg border border-ivory-4/30 bg-ink-2/40"
    aria-hidden="true"
  ></div>

  <aside class="space-y-4">
    <div class="flex gap-2">
      {#each METHODS as m (m)}
        <button
          type="button"
          onclick={() => (method = m)}
          class:list={[
            'rounded-md px-3 py-1.5 text-xs font-sans border transition-colors',
            method === m
              ? 'bg-lime-signature text-ink border-lime-signature'
              : 'text-ivory-2 border-ivory-4/40 hover:border-lime-signature/40',
          ]}
        >
          {m === 'afnor' ? 'AFNOR / Sobr.ia' : 'EcoLogits'}
        </button>
      {/each}
    </div>

    {#if !disabled}
      <dl class="grid grid-cols-3 gap-2 text-center">
        <div class="rounded border border-amber/30 bg-amber/5 p-3">
          <dt class="text-[10px] uppercase text-ivory-3 tracking-wider">P5</dt>
          <dd class="mt-1 font-mono text-lg text-amber">{stats.p5.toFixed(2)}</dd>
        </div>
        <div class="rounded border border-lime-signature/40 bg-lime-signature/10 p-3">
          <dt class="text-[10px] uppercase text-ivory-3 tracking-wider">P50</dt>
          <dd class="mt-1 font-mono text-lg text-lime-signature">{stats.p50.toFixed(2)}</dd>
        </div>
        <div class="rounded border border-amber/30 bg-amber/5 p-3">
          <dt class="text-[10px] uppercase text-ivory-3 tracking-wider">P95</dt>
          <dd class="mt-1 font-mono text-lg text-amber">{stats.p95.toFixed(2)}</dd>
        </div>
      </dl>
      <p class="text-xs text-ivory-3">
        gCO₂eq / prompt · {PARAMS[method].label}
      </p>
    {:else}
      <p class="text-xs text-ivory-3">
        Visualisation 3D désactivée (WebGL absent ou prefers-reduced-motion actif).
      </p>
    {/if}
  </aside>
</div>
