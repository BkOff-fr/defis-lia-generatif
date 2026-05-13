# Licences des polices embarquées — Sobr.ia

> **Version** : v0.2 — alignée sur le design system v2 (chantier C09.C).
> **Statut** : conformité OSS pour `web/static/fonts/*.woff2`.
> **À mettre à jour** quand on rapatrie une nouvelle police ou un nouveau sous-ensemble Unicode.

Toutes les polices embarquées sont publiées sous **SIL Open Font License 1.1** (OFL).
Texte intégral plus bas (identique pour les trois). Les trois familles sont
auto-hébergées dans `web/static/fonts/` pour respecter la CSP
`default-src 'self'` de Tauri (cf. [CLAUDE.md](../CLAUDE.md) §3 anti-patterns).

---

## Inventaire des fichiers embarqués

| Fichier WOFF2                             | Famille          | Style               | Provenance                                      | SHA-256                                                            |
| ----------------------------------------- | ---------------- | ------------------- | ----------------------------------------------- | ------------------------------------------------------------------ |
| `geist-latin.woff2`                       | Geist            | 300–700 (var.)      | github.com/vercel/geist-font                    | `0cbbe6286a00f356e98980783cc950a9b693751e04aedfb97d9526ff6dc2b316` |
| `geist-latin-ext.woff2`                   | Geist            | 300–700 (var.)      | idem                                            | `3640f9351b2a4631515dffea38092c24e70db9df2ef1e97c10cbdeb4163e6186` |
| `instrument-serif-latin.woff2`            | Instrument Serif | 400 normal          | github.com/Instrument/instrument-serif          | `5eb09b5ac0e28b67c2f041c8ba6d244604ca0c0980d65912ab2d47fed84ddc31` |
| `instrument-serif-latin-ext.woff2`        | Instrument Serif | 400 normal          | idem                                            | `290e6267dd833bf5f899eba4c29ad0a9b09dbe53f6075b18af38057159e1ff20` |
| `instrument-serif-italic-latin.woff2`     | Instrument Serif | 400 italic          | idem                                            | `5a51946dfffa82972bc98745359c46761515641fda557c25116459a9f83da4a7` |
| `instrument-serif-italic-latin-ext.woff2` | Instrument Serif | 400 italic          | idem                                            | `d0014821252a8a770bf7551641572f239a8ede83d75652d84fef0c3627751baf` |
| `jetbrains-mono-latin.woff2`              | JetBrains Mono   | 400–600 (var.)      | github.com/JetBrains/JetBrainsMono              | `83c005d49d8a6a50474c73a5a36ac0468076e9c4a29da7bdb14995d80560a5be` |
| `jetbrains-mono-latin-ext.woff2`          | JetBrains Mono   | 400–600 (var.)      | idem                                            | `db5ff4db83e580426280e9337a58dc57d3a83784a1b03ad80914651594441d52` |

Pour re-vérifier l'intégrité d'un fichier après mise à jour :

```powershell
Get-FileHash -Algorithm SHA256 web\static\fonts\<file>.woff2
```

```bash
sha256sum web/static/fonts/<file>.woff2
```

---

## Copyrights par famille

```
Copyright 2024 The Geist Project Authors
  (https://github.com/vercel/geist-font)

Copyright 2022 The Instrument Serif Project Authors
  (https://github.com/Instrument/instrument-serif)

Copyright 2020 The JetBrains Mono Project Authors
  (https://github.com/JetBrains/JetBrainsMono)
```

Les trois fichiers Software sont distribués sous le texte de licence ci-dessous.

---

## SIL Open Font License v1.1 (texte intégral)

