---
title: "Practice Workbook"
subtitle: "Problem, workspace and answer key"
course: "Folisift"
language: en
sources: [PD0]
source_scope: "PD0:14"
running: "Practice Workbook"
---

::: front kicker="PRACTICE" title="Practice Workbook" lead="Solve first, then check the answer key."
:::

::: toc title="Contents" note="Page numbers refer to this PDF."
:::

## Problems

::: problem title="Problem 1"
Read `(A (B C))` as data. How many top-level elements are there?
:::

::: workspace label="YOUR WORKSPACE" lines=4
:::

::: problem title="Problem 2"
Explain why `quote` prevents evaluation. Use one example.
:::

::: workspace label="YOUR WORKSPACE" lines=4
:::

## Worked example

::: example label="FADED EXAMPLE"
For `(A (B C))`, the top level has two elements: `A` and the sublist `(B C)`. The sublist has two elements. Finish the sentence: the outer list has ___ elements. [PD0, p. 14]
:::

## Practice

::: problem title="Problem 3"
Mark each statement true or false: `(A B)` is a function application when `A` is the operator.
:::

::: workspace label="YOUR WORKSPACE" lines=3
:::

## Answer key

::: answer-key
**Problem 1:** two top-level elements. **Problem 2:** `quote` returns the expression as data; e.g. `(quote (A B))` yields the list. **Problem 3:** true. [PD0, p. 14]
:::
