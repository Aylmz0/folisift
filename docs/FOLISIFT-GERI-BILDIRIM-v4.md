# Folisift 0.22.0 — Geri Bildirim v4 (PDF + sistem değerlendirmesi)

**Tarih:** 5 Kasım 2026
**Sürüm:** Folisift **0.22.0** · build **230** · MCP schema **17** · **41 tool**
**Önceki rapor:** `FOLISIFT-GERI-BILDIRIM-v3.md` (0.20.0, 9.2/10)
**İnceleme kapsamı:** `PD0-PD1_Calisma_Ozeti.pdf` (ana çıktı) + PD2/PD3 çıktılarından örnek sayfalar + taze CLI/MCP testleri
**Durum:** Final — PDF görsel incelemesi, QA ve kabul testleri tamamlandı

---

## 0. Kısa hüküm

**PDF tarafı güçlü:** 23 sayfa, hedef 23; A4, tagged, kitapçık metadata’sı doğru;
içindekiler sayfa numaraları tutarlı; gövde tipografisi ve tablolar dengeli;
sözlük artık tek dört sütunlu tablo ve gövde tablolarıyla aynı font/leading
ailesinde. D2 diyagramları (PD2 s. 8, PD3 s. 7) temiz ve kaynak atıflı.

**Sistem tarafı gerçekten ilerledi:** `visual_candidates` artık bbox bazında
aday üretiyor; native görsel / adaptif crop / D2 concept-schematic / unique
ayrımı çalışıyor; `visual_decision` asset + audit + Markdown üretiyor;
`visual_assets` check/materialize/refresh yapıyor. Manuel v1 kalibrasyonu
data-mining 14/45/71, PD0 1/2/7/9, PD2 5/29/53 ve PD3 76 üzerinde tamamlandı.

**Kalan en önemli sorun P1:** eski `visual_gate` hâlâ `data/blocks` içindeki ham
`figure`/`table` bloklarını sayıyor. PD0’da 82, PD2’de 28, PD3’te 136
figure-page görünüyor; PD3’te bu tüm sayfalara yakın. `visual_gate="warn"`
olduğu için build geçiyor, ama QA çıktısında `missing_figures` 40/24/118 ve
`passed:true` birlikte görünüyor. Bu, “görsel kapsam” raporunun güvenilirliğini
düşürüyor; `required` moda geçen bir guide yüzlerce yanlış sayfa yüzünden
kilitlenir. Yeni aday pipeline’ı bu gate’e bağlanmalı.

**Puan: 9.1 / 10** (v3: 9.2)

v3’e göre artılar: gerçek görsel aday pipeline’ı, native/crop ayrımı, kalite
kapısı, audit ve MCP araçları. Eksi: gate truth P1’i ve sayfa doluluk/ürün
metni cilası. Aşağıdaki §3 ve §4 bunu ayrıştırıyor.

---

## 0.1 Ortam ve yöntem

- Folisift 0.22.0, build 230, MCP schema 17, 41 tool
- PDFium `chromium/8058`; D2 v0.9.0 (`doctor` pass)
- Ana PDF: `output/PD0-PD1_Calisma_Ozeti.pdf` — 23 sayfa, 538.621 bayt
- Ek PDF’ler: `PD2_Calisma_Ozeti.pdf` (24 s.), `PD3_Calisma_Ozeti.pdf` (21 s.)
- PDF incelemesi: `pdfinfo`, `pdftotext`, 12 sayfanın 110 DPI render’ı ve
  doğrudan görsel inceleme (kapak, TOC, s. 2/4/10/11/12/15/17/18/19/20/21/22/23)
- QA: `qa --guide pd0-pd1`, `pd2`, `pd3`
- Kabul: `make acceptance` (21 kontrol), `make visuals-v5`
- MCP: gerçek stdio çağrılarıyla `visual_candidates` → `visual_decision` →
  `visual_assets check`
- Hata yolları: geçersiz decision, eksik `image_id`, düşük çözünürlüklü asset,
  silinmiş asset, `materialize`, `refresh`

---

## 0.2 Test matrisi

