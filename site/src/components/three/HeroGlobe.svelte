<script lang="ts">
  import { onMount } from 'svelte';
  import { hasWebGL, prefersReducedMotion, latLonToVec3 } from '$lib/three-utils';
  import datacenters from '../../data/datacenters.json';

  interface Props {
    height?: string;
    children?: import('svelte').Snippet;
  }
  let { height = '70vh', children }: Props = $props();

  let container: HTMLDivElement | null = $state(null);
  let disabled = $state(false);

  onMount(() => {
    if (!container) return;
    if (!hasWebGL() || prefersReducedMotion()) {
      disabled = true;
      return;
    }

    let canceled = false;
    let stopFn: (() => void) | null = null;

    (async () => {
      const THREE = await import('three');
      if (canceled || !container) return;

      const width = container.clientWidth;
      const heightPx = container.clientHeight;

      const scene = new THREE.Scene();
      const camera = new THREE.PerspectiveCamera(45, width / heightPx, 0.1, 100);
      camera.position.set(0, 0, 7);

      const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
      renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
      renderer.setSize(width, heightPx);
      renderer.setClearColor(0x000000, 0);
      container.appendChild(renderer.domElement);

      // ─── Group everything so we can rotate as one ──────────────────────
      const group = new THREE.Group();
      scene.add(group);

      // ─── Globe sphere (matte ink fill + low-poly wireframe overlay) ────
      const sphereGeom = new THREE.IcosahedronGeometry(2, 4);
      const sphereMat = new THREE.MeshStandardMaterial({
        color: 0x0e1310,
        roughness: 0.95,
        metalness: 0.05,
        flatShading: true,
      });
      const sphereMesh = new THREE.Mesh(sphereGeom, sphereMat);
      group.add(sphereMesh);

      const wireframeGeom = new THREE.IcosahedronGeometry(2.005, 4);
      const wireframeMat = new THREE.LineBasicMaterial({
        color: 0xc5f04a,
        transparent: true,
        opacity: 0.18,
      });
      const wireframe = new THREE.LineSegments(
        new THREE.WireframeGeometry(wireframeGeom),
        wireframeMat,
      );
      group.add(wireframe);

      // ─── Datacenters (28 emissive lime spots) ─────────────────────────
      const dcGeom = new THREE.SphereGeometry(0.04, 12, 12);
      const dcMats: THREE.MeshStandardMaterial[] = [];
      const dcOffsets: number[] = [];
      const dcVec = new THREE.Vector3();
      for (let i = 0; i < datacenters.length; i++) {
        const d = datacenters[i];
        latLonToVec3(d.lat, d.lon, 2.02, dcVec);
        const mat = new THREE.MeshStandardMaterial({
          color: 0xc5f04a,
          emissive: 0xc5f04a,
          emissiveIntensity: 1.4,
          transparent: true,
          opacity: 0.9,
        });
        dcMats.push(mat);
        dcOffsets.push(Math.random() * Math.PI * 2);
        const mesh = new THREE.Mesh(dcGeom, mat);
        mesh.position.copy(dcVec);
        group.add(mesh);
      }

      // ─── Particles (1000 ambre, rising) ───────────────────────────────
      const PARTICLE_COUNT = 1000;
      const particleGeom = new THREE.SphereGeometry(0.012, 4, 4);
      const particleMat = new THREE.MeshBasicMaterial({
        color: 0xf5b769,
        transparent: true,
        opacity: 0.55,
      });
      const particles = new THREE.InstancedMesh(particleGeom, particleMat, PARTICLE_COUNT);
      const dummy = new THREE.Object3D();
      const particlePositions: { x: number; y: number; z: number; speed: number }[] = [];
      for (let i = 0; i < PARTICLE_COUNT; i++) {
        const x = (Math.random() - 0.5) * 8;
        const y = Math.random() * 6 - 3;
        const z = (Math.random() - 0.5) * 8;
        particlePositions.push({ x, y, z, speed: 0.0015 + Math.random() * 0.0035 });
        dummy.position.set(x, y, z);
        dummy.updateMatrix();
        particles.setMatrixAt(i, dummy.matrix);
      }
      particles.instanceMatrix.needsUpdate = true;
      scene.add(particles);

      // ─── Lights (ambient subtle + key from camera side) ───────────────
      scene.add(new THREE.AmbientLight(0xffffff, 0.25));
      const keyLight = new THREE.DirectionalLight(0xffffff, 0.9);
      keyLight.position.set(3, 2, 4);
      scene.add(keyLight);

      // ─── Interaction (auto-orbit + pointer drag) ──────────────────────
      let rotX = 0.15;
      let rotY = 0;
      let dragging = false;
      let lastX = 0;
      let lastY = 0;

      const onPointerDown = (e: PointerEvent) => {
        dragging = true;
        lastX = e.clientX;
        lastY = e.clientY;
        (e.target as HTMLElement).setPointerCapture?.(e.pointerId);
      };
      const onPointerMove = (e: PointerEvent) => {
        if (!dragging) return;
        rotY += (e.clientX - lastX) * 0.005;
        rotX += (e.clientY - lastY) * 0.005;
        rotX = Math.max(-1.2, Math.min(1.2, rotX));
        lastX = e.clientX;
        lastY = e.clientY;
      };
      const onPointerUp = () => {
        dragging = false;
      };
      renderer.domElement.addEventListener('pointerdown', onPointerDown);
      window.addEventListener('pointermove', onPointerMove);
      window.addEventListener('pointerup', onPointerUp);

      // ─── Resize ───────────────────────────────────────────────────────
      const ro = new ResizeObserver(() => {
        if (!container) return;
        const w = container.clientWidth;
        const h = container.clientHeight;
        camera.aspect = w / h;
        camera.updateProjectionMatrix();
        renderer.setSize(w, h);
      });
      ro.observe(container);

      // ─── RAF loop ─────────────────────────────────────────────────────
      let raf = 0;
      let last = performance.now();
      const animate = (now: number) => {
        const dt = (now - last) / 1000;
        last = now;

        if (!dragging) {
          rotY += dt * 0.08;
        }
        group.rotation.x = rotX;
        group.rotation.y = rotY;

        for (let i = 0; i < dcMats.length; i++) {
          const phase = now * 0.0018 + dcOffsets[i];
          const s = Math.sin(phase);
          dcMats[i].opacity = 0.55 + 0.45 * s * s;
        }

        for (let i = 0; i < PARTICLE_COUNT; i++) {
          const p = particlePositions[i];
          p.y += p.speed * dt * 60;
          if (p.y > 3.5) {
            p.x = (Math.random() - 0.5) * 8;
            p.y = -3.5;
            p.z = (Math.random() - 0.5) * 8;
          }
          dummy.position.set(p.x, p.y, p.z);
          dummy.updateMatrix();
          particles.setMatrixAt(i, dummy.matrix);
        }
        particles.instanceMatrix.needsUpdate = true;

        renderer.render(scene, camera);
        raf = requestAnimationFrame(animate);
      };
      raf = requestAnimationFrame(animate);

      stopFn = () => {
        cancelAnimationFrame(raf);
        ro.disconnect();
        renderer.domElement.removeEventListener('pointerdown', onPointerDown);
        window.removeEventListener('pointermove', onPointerMove);
        window.removeEventListener('pointerup', onPointerUp);
        renderer.dispose();
        sphereGeom.dispose();
        wireframeGeom.dispose();
        dcGeom.dispose();
        particleGeom.dispose();
        dcMats.forEach((m) => m.dispose());
        sphereMat.dispose();
        wireframeMat.dispose();
        particleMat.dispose();
        if (container && renderer.domElement.parentNode === container) {
          container.removeChild(renderer.domElement);
        }
      };
    })();

    return () => {
      canceled = true;
      stopFn?.();
    };
  });
</script>

{#if disabled && children}
  {@render children()}
{:else}
  <div bind:this={container} style:height class="w-full" aria-hidden="true"></div>
{/if}