```
-----------------------------------------------------------
SIL OPEN FONT LICENSE Version 1.1 - 26 February 2007
-----------------------------------------------------------

PREAMBLE
The goals of the Open Font License (OFL) are to stimulate worldwide
development of collaborative font projects, to support the font creation
efforts of academic and linguistic communities, and to provide a free and
open framework in which fonts may be shared and improved in partnership
with others.

The OFL allows the licensed fonts to be used, studied, modified and
redistributed freely as long as they are not sold by themselves. The
fonts, including any derivative works, can be bundled, embedded,
redistributed and/or sold with any software provided that any reserved
names are not used by derivative works. The fonts and derivatives,
however, cannot be released under any other type of license. The
requirement for fonts to remain under this license does not apply
to any document created using the fonts or their derivatives.

DEFINITIONS
"Font Software" refers to the set of files released by the Copyright
Holder(s) under this license and clearly marked as such. This may
include source files, build scripts and documentation.

"Reserved Font Name" refers to any names specified as such after the
copyright statement(s).

"Original Version" refers to the collection of Font Software components as
distributed by the Copyright Holder(s).

"Modified Version" refers to any derivative made by adding to, deleting,
or substituting -- in part or in whole -- any of the components of the
Original Version, by changing formats or by porting the Font Software to a
new environment.

"Author" refers to any designer, engineer, programmer, technical
writer or other person who contributed to the Font Software.

PERMISSION & CONDITIONS
Permission is hereby granted, free of charge, to any person obtaining
a copy of the Font Software, to use, study, copy, merge, embed, modify,
redistribute, and sell modified and unmodified copies of the Font
Software, subject to the following conditions:

1) Neither the Font Software nor any of its individual components,
in Original or Modified Versions, may be sold by itself.

2) Original or Modified Versions of the Font Software may be bundled,
redistributed and/or sold with any software, provided that each copy
contains the above copyright notice and this license. These can be
included either as stand-alone text files, human-readable headers or
in the appropriate machine-readable metadata fields within text or
binary files as long as those fields can be easily viewed by the user.

3) No Modified Version of the Font Software may use the Reserved Font
Name(s) unless explicit written permission is granted by the corresponding
Copyright Holder. This restriction only applies to the primary font name as
presented to the users.

4) The name(s) of the Copyright Holder(s) or the Author(s) of the Font
Software shall not be used to promote, endorse or advertise any
Modified Version, except to acknowledge the contribution(s) of the
Copyright Holder(s) and the Author(s) or with their explicit written
permission.

5) The Font Software, modified or unmodified, in part or in whole,
must be distributed entirely under this license, and must not be
distributed under any other license. The requirement for fonts to
remain under this license does not apply to any document created
using the Font Software.

TERMINATION
This license becomes null and void if any of the above conditions are
not met.

DISCLAIMER
THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT
OF COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL THE
COPYRIGHT HOLDER BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM
OTHER DEALINGS IN THE FONT SOFTWARE.
```

Source canonique : <https://openfontlicense.org> · récupéré le 2026-05-13 depuis le dépôt Geist (LICENSE.txt à la racine).

---

## Reserved Font Names

D'après les copyrights upstream, **aucune des trois familles ne déclare de
*Reserved Font Name*** dans son fichier OFL.txt. La clause OFL §3 n'impose
donc aucun renommage en cas de redistribution non modifiée — c'est ce que
nous faisons (les WOFF2 sont des sous-ensembles Unicode du WOFF2 originel,
sans modification de glyphes).

Si nous étions amenés à modifier les fichiers (re-hinting, fork, etc.), il
faudrait renommer la famille (par ex. `Sobria Geist`) — pas le cas
aujourd'hui.

---

## Mise à jour du référentiel

1. Mettre à jour le WOFF2 dans `web/static/fonts/`.
2. Recalculer le SHA-256 (`Get-FileHash` ou `sha256sum`).
3. Mettre à jour la ligne correspondante dans la table « Inventaire » ci-dessus.
4. Bumper `web/static/fonts/README.md` § Inventaire.
5. Commit `chore(web): update <fontname> WOFF2 (subset latin/latin-ext)`.
