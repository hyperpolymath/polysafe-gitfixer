;; SPDX-License-Identifier: AGPL-3.0-or-later
;; SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
;; ECOSYSTEM.scm — polysafe-gitfixer

(ecosystem
  (version "1.0.0")
  (name "polysafe-gitfixer")
  (type "project")
  (purpose "// SPDX-License-Identifier: MIT AND Palimpsest-0.8")

  (position-in-ecosystem
    "Part of hyperpolymath ecosystem. Follows RSR guidelines.")

  (related-projects
    (project (name "rhodium-standard-repositories")
             (url "https://github.com/hyperpolymath/rhodium-standard-repositories")
             (relationship "standard")))

  (what-this-is "// SPDX-License-Identifier: MIT AND Palimpsest-0.8")
  (what-this-is-not "- NOT exempt from RSR compliance"))
