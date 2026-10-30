---
title: "Revizyon Sayfası"
subtitle: "Görsel çalışma örneği"
course: "Folisift"
language: tr
sources: [PD0]
running: "Revizyon Sayfası"
---

::: front kicker="ÖRNEK" title="Revizyon Sayfası" lead="Kısa tanımlar, karşılaştırma, hatırlama ve tek sayfa sentez."
:::

## 1. Kapsam ve harita

Bu örnek, bir kaynak bölümünün sınav öncesi tekrar sayfasını gösterir. Uzun
paragraf yok; her blok tek bir iş yapar.

::: definition label="BİLİNMESİ GEREKEN"
Kod ve veri aynı temsili paylaştığında, programın kendisi de işlenebilir bir
veri olur. [PD0, s. 12]
:::

## 2. Bilmen gerekenler

::: must-know
- **Tanım:** Bir listenin ilk elemanı, kalan listenin önüne eklenir. [PD0, s. 13]
- **Ayrım:** Veri olarak liste, kod olarak fonksiyon uygulaması. [PD0, s. 14]
- **Koşul:** Değerlendirme sırası, operatörün türüne bağlıdır. [PD0, s. 15]
- **Tuzak:** `quote` ile `eval` aynı şey değildir. [PD0, s. 16]
:::

## 3. Karşılaştırma

::: table caption="Veri ve kod yorumu" source="PD0, s. 14"
| Bakış | Girdi | Sonuç |
|---|---|---|
| Veri | `(A B C)` | Üç elemanlı liste |
| Kod | `(A B C)` | `A` fonksiyonu, `B C` argümanları |
:::

## 4. Hatırlama

::: recall
`(A B C)` ifadesi veri olarak ve kod olarak nasıl yorumlanır?
:::

::: answer label="CEVAP"
Veri olarak üç elemanlı listedir; kod olarak `A` fonksiyonunun `B` ve `C`
argümanlarına uygulanmasıdır. [PD0, s. 14]
:::

::: pagebreak

## 5. Tek sayfa sentez

::: synthesis title="Tek sayfa sentez"
- Kod ve veri aynı temsili paylaşır.
- `quote` veriyi korur; `eval` değerlendirir.
- Liste, atom ve alt listelerden oluşur.
- Fonksiyon uygulaması soldaki operatörle başlar.
- Sınavda önce veri/kod yorumunu ayır.
:::
