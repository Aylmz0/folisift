---
title: "LISP Revizyon Sayfası"
subtitle: "Sınav öncesi kısa tekrar"
course: "BMÜ325 Programlama Dilleri"
language: tr
sources: [PD2]
running: "LISP Revizyon Sayfası"
cover_notes: ["Kaynak: PD2, s. 26–29", "Folisift revizyon biçimi"]
---

::: front kicker="REVİZYON" title="LISP: kod ve veri" lead="Tek sayfada tanımlar, karşılaştırma, tuzaklar ve hatırlama."
:::

## Kapsam ve harita

Bu sayfa PD2’nin LISP bölümünü sınav öncesi tekrar için özetler. Kaynak
aralığı: PD2, s. 26–29. Harita, kavramlar arasındaki ilişkiyi gösterir; tek
başına okunmaz, “haritayı kapat ve yeniden çiz” sorusuyla birlikte kullanılır.

::: diagram id="lisp-map"
:::

::: must-know
- **LISP:** LISt Processing; McCarthy tarafından MIT’te IBM 704 için tasarlandı. [PD2, s. 26]
- **İki veri tipi:** atomlar ve listeler. [PD2, s. 26]
- **Sözdizimi:** lambda kalkülüsüne dayanır. [PD2, s. 26]
- **Kod = veri:** program kodu ile veri aynı liste temsilini paylaşır. [PD2, s. 27]
- **quote:** ifadeyi değerlendirmeden veri olarak korur. [PD2, s. 27]
- **eval:** ifadeyi kod olarak değerlendirir. [PD2, s. 27]
:::

## Bilmen gerekenler

::: definition label="TEMEL TANIM"
LISP’te liste, atomların ve alt listelerin parantezli dizisidir. `(A B C D)`
düz liste; `(A (B C) D (E (F G)))` iç içe listedir. [PD2, s. 29]
:::

::: critical label="KRİTİK AYRIM"
Aynı `(A B C D)` metni iki farklı okuma üretir: **veri** olarak A, B, C, D
elemanlarını taşıyan liste; **kod** olarak A fonksiyonunun B, C, D
argümanlarına uygulanması. [PD2, s. 27]
:::

## Karşılaştırma

::: table caption="Aynı liste, iki okuma" source="PD2, s. 27"
| Okuma | `(A B C D)` ne demek? | Sonuç |
|---|---|---|
| Veri | A, B, C, D elemanlı liste | Liste veri olarak kullanılır |
| Kod | A fonksiyonu, B C D argümanları | Fonksiyon uygulanır |
:::

::: table caption="LISP’in iki veri tipi" source="PD2, s. 26"
| Tip | Örnek | Rol |
|---|---|---|
| Atom | `A`, `42` | Bölünemeyen tek birim |
| Liste | `(A B C)` | Atomları ve alt listeleri taşır |
:::

## Çözümlü örnekler

::: example label="ÇÖZÜMLÜ ÖRNEK"
`(A (B C) D (E (F G)))` ifadesini oku.

1. En dıştaki liste dört öğe taşır: `A`, `(B C)`, `D`, `(E (F G))`.
2. `(B C)` bir alt listedir; `B` ve `C` onun atomlarıdır.
3. `(E (F G))` içinde `E` atomu ve `(F G)` alt listesi vardır.
4. Sonuç: dış liste 4 öğeli, iç içe iki alt liste vardır. [PD2, s. 29]
:::

::: answer label="ŞİMDİ SEN YAP"
`(A B C D)` ifadesini veri ve kod olarak iki cümlede açıkla.
:::

## Sınav tuzakları

::: critical label="SINAV TUZAĞI"
“Kod = veri” cümlesini “LISP’te her şey aynıdır” diye genelleştirme. Doğrusu:
**temsil** ortaktır; **değerlendirme** bağlama göre değişir. `quote` veriyi
korur, `eval` değerlendirir. [PD2, s. 27]
:::

::: critical label="SINAV TUZAĞI"
Liste ile atomu karıştırma. `(A B)` liste; `A` ve `B` atomdur. İç içe liste
`(A (B C))` iki öğelidir; üç öğeli sanma. [PD2, s. 29]
:::

::: critical label="SINAV TUZAĞI"
LISP’in yalnızca iki veri tipi olduğunu unutma: atom ve liste. Bu, “iki veri
yapısı” değil, iki temel temsil türüdür. [PD2, s. 26]
:::

::: pagebreak

## Hatırlama

::: recall
LISP’i kim, nerede, hangi makine için tasarladı?
:::

::: recall
LISP’in iki veri tipi nedir?
:::

::: recall
`(A B C D)` veri olarak ve kod olarak nasıl okunur?
:::

::: recall
`(A (B C) D (E (F G)))` dış listesinde kaç öğe vardır?
:::

::: recall
`quote` ile `eval` arasındaki farkı bir cümlede açıkla.
:::

### Cevaplar

::: answer label="CEVAP 1"
McCarthy tarafından MIT’te IBM 704 için tasarlandı. [PD2, s. 26]
:::

::: answer label="CEVAP 2"
Atomlar ve listeler. [PD2, s. 26]
:::

::: answer label="CEVAP 3"
Veri olarak A, B, C, D elemanlı listedir; kod olarak A fonksiyonunun B, C, D
argümanlarına uygulanmasıdır. [PD2, s. 27]
:::

::: answer label="CEVAP 4"
Dört öğe: `A`, `(B C)`, `D`, `(E (F G))`. [PD2, s. 29]
:::

::: answer label="CEVAP 5"
`quote` ifadeyi değerlendirmeden veri olarak korur; `eval` ifadeyi kod olarak
değerlendirir. [PD2, s. 27]
:::

::: pagebreak

## Tek sayfa sentez

::: synthesis title="LISP: tek sayfa sentez"
- LISP = LISt Processing; McCarthy, MIT, IBM 704.
- İki temel temsil: atom ve liste.
- Liste, atomları ve alt listeleri taşır.
- Kod ve veri aynı liste temsilini paylaşır.
- `quote` veriyi korur; `eval` değerlendirir.
- Sınavda önce “veri mi, kod mu?” sorusunu ayır.
:::

## Tekrar planı

::: review-plan
- **24 saat:** Haritayı kapat, LISP’in iki veri tipini ve kod/veri ayrımını yaz.
- **3 gün:** `(A (B C) D (E (F G)))` için dış/iç öğe sayısını yeniden çıkar.
- **7 gün:** Hatırlama sorularını cevapsız çöz; sonra cevap anahtarıyla karşılaştır.
- **14 gün:** Karışık sette LISP ile başka bir dil ailesini karşılaştır.
:::
