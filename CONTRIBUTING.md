# Contribuer à Sobr.ia

Merci de l'intérêt que vous portez au projet. Sobr.ia est open-source (MIT, Etalab 2.0, CC-BY) et ouvert aux contributions.

---

## Avant tout

1. Lire [`README.md`](README.md), [`CLAUDE.md`](CLAUDE.md), et le [cahier des charges](docs/CAHIER-DES-CHARGES-v1.0.md).
2. Identifier le sprint actif dans [`docs/ROADMAP.md`](docs/ROADMAP.md) et le brief associé.
3. Respecter les [ADR](docs/adr/) — les décisions architecturales sont actées.

---

## Cycle de contribution

1. **Discuter d'abord** — ouvrir une issue GitHub pour décrire l'idée avant de coder.
2. **Forker / brancher** — branche `feature/<scope>-<short-desc>` ou `fix/<short-desc>`.
3. **Coder en respectant** la [Definition of Done](CLAUDE.md#5-definition-of-done-dod).
4. **Commits** au format Conventional Commits (voir CLAUDE.md §4.3).
5. **Pull Request** — référencer l'issue + ADR concernés.
6. **Revue** — au moins 1 mainteneur, CI verte, et discussion résolue.

---

## Standards techniques

| Aspect | Outil | Cible |
|--------|-------|-------|
| Format Rust | `cargo fmt --all` | obligatoire |
| Lint Rust | `cargo clippy --workspace --all-targets -- -D warnings` | obligatoire |
| Couverture tests | `cargo tarpaulin --workspace` | ≥ 80 % |
| Format TS | `prettier` | obligatoire |
| Lint TS | `eslint` strict | obligatoire |
| Audit sécurité | `cargo audit` + `cargo deny check` | 0 critical |
| Accessibilité | `axe-core` via Playwright | RGAA AA |

---

## Developer Certificate of Origin (DCO)

En ouvrant une PR, vous certifiez que vous avez le droit de soumettre la
contribution et que vous acceptez le DCO ([texte officiel](https://developercertificate.org/)).

Signer vos commits :

```bash
git commit -s -m "feat(estimator): add Monte-Carlo propagation"
```

---

## Méthodologie scientifique — règles strictes

Voir CLAUDE.md §6 et `docs/methodology/`.

- ✅ Toute formule de calcul est sourcée (commentaire avec DOI/URL).
- ✅ Toute hypothèse a une distribution d'incertitude documentée.
- ✅ Tout résultat affiche un intervalle P5-P95.
- ✅ Reproductibilité par seed (`SOBRIA_SEED`, défaut 42).
- ❌ Pas de chiffre "magique" sans citation.

---

## Architecture médaillon — règles non-négociables

Voir ADR-0009.

- Toute donnée externe traverse Copper → Silver → Gold.
- Une nouvelle source = un seul trait `DataLayer` implémenté.
- Pas de transformation *ad hoc* hors du pipeline.

---

## Communication

- **Issues** : pour les bugs, idées, questions.
- **Discussions** : pour les sujets d'architecture.
- **Email mainteneurs** : sécurité ou conflit (voir SECURITY.md).

Tous les échanges suivent le [Code de conduite](CODE_OF_CONDUCT.md) (Contributor Covenant 2.1).
