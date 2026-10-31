---
title: "Alıştırma Kitabı"
subtitle: "Soru, çalışma alanı ve cevap anahtarı"
course: "Folisift"
language: tr
sources: [PD0]
source_scope: "PD0:14"
running: "Alıştırma Kitabı"
---

::: front kicker="ALIŞTIRMA" title="Alıştırma Kitabı" lead="Önce çöz, sonra cevap anahtarını kontrol et."
:::

::: toc title="İÇİNDEKİLER" note="Sayfa numaraları bu PDF’e aittir."
:::

## Sorular

::: problem title="Soru 1"
`(A (B C))` ifadesini veri olarak oku. Üst düzeyde kaç öğe vardır?
:::

::: workspace label="ÇALIŞMA ALANI" lines=4
:::

::: problem title="Soru 2"
`quote` neden değerlendirmeyi engeller? Bir örnekle açıkla.
:::

::: workspace label="ÇALIŞMA ALANI" lines=4
:::

## Çözümlü örnek

::: example label="KADEMELİ ÖRNEK"
`(A (B C))` için üst düzey iki öğe vardır: `A` ve `(B C)` alt listesi. Alt liste iki öğe taşır. Cümleyi tamamla: dış liste ___ öğelidir. [PD0, s. 14]
:::

## Alıştırma

::: problem title="Soru 3"
Şu ifadeyi doğru/yanlış işaretle: `A` operatörse `(A B)` bir fonksiyon uygulamasıdır.
:::

::: workspace label="ÇALIŞMA ALANI" lines=3
:::

## Cevap anahtarı

::: answer-key
**Soru 1:** iki üst düzey öğe. **Soru 2:** `quote`, ifadeyi veri olarak döndürür; örneğin `(quote (A B))` listeyi verir. **Soru 3:** doğru. [PD0, s. 14]
:::
