---
title: "D2 PDF baseline"
subtitle: "Sequence, class and ERD rendering"
course: "Baseline"
language: en
sources: [PD0]
running: "D2 PDF baseline"
---

::: front kicker="BASELINE" title="D2 per-kind rendering" lead="The same guide renders one diagram of each v3 kind."
:::

## A · Rendering

The baseline verifies that the D2 v3 sequence, class and entity-relationship
renderers reach the PDF with stable layout and typography. [PD0, s. 1]

::: diagram id="sequence-pilot"
:::

The sequence diagram places actors above ordered messages; the note belongs to
the client lifeline. [PD0, s. 1]

::: diagram id="class-pilot"
:::

The class diagram keeps fields and methods inside the class body and draws
inheritance with a triangle arrowhead. [PD0, s. 1]

::: diagram id="erd-pilot"
:::

The entity-relationship diagram keeps primary and foreign keys visible and
draws the many-to-one relation between orders and customers. [PD0, s. 1]
