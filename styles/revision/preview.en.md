---
title: "Revision Sheet"
subtitle: "Visual study sample"
course: "Folisift"
language: en
sources: [PD0]
running: "Revision Sheet"
---

::: front kicker="SAMPLE" title="Revision Sheet" lead="Short definitions, comparison, recall and a one-page synthesis."
:::

## 1. Scope and map

This sample shows the pre-exam review page for one source section. There are no
long paragraphs; each block does one job.

::: definition label="MUST KNOW"
When code and data share one representation, the program itself becomes
processable data. [PD0, p. 12]
:::

## 2. Must-know

::: must-know
- **Definition:** The first element of a list is consed onto the rest. [PD0, p. 13]
- **Distinction:** A list as data versus a function application as code. [PD0, p. 14]
- **Condition:** Evaluation order depends on the operator type. [PD0, p. 15]
- **Trap:** `quote` and `eval` are not the same operation. [PD0, p. 16]
:::

## 3. Comparisons

::: table caption="Data and code readings" source="PD0, p. 14"
| Reading | Input | Result |
|---|---|---|
| Data | `(A B C)` | A three-element list |
| Code | `(A B C)` | Function `A` applied to `B` and `C` |
:::

## 4. Exam traps

::: critical label="EXAM TRAP"
Do not treat the shared representation as a claim that evaluation is also shared. [PD0, p. 14]
:::

## 5. Recall

::: recall
How is `(A B C)` read as data and as code?
:::

::: answer label="ANSWER"
As data it is a three-element list; as code it applies function `A` to arguments
`B` and `C`. [PD0, p. 14]
:::

::: pagebreak

## 6. One-page synthesis

::: synthesis title="One-page synthesis"
- Code and data share one representation.
- `quote` preserves data; `eval` evaluates it.
- A list contains atoms and sublists.
- Function application starts with the operator on the left.
- In an exam, separate the data reading from the code reading first.
:::

## 7. Review plan

::: review-plan
Re-read the recall answers, redraw the map, then solve one faded example without looking at the answer key. [PD0, p. 14]
:::