| ID | Test | Durum | Kanıt |
|---|---|---|---|
| T01 | `make acceptance` | **PASS** | 21/21 true |
| T02 | `make visuals-v5` | **PASS** | 13/13 A-V5 |
| T03 | PD0-PD1 QA | **PASS** | 23 s., coverage 13/13, audit pass |
| T04 | PD2 QA | **PASS** | 24 s., coverage 20/20, audit pass |
| T05 | PD3 QA | **PASS** | 21 s., coverage 5/5, audit pass |
| T06 | PDF metadata/tagged | **PASS** | A4, 23 s., PDF 1.7, tagged |
| T07 | TOC ve sayfa referansları | **PASS** | TOC numaraları gövdeyle uyumlu |
| T08 | Sözlük tek tablo | **PASS** | 4 sütun, grup satırları, gövde fontu |
| T09 | D2 diyagramları | **PASS** | PD2 s. 8, PD3 s. 7 temiz |
| T10 | Data-mining adayları | **PASS** | 17 aday; top-5 tamamı core |
| T11 | PD2 D2/karikatür ayrımı | **PASS** | ağaç `d2_concept`; karikatür `decorative` |
| T12 | PD0 kapak guard’ı | **PASS** | s. 1 figürleri `decorative/skip` |
| T13 | PD3 76 fragment merge | **PASS** | 31 parça → tek `redraw` |
| T14 | Native extract + kalite | **PASS** | düşük DPI asset reddedildi |
| T15 | Crop decision + audit | **PASS** | asset + audit + Markdown |
| T16 | Assets check/materialize/refresh | **PASS** | missing→restore→ok |
| T17 | Geçersiz decision/eksik image_id | **PASS** | exit 1, audit yazılmadı |
| T18 | MCP tool sözleşmesi | **PASS** | schema 17, 41 tool |
| T19 | `qa.visual` gate truth | **FAIL (P1)** | PD0 82 figure-page / 40 missing; PD3 136 / 118 |
| T20 | Sayfa doluluk / ürün metni | **PARTIAL (P2)** | s. 2/10/17/22’de geniş boşluk; “PD2–PD8” roadmap metni |
| T21 | Kalibrasyon istatistiği | **PARTIAL (P2)** | v1 manuel; precision/recall otomatik değil |

---

## 1. PDF incelemesi

### 1.1 Metadata ve yapı

```text
Title: Programlama Dilleri — Çalışma Kılavuzu
Author: Folisift
Pages: 23
Page size: A4 (595.276 × 841.89 pt)
Tagged: yes
PDF version: 1.7
File size: 538,621 bytes
```

Hedef `target_pages: 23`; çıktı 23. Kapak, ön not, TOC, iki bölüm, C–G ortak
bölümleri ve kapsam denetimi beklenen sırada. TOC numaraları gövdeyle uyumlu.

### 1.2 Kapak ve ön kısım

- Kapak sade, kurumsal ve “Pilot sürüm · PD0–PD1” etiketiyle dürüst.
- “Bu kitapçık hakkında” sayfası kapsam, yöntem, doğruluk ve sürüm bilgisini
  veriyor; kaynak sayısı ve alt bölüm sayısı net.
- Kısaltmalar ve dil notu okuyucuya gerçek değer katıyor.

### 1.3 Gövde ve tipografi

- Gövde 9.5 pt, tablo 7.9–8.6 pt bandında; satır uzunluğu p50 44, p90 82
  karakter. Bu, tek sütun A4 için kabul edilebilir.
- Başlık hiyerarşisi (bölüm → alt bölüm → kutu) tutarlı.
- `Kritik ayrım` ve `Açıklayıcı not` kutuları görsel olarak ayrışıyor; kaynak
  referansları küçük caps ve tutarlı.
- Sayfa 4, 11, 12, 15, 17, 20, 21, 22, 23’te taşma, kesilmiş tablo veya
  okunamayan küçük font yok; `audit` pass.

### 1.4 Tablolar

- s. 12 “programlama alanları” ve “dil değerlendirme ölçütleri” tabloları
  dengeli; satır yüksekliği ve zebra okunabilir.
