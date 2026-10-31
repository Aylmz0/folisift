---
title: "Formül ve Başvuru Sayfası"
subtitle: "Semboller, formüller, birimler ve tuzaklar"
course: "Folisift"
language: tr
sources: [PD0]
source_scope: "PD0:14-15"
running: "Başvuru Sayfası"
---

::: front kicker="BAŞVURU" title="Başvuru Sayfası" lead="Arama sırası: sembol, formül, birim, tuzak."
:::

::: toc title="İÇİNDEKİLER" note="Sayfa numaraları bu PDF’e aittir."
:::

## Semboller

::: table caption="Temel semboller" source="PD0, s. 14" widths="24,46,30"
| Sembol | Anlam | Birim |
|---|---|---|
| `A`, `B`, `C` | atomlar | — |
| `(A B C)` | liste | — |
| `quote` | veri olarak koru | — |
:::

## Formüller

::: formula title="Liste okuma" source="PD0, s. 14"
`L = (x1 x2 ... xn)` listesinin üst düzeyinde `n` öğe vardır. İç içe liste, üst düzeyde tek öğe sayılır.
:::

::: formula title="Değerlendirme kuralı" source="PD0, s. 15"
`(F A1 A2)` ifadesi `F` fonksiyonunu değerlendirir ve değerlendirilen argümanlara uygular. `(quote X)` ifadesi `X` değerini değerlendirmeden döndürür.
:::

## Birimler

::: table caption="Gösterim kuralları" source="PD0, s. 14" widths="30,70"
| Biçim | Okuma |
|---|---|
| `(A B C)` | veri listesi veya fonksiyon uygulaması |
| `(quote X)` | veri |
| `(eval X)` | kod |
:::

## Tuzaklar

::: critical label="SINAV TUZAĞI"
Lisp’te “her şey aynıdır” deme. Temsil ortaktır; değerlendirme bağlama göre değişir. [PD0, s. 15]
:::
