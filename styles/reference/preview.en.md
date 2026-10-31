---
title: "Formula and Reference Sheet"
subtitle: "Symbols, formulas, units and pitfalls"
course: "Folisift"
language: en
sources: [PD0]
source_scope: "PD0:14-15"
running: "Reference Sheet"
---

::: front kicker="REFERENCE" title="Reference Sheet" lead="Lookup order: symbol, formula, unit, trap."
:::

::: toc title="Contents" note="Page numbers refer to this PDF."
:::

## Symbols

::: table caption="Core symbols" source="PD0, p. 14" widths="24,46,30"
| Symbol | Meaning | Unit |
|---|---|---|
| `A`, `B`, `C` | atoms | — |
| `(A B C)` | list | — |
| `quote` | preserve as data | — |
:::

## Formulas

::: formula title="List reading" source="PD0, p. 14"
A list `L = (x1 x2 ... xn)` has `n` top-level elements. A nested list counts as one element at the parent level.
:::

::: formula title="Evaluation rule" source="PD0, p. 15"
`(F A1 A2)` evaluates `F` as a function and applies it to the evaluated arguments. `(quote X)` returns `X` without evaluation.
:::

## Units

::: table caption="Notation conventions" source="PD0, p. 14" widths="30,70"
| Form | Reading |
|---|---|
| `(A B C)` | data list or function application |
| `(quote X)` | data |
| `(eval X)` | code |
:::

## Pitfalls

::: critical label="EXAM TRAP"
Do not say “everything is the same” in Lisp. The representation is shared; evaluation is context-dependent. [PD0, p. 15]
:::