- s. 18–19 sözlük artık **tek dört sütunlu tablo**. `###` grup başlıkları
  tam genişlikte pale-sage satır; font ve leading gövde tablolarıyla aynı.
  Önceki “itiş tıkış” görünüm gerçekten düzelmiş.
- s. 19’da ikinci tablo (Sık Karıştırılan Ayrımlar) sayfa başında
  `Markup / SDLC` satırıyla devam ediyor; satır önceki sayfadan taşınmış ama
  grup başlığı tekrar etmiyor. Küçük ama gerçek bir okuma kopukluğu.
- s. 20 D.1/D.4 tabloları ve s. 21 E.2 olgu tablosu temiz.

### 1.5 Sözlük ve D2

- Sözlük: 2 sayfa, 4 sütun, tek tablo, grup satırları; **PASS**.
- PD2 s. 8: “Düz liste / İç içe liste” D2 şeması, caption ve kaynak atıflı;
  kutular ve bağlantılar net.
- PD3 s. 7: atama ağacı D2 şeması, kod blokları ve caption uyumlu.
- PD0-PD1 s. 10: SDLC beş aşama şeması; caption “Şekil 0.4”; kaynak atıflı.

### 1.6 Sayfa doluluğu

`audit` underfilled sayfa raporlamıyor; ancak görsel incelemede:

- s. 2 (ön not) alt ~1/3 boş,
- s. 10 (bölüm özeti) alt ~%40 boş,
- s. 17 (bölüm sonu) alt ~%30 boş,
- s. 22 (F.2 roadmap) alt ~%30 boş.

Bunlar hata değil; bölüm sonu ritmi. Yine de “kitap hissi” için editöryel
doluluk (açıklayıcı not, kısa örnek, recall sorusu) eklenebilir. `audit`
underfilled eşiği bunu yakalamıyor; ayrı bir “page fill” metriği faydalı olur.

### 1.7 Okuyucuya görünen ürün metni

s. 2: “PD2–PD8 bölümleri aynı şablonla eklenecek.”
s. 22: “PD3–PD8 sonraki bölümlerle bağlantı” listesi.

Bunlar pilot ekip içi yol haritası gibi duruyor; final okuyucu için ya
“Pilotun sınırı” kutusunda kalmalı ya da kaldırılmalı. İçerik hatası değil,
ürün dili/editoryal sızıntı.

---

## 2. Yeni görsel pipeline değerlendirmesi

### 2.1 Aday sonuçları

`visual_candidates` ile taze çağrılar:

| Kaynak | Sonuç |
|---|---|
| `verimadenciliği` 14/45/71 | 17 aday; top-5 tamamı `core`; workflow `d2_concept/redraw`; tablo `native_table`; skewness grubu `group_members=4/describe`; scatter/ROC `unique/describe`; metin slaytları `text/skip` |
| PD2 5/29/53 | 4 aday; p53 89 `d2_concept/redraw`; p29 72 `d2_concept/redraw`; p5 ağaç 62 `d2_concept/redraw`; p5 karikatür 20 `decorative/skip` |
| PD0 1–10 | 11 aday; p9 “Mimar” 80 `d2_concept/redraw`; p7 TIOBE 79 `unique/describe`; p1 kapakları 20 `decorative/skip`; p2 metin 45 `text/skip` |
| PD3 76 | Tek aday; 31 parça birleşti; 62 `d2_concept/redraw` |

Bu, v0.20.0’daki “sayfa bazlı chart_likely” sorununu gerçekten çözüyor.
Özellikle PD2 karikatürünün 95’ten 20’ye düşmesi ve PD0 kapaklarının
temizlenmesi manuel kalibrasyonun karşılığı.

### 2.2 Native / crop / kalite

- Native extraction: `p76_1` gibi düşük çözünürlüklü asset’ler 128 px/75 DPI
  olduğu için `visual_decision embed_image` tarafından reddedildi.
- Crop: data-mining s. 45 workflow 995×756, 310 DPI, hash’li PNG üretildi;
  caption/alt/source zorunlu.
- `visual_assets check` asset silinince exit 1; `materialize` aynı hash’i geri
  getirdi; `refresh` hash/quality/source digest’i güncelledi.
