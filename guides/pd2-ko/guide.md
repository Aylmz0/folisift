---
title: "LISP — Bilgi Organizatörü"
subtitle: "Tek sayfa"
course: "BMÜ325 Programlama Dilleri"
language: tr
sources: [PD2]
source_scope: "PD2:26-29"
running: "LISP Bilgi Organizatörü"
---

::: organiser title="LISP" source="PD2, s. 26–29"

## Büyük sorular
- LISP neden listeler üzerine kuruldu?
- Kod ve veri neden aynı temsili paylaşır?
- `quote` ile `eval` neyi değiştirir?
- LISP’in iki veri tipi neydi?
- İç içe liste nasıl okunur?

## Anahtar sözcükler
- **LISP:** LISt Processing; McCarthy, MIT, IBM 704. [PD2, s. 26]
- **Atom:** Bölünemeyen tek birim. [PD2, s. 26]
- **Liste:** Atomların ve alt listelerin parantezli dizisi. [PD2, s. 26]
- **Lambda kalkülüs:** LISP sözdiziminin dayandığı model. [PD2, s. 26]
- **Kod = veri:** Aynı liste hem program hem veri olabilir. [PD2, s. 27]
- **quote:** İfadeyi değerlendirmeden korur. [PD2, s. 27]
- **eval:** İfadeyi kod olarak değerlendirir. [PD2, s. 27]

## Kronoloji
- 1958–1960: McCarthy’nin LISP çalışması. [PD2, s. 26]
- 1960’lar: MIT’te IBM 704 üzerinde geliştirme. [PD2, s. 26]

## Harita veya şema
- LISP → Liste
- Liste → Atom
- Liste → Alt liste
- Liste → Kod
- Liste → Veri
- quote → Veri
- eval → Kod

## Kritik ayrımlar
- **Atom vs liste:** atom bölünemez; liste öğeler taşır.
- **Veri vs kod:** aynı temsil, farklı değerlendirme.
- **quote vs eval:** koruma vs değerlendirme.
- **Düz liste vs iç içe liste:** öğe sayısı ve derinlik değişir.

:::
