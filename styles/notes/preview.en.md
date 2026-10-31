---
title: "Guided Notes"
subtitle: "Cornell-style cue and notes sample"
course: "Folisift"
language: en
sources: [PD0]
source_scope: "PD0:14-15"
running: "Guided Notes"
---

::: front kicker="SAMPLE" title="Guided Notes" lead="Cues on the left, notes on the right, summary at the end."
:::

::: toc title="Contents" note="Page numbers refer to this PDF."
:::

## Learning goals

- Separate data from code in a Lisp list.
- Explain `quote` and `eval` with one example each.

## Key concepts

::: cue label="What is a list?"
A list is a parenthesized sequence of atoms and sublists. `(A B C)` has three elements. [PD0, p. 14]
:::

::: cue label="Data or code?"
The same list can be read as data or as a function application. The operator position decides the reading. [PD0, p. 14]
:::

::: cue label="Why quote?"
`quote` preserves an expression as data without evaluating it. [PD0, p. 15]
:::

## Examples

::: example label="WORKED EXAMPLE"
Read `(A B C)` as data: A, B and C are elements. Read it as code: function `A` is applied to `B` and `C`. [PD0, p. 14]
:::

::: cue label="Your turn"
Write one sentence that distinguishes representation from evaluation.
:::

::: workspace label="NOTES" lines=3
:::

## Summary

::: summary
Representation is shared; evaluation depends on context. Use `quote` for data and `eval` for code. [PD0, p. 15]
:::