- Hatalı karar, eksik `image_id`, geçersiz `redraw_kind` artık audit’e
  yazılmadan reddediliyor.

### 2.3 D2 / unique sınırı

- Sözdizim grafiği, dil soyağacı, LISP gösterimi, sınıflandırma akışı:
  `d2_concept/redraw` — doğru.
- ROC, scatter, quantile, TIOBE tablosu: `unique/describe` — doğru.
- `DBSCAN/K-means/Max` algoritma karşılaştırması artık `unique`; D2 schematic
  yalnızca nitel küme/daire konsepti için host seçeneği. Bu, “D2 her grafiği
  çizer” yanılgısını kapatıyor.

### 2.4 Kalibrasyon sınırı

v1 manuel inceleme tamamlandı; ancak precision/recall istatistiği yok. Bilinen
açıklar:

- s. 14 skewness panellerinin caption seed’i zayıf (“ve mod değerleri”);
  grup doğru ama caption’ı host yazmalı.
- PD0 s. 7 TIOBE tablosu görseli `describe`; host isterse tablo çıkarımı/crop
  seçebilir.
- `visual_brief` hâlâ sayfa özeti; aday listesi için `visual_candidates`
  ayrı çağrı.

Bunlar P2; ana karar akışını bozmuyor.

---

## 3. Bulgular

### F-V4-01 — P1 — `qa.visual` figure-page gate ham layout bloklarını sayıyor

**Kanıt (taze `qa`):**

| Guide | figure_pages | missing_figures | low_text | entries | gate | passed |
|---|---:|---:|---:|---:|---|---|
| pd0-pd1 | 82 | 40 | 55 | 55 | warn | true |
| pd2 | 28 | 24 | 9 | 9 | warn | true |
| pd3 | 136 | 118 | 18 | 18 | warn | true |

PD3’te 136 figure-page, 136 sayfalık kaynağın neredeyse tamamı; gerçek figure
caption sayısı 5. PD0’da PDF’de 4 figure caption var, gate 82 sayfa görüyor.
`warn` modu doğru davranıp build’i düşürmüyor, ama `passed:true` ile
`missing_figures` aynı raporda yer alınca “görsel kapsam geçti” okunuyor.

**Etki:** `visual_gate=required` yapan bir guide, yüzlerce yanlış figure-page
yüzünden kullanılamaz. Yeni `visual_candidates` pipeline’ı bu gate’e bağlı
değil; iki görsel sistem yan yana yaşıyor.

**Öneri:** `figure_pages`’i ham `figure`/`table` bloklarından değil,
`visual_candidates` çıktısından türet. `visual candidates --write` ile
`data/visual-candidates.json` üret; source digest + scorer version ile cache’le.
Gate yalnızca `score >= eşik` veya `relevance=core` adaylarını beklesin.
`qa.visual` alanına `status: advisory|required|passed|failed` ekle; `warn`
modunda `passed:true` yerine `advisory:true` yaz.

### F-V4-02 — P2 — Sayfa doluluğu ve bölüm sonu ritmi

s. 2, 10, 17, 22’de geniş boşluk var. `audit` underfilled demiyor; ama görsel
olarak “kitap” hissini zayıflatıyor. Öneri: bölüm sonlarına kısa bir
“kendini sına” sorusu, karşılaştırma veya kaynak notu; ya da bilinçli
`intentional_pages` ile işaretle.

### F-V4-03 — P2 — Okuyucuya görünen roadmap/ürün metni

s. 2 “PD2–PD8 eklenecek” ve s. 22 “sonraki bölümler” listesi final okuyucu
için iç metin. “Pilotun sınırı” kutusunda tutulmalı veya kaldırılmalı.

### F-V4-04 — P2 — s. 19 tablo devamında grup başlığı yok

“Sık Karıştırılan Ayrımlar” tablosu sayfa başında `Markup / SDLC` satırıyla
devam ediyor; grup başlığı tekrar etmiyor. Tablo `header`’ı sütun başlığını
tekrarlıyor ama grup satırını tekrarlamıyor. Küçük okuma kopukluğu.

