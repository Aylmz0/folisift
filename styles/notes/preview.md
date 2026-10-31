---
title: "Rehberli Notlar"
subtitle: "Cornell tarzı ipucu ve not örneği"
course: "Folisift"
language: tr
sources: [PD0]
source_scope: "PD0:14-15"
running: "Rehberli Notlar"
---

::: front kicker="ÖRNEK" title="Rehberli Notlar" lead="Solda ipuçları, sağda notlar, sonda özet."
:::

::: toc title="İÇİNDEKİLER" note="Sayfa numaraları bu PDF’e aittir."
:::

## Öğrenme hedefleri

- Lisp listesinde veri ile kodu ayır.
- `quote` ve `eval` kavramlarını birer örnekle açıkla.

## Ana kavramlar

::: cue label="Liste nedir?"
Liste, atomların ve alt listelerin parantezli dizisidir. `(A B C)` üç öğe taşır. [PD0, s. 14]
:::

::: cue label="Veri mi, kod mu?"
Aynı liste hem veri hem fonksiyon uygulaması olarak okunabilir. Okuma biçimini operatör konumu belirler. [PD0, s. 14]
:::

::: cue label="quote neden?"
`quote`, ifadeyi değerlendirmeden veri olarak korur. [PD0, s. 15]
:::

## Örnekler

::: example label="ÇÖZÜMLÜ ÖRNEK"
`(A B C)` ifadesini veri olarak oku: A, B ve C öğedir. Kod olarak oku: `A` fonksiyonu `B` ve `C` argümanlarına uygulanır. [PD0, s. 14]
:::

::: cue label="Sıra sende"
Temsil ile değerlendirme arasındaki farkı bir cümleyle yaz.
:::

::: workspace label="NOTLAR" lines=3
:::

## Özet

::: summary
Temsil ortaktır; değerlendirme bağlama göre değişir. Veri için `quote`, kod için `eval` kullan. [PD0, s. 15]
:::
