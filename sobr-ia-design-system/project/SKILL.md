---
name: sobria-design
description: Use this skill to generate well-branded interfaces and assets for Sobr.ia (outil de calcul et d'exploration dédié à la sobriété de l'IA générative), either for production or throwaway prototypes/mocks/etc. Contains essential design guidelines, colors, type, fonts, assets, and UI kit components for prototyping.
user-invocable: true
---

Read the `README.md` file within this skill, and explore the other available files (`colors_and_type.css`, `preview/`, `ui_kits/app/`, `ui_kits/extension/`).

If creating visual artifacts (slides, mocks, throwaway prototypes, etc), copy assets out and create static HTML files for the user to view. If working on production code, you can copy assets and read the rules here to become an expert in designing with this brand.

If the user invokes this skill without any other guidance, ask them what they want to build or design, ask some questions, and act as an expert designer who outputs HTML artifacts or production code, depending on the need.

Key constraints to respect:
- **French-first copy**, English available — sober technical tone, no marketing fluff, no exclamation marks.
- **Dark mode by default**, OLED-friendly. Light mode is opt-in.
- **All numeric values come with an uncertainty interval** (P5–P95).
- **Permanent reminder of local-first**: "🔒 100 % local, aucune donnée envoyée" in shell/footer.
- **Typography**: Inter (UI) + JetBrains Mono (numbers, code, hashes).
- **Avoid AI slop**: no purple gradients, no decorative emoji (functional only — 🧮 📚 ⚖ 📈 🌍 🔒 🌱), no big drop shadows. Borders 1px, radii 4/8/12 px.
- **Iconography is Unicode emoji** (functional) by deliberate choice — see `README.md` § ICONOGRAPHY.