### F-V4-05 — P2 — Kalibrasyon v1 ve örneklem

Manuel kalibrasyon tamamlandı; ancak top-5 precision/top-10 recall
istatistiksel olarak ölçülmedi. PD0/PD1 dış örneklem olarak ayrılmadı.
Bir sonraki turda etiketli set + confusion matrix gerekir.

### F-V4-06 — P3 — `visual_brief` aday listesi döndürmüyor

Sayfa rolü/önerisi var; bbox adayları yok. Host iki çağrı yapıyor
(`visual_brief` + `visual_candidates`). `visual_brief`’e `top_candidates`
eklenebilir; maliyet için opsiyonel olmalı.

### F-V4-07 — P3 — Gerçek guide’da native/crop örneği yok

Mekanizma smoke testte ve data-mining’de çalışıyor; shipped PD0–PD3
guide’larında `embedded_crops=0`, `embedded_images=0`. D2 redraw tercih
edilmiş; bu doğru olabilir, ama “gerçek guide’da kaynak görsel” kanıtı için
bir örnek eklenmeli.

### F-V4-08 — P3 — `visual_assets check` source digest’i raporlamıyor

`refresh` digest’i güncelliyor; `check` yalnızca asset hash’ine bakıyor.
Build `VISUAL_SOURCE_CHANGED` uyarısı veriyor; `assets check` de aynı bilgiyi
döndürmeli.

---

## 4. Puanlama

| Alan | Puan | Gerekçe |
|---|---:|---|
| İçerik doğruluğu | 9.5 | Kaynak referansları, açıklayıcı notlar, tablolar güçlü |
| PDF tasarımı/tipografi | 9.4 | Sözlük düzeldi; boşluk/ritim ufak eksik |
| Görsel pipeline | 9.0 | Aday/native/crop/audit gerçek; gate entegrasyonu eksik |
| MCP/CLI yüzeyi | 9.3 | 41 tool, schema 17; kararlar tek çağrıda |
| Gate truth/QA | 8.2 | P1 figure-page gürültüsü; `passed:true` yanıltıcı |
| Dokümantasyon/reproducibility | 9.2 | ADR, plan, kalibrasyon, acceptance güçlü; build/tag kaydı sonradan düzeltildi |
| **Genel** | **9.1** | v3’e göre yeni yetenek güçlü; P1 gate truth puanı aşağı çekiyor |

---

## 5. Öncelikli sonraki adımlar

1. **P1:** `visual_gate` sayfalarını `visual_candidates` manifestinden türet;
   PD0/PD2/PD3 için `required` modda kabul testi ekle.
2. **P2:** s. 2/10/17/22 için editöryel doluluk veya bilinçli intentional-page.
3. **P2:** pilot/roadmap metnini okuyucu yüzeyinden çıkar.
4. **P2:** s. 19 tablo devamında grup başlığını tekrarla.
5. **P2:** kalibrasyon v2: PD0/PD1 dış örneklem, precision/recall, hata matrisi.
6. **P3:** `visual_brief` opsiyonel `top_candidates`; `assets check` source digest.
7. **P3:** en az bir shipped guide’da gerçek native/crop örneği.

---

## 6. Kanıt komutları

```bash
make acceptance
make visuals-v5
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

pdfinfo output/PD0-PD1_Calisma_Ozeti.pdf
pdftotext -layout output/PD0-PD1_Calisma_Ozeti.pdf -
folisift qa --guide pd0-pd1
folisift qa --guide pd2
folisift qa --guide pd3

folisift visual candidates --source sources/verimadenciliği.pdf --pages 14,45,71 --min-score 0
folisift visual candidates --source sources/PD2.pdf --pages 5,29,53 --min-score 0
folisift visual candidates --source sources/PD3.pdf --pages 76 --min-score 0
```

**Sonuç:** PDF yayınlanabilir kalitede; görsel pipeline kullanılabilir ve
kalibrasyonu v1 olarak tamamlandı. 9.1’in altına düşüren tek ciddi kalem
`qa.visual` gate truth P1’i; onu kapattıktan sonra 9.4–9.5 bandı gerçekçi.
