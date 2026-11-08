# Folisift 0.23.2 — Beşinci Üretim Geri Bildirimi (v5)

**Tarih:** 7 Kasım 2026
**Sürüm:** Folisift **0.23.2** · build **248** · MCP schema **19** · **41 tool**
**Önceki rapor:** `FOLISIFT-GERI-BILDIRIM-v4.md` (0.22.0 / build 230)
**Kapsam:** Üç PDF’in (26/10/19) final üretimi, 41 tool’un tam testi, greenfield `prepare_guide`/`scaffold_guide` probe’u, kabul testleri ve kalan bug’lar.

---

## 0. Kısa hüküm

**0.23.2, v4’teki P0 listesinin tamamına yakınını kapattı.** `coverage_manifest generated` artık ana rehberde **139/139**; `4.4.4` ve `10.2` otomatik manifestte var. `fields`/`max_tokens` nested dizilere uygulanıyor; `hint` birleşik başlıkları sayıyor; `read_section` normal başlıkları çözüyor; `diagram list/validate` MCP’de çalışıyor; `guide_plan` kaynak sayfalarını dolduruyor; `excluded` ve `decorative` doğru davranıyor; OCR hatası doctor ile tutarlı.

**Puan: 9.2 / 10** (v4: 9.5 — ancak v4 33 testlik daha dar matrise dayanıyordu; v5 41 tool + greenfield matrisi uyguluyor).

**Dağılım: 30 PASS / 9 PARTIAL / 2 FAIL.**
Üretim hattı alt puanı **9.7**; sıfırdan rehber üretme (greenfield) alt puanı **6.5**. Genel puan bu iki yolun ağırlıklı ortalamasıdır.

10/10 için kalan gerçek blocker’lar:
1. `prepare_guide` / `scaffold_guide` tek bölüm üretiyor ve `::: front` strict audit’te chapter-opener split veriyor.
2. `coverage_manifest build` dijitalde TOC/gövde metnini canonical satır sayıyor; `review_diff` anahtar uzayı uyuşmuyor; `review_eligible:false` iken review dosyası yazmıyor.
3. `read_source` küçük `max_tokens` ile sayfa metnini tamamen düşürüyor (`returned:0`).

---

## 1. Bu turda üretilen PDF’ler

| PDF | Sayfa | Coverage | Audit (strict) | Görsel | Kart | SHA-256 (ilk 16) |
|---|---:|---|---|---|---:|---|
| `output/is-hukuku.pdf` | **26** | **141/141** | **pass**, 0 issue | 3 D2 spec / 6 asset, 18 visual entry, missing 0 | **9** | `c96e86f8e32e4ba5` |
| `output/is-hukuku-irac.pdf` | **10** | **24/24** | **pass**, 0 issue | — | **5** | `9a1027db3c13e8ed` |
| `output/dijital-donusum-temelleri.pdf` | **19** | **41/41** | **pass**, 0 issue | 6 D2 + **2 gömülü kaynak figürü**, 79 entry, missing 0, unresolved 0 | **8** | `52648082885fc7c7` |

### 1.1 Determinizm ve bütünlük

| Guide | İçerik hash’i (build = qa) | PDF boyutu | Bookmark | Etiketli |
|---|---|---|---:|---:|---|
| `is-hukuku` | `fcf7244ae28a5dcf8924d651713af6be9d83a6103e43d6425822084921b1e3da` | 513.122 B | 44 | evet |
| `is-hukuku-irac` | `43609446bee7dd499261a0dac6b8c2d0ddc03b69c48840168c43594bdc49d0fc` | 196.664 B | 56 | evet |
| `dijital-donusum-temelleri` | `a08848781c3bbeba3d2e6f0181aefb034e6fcc58707571428d2ac5f5def366b1` | 883.648 B | 32 | evet |

Üç PDF de A4, PDF 1.7, tagged, yerel-only. `qa_guide` içindeki `build.content_hash` ile `build_guide` hash’i aynı; ikinci build’de cache hit ve aynı sayfa sayısı alındı. Ağ erişimi yok; OCR off; reranker off.

### 1.2 Kart sayıları

| Guide | `qa_guide` | `cards include_definitions:true` | Varsayılan `cards` |
|---|---:|---:|---:|
| Ana rehber | 9 (8 recall + 1 definition) | 9 | 8 |
| IRAC | 5 | 5 | 5 |
| Dijital | 8 (6 recall + 2 definitions) | 8 | 6 |

`cards` varsayılanının definition’ları dışlaması ile `qa_guide`’ın dahil etmesi arasındaki fark P3 notu olarak aşağıda.

---

## 2. 0.23.2’de kapanan v4 maddeleri

| v4 maddesi | 0.23.2 kanıtı | Durum |
|---|---|---|
| `coverage_manifest generated` body’siz anahtar + `4.4.4`/`10.2` | Ana rehber `validate generated` → **139/139**, `origin_counts: {heading:135, text:4}`, `missing: []` | **KAPANDI** |
| `fields`/`max_tokens` nested kesme | `per_source[].sections[]` artık `{returned:2, omitted:3, total:5}` sarmasıyla geliyor; `review_diff` de budgeted | **KAPANDI** |
| `hint` birleşik başlık körlüğü | Dijital `hint` → `covered:27`, **`merged_heading_hits:25`**, `likely_missing:0` | **KAPANDI** |
| `read_section` path/normal başlık | `read_section(heading:"4.4")` → 90 char, `page_start/page_end:12`; path artık `guides/<slug>/data/blocks` | **KAPANDI** (inline başlıklar hariç) |
| `diagram list/validate` MCP path | Ana rehber `list` → 3/3 `ok`; dijital `validate` → 6/6 `ok`, `errors: []` | **KAPANDI** |
| `guide_plan source_pages` boş | Ana rehber 24 bölüm; **17 kaynak-backed**, **7 guide-only** `source_pages_unknown:true` (A.1/A.2, C.4, D–G) | **KAPANDI** |
| `status:"excluded"` yok | Test manifesti: `canonical_total:41, claimed:40, excluded:1, missing:0, passed:true` | **KAPANDI** |
| `decorative` unresolved sayılıyor | Test kararı sonrası `visual.status:"passed"`, `decorative:1`, `unresolved:[]` | **KAPANDI** |
| OCR belirsizliği | `source_insights ocr:true` → `OCR_UNAVAILABLE: build with --features ocr-tesseract or pass --ocr-server-url`; doctor aynı dili kullanıyor | **KAPANDI** (özellik yok, mesaj net) |
| `source_brief` 141 heading | `heading_count:141`, `inventory_heading_count:141`, `manifest_heading_count:141`, `review_added:1`, `ocr.needed:false` | **KAPANDI** |

---

## 3. 0.23.2 test matrisi — 41 tool

| # | Tool | Sonuç | Kanıt |
|---:|---|---|---|
| 1 | `workspace_info` | **PASS** | 0.23.2 / build 248 / schema 19 / 41 tool / 3 guide / 5 source / PDFium chromium/8058 |
| 2 | `doctor` | **PASS** | pdfium ok; 3 guide, 0 invalid; 3 style compatible; OCR off bilgisi açık |
| 3 | `list_guides` | **PASS** | 3 guide valid; section 24/14/25; source 1/1/3 |
| 4 | `guide_status` | **PARTIAL** | Ana rehberde `empty_sections:["3 kaynaklar","3"]`; coverage gate `empty_sections:[]` diyor. `inventory` yolu `guides/is-hukuku/data/inventory.json` yok. |
| 5 | `prepare_guide` | **FAIL** | Probe `probe-0232`: 83 kaynak heading’ine rağmen **1 chapter** (`Ders_Notu_1_VİZE.pdf`), `outline.items:[]`, strict audit `chapter opener split [2,3]`. |
| 6 | `scaffold_guide` | **FAIL** | Probe `probe-scaffold` aynı `::: front` split’ini üretiyor. `style:"clean"+domain:"law"` ve varsayılan `workbook`+`structure:"irac"` reddediliyor. |
| 7 | `source_brief` | **PASS** | Ana rehber 141 heading; `manifest_source:"combined"`; `ocr.needed:false`; `low_text_pages:[]` |
| 8 | `read_source` | **PARTIAL** | `max_tokens:200`, pages 1–2 → `returned_pages:2` ama `pages:[{omitted:2, returned:0}]`; hiç metin yok. `max_tokens:1000` ile sayfa 1 tam geliyor. |
| 9 | `read_section` | **PARTIAL** | `4.4` → 90 char, s.12 doğru. `4.4.4` → `section "4.4.4" not found`. |
| 10 | `read_blocks` | **PASS** | Dijital s.4 → figure `p4_1`, heading, paragraph blokları; `view.page_count:31` |
| 11 | `search_source` | **PASS** | `sources/is-hukuku.pdf`, “kıdem tazminatı” → 2/28 sayfa; s.12/s.14 snippet + hint |
| 12 | `citation_snippet` | **PASS** | Ana rehber s.66, “kıdem tazminatı” → exact cümle |
| 13 | `section_brief` | **PARTIAL** | Normal `4.4` çalışıyor; `4.4.4` “section not found”. |
| 14 | `heading_brief` | **PARTIAL** | Normal başlık çalışıyor; inline `4.4.4` “section not found”. |
| 15 | `chapter_brief` | **PARTIAL** | Normal `12` çalışıyor; `4.4.4` “section not found”. |
| 16 | `source_topics` | **PASS** | Lexical `heading-terms` 5 topic; semantic `model2vec` 5 topic; `label_source:"heading"` |
| 17 | `source_insights` | **PASS** | Non-OCR insights çalışıyor; `ocr:true` → net `OCR_UNAVAILABLE`; doctor ile tutarlı |
| 18 | `analyze_source` | **PASS** | Workspace içi out → 117 sayfa / 36.439 kelime / `headings.review.md`; `/var/tmp` → net “path escapes the workspace” |
| 19 | `table_extract` | **PASS** | Ana s.9/s.13 ve dijital s.4/s.8; `table_likely` + `reason` dönüyor |
| 20 | `verify_citations` | **PARTIAL** | `checked:5` ama `missing:75`; `total:189`, `referenced:114`, `unreferenced:75`. “missing” checked içinde mi, tüm guide’da mı belirsiz. |
| 21 | `visual_brief` | **PASS** | Dijital s.4 → figure/heading/paragraph özeti, caption adayı |
| 22 | `visual_evidence` | **PASS** | Dijital s.4 → layout-block tabanlı kanıt; `basis:"layout-blocks"` |
| 23 | `visual_candidates` | **PASS** | Dijital s.4/s.13 → bbox, `confidence`, `content_class`, `caption_seed`, `recommended_width_percent` |
| 24 | `visual_decision` | **PASS** | `embed_crop` (crop+bbox) ve `decorative` kabul; audit JSON + hash + markdown üretiyor. Argüman doğrulaması iteratif (P2). |
| 25 | `visual_extract` | **PARTIAL** | `out:"...probe-p4_1.png"` → `format:"jpg"`, dosya gerçekte **JPEG** (330×93). Uzantı ile format uyuşmuyor. |
| 26 | `render_pages` | **PASS** | s.4 @100dpi → 1000×750 PNG; crop ve contact-sheet (1224×924) çalışıyor |
| 27 | `visual_assets` | **PASS** | Dijital `checked:2, missing:0, mismatched:0, ok:true`; ana rehber 0 asset, ok |
| 28 | `crop_source` | **PASS** | `bilesenler.png` 1920×1500, `kar-formulu.png` 1800×1230; PNG, kalite raporu tam |
| 29 | `coverage_manifest` | **PARTIAL** | `generated` ana rehber 139/139; dijital `build` 93 satır (51 heading + 28 toc + 14 text), `validate` 55/93; `review_diff` 92 vs 40; `adopt:true` → `adopted:false`, review dosyası yok. |
| 30 | `check_coverage` | **PASS** | Ana 141/141, IRAC 24/24, dijital 41/41; `missing:0`, `empty_sections:[]` |
| 31 | `build_guide` | **PASS** | 26/10/19 sayfa; üçü de `passed:true`, `warnings:0` |
| 32 | `audit_guide` | **PASS** | Üç PDF strict: `issues:[]`, `empty_pages:[]`, `overflow_pages:[]`, `chapter_split_pages:[]`, `underfilled_pages:[]` |
| 33 | `qa_guide` | **PASS** | Üçü `passed:true`; kart 9/5/8; dijital `visual.status:"passed"`, `missing:0`, `unresolved:0` |
| 34 | `cost_report` | **PASS** | Ana 16.864 guide / 59.535 source token; dijital 10.878 / 86.351; IRAC 5.452 / 59.535 |
| 35 | `cards` | **PASS** | `include_definitions:true` → 9/5/8; CSV’ler yazıldı; varsayılan 8/5/6 |
| 36 | `graph` | **PASS** | Ana rehber `status` → 2.090 node / 3.882 edge; `document-graph.json` yazıldı |
| 37 | `links` | **PASS** | Ana rehber 186 link; `model_verified:true`; `document-links.json` |
| 38 | `style` | **PASS** | 5 style listelendi; reader/accessible/brief tanımları domain/structure matrisiyle geliyor |
| 39 | `diagram` | **PASS** | Ana `list` 3/3 ok; dijital `validate` 6/6 ok; `errors:[]` |
| 40 | `guide_plan` | **PASS** | 24 bölüm; 17 kaynak-backed; 7 `source_pages_unknown:true`; `target_pages:"24-26"` |
| 41 | `claims` | **PASS** | 189 claim; 114 referenced; 75 unreferenced; `data/claims.md` yazıldı; `noise_filtered:61` |

**Dağılım: 30 PASS / 9 PARTIAL / 2 FAIL.** PARTIAL’lerin 4’ü aynı kök nedene (inline başlık çözümlemesi) bağlı; 2’si coverage/visual sözleşmesi; 1’i `read_source` bütçesi; 1’i `guide_status`; 1’i `verify_citations` anlamı.

---

## 4. Kalan bug’lar ve kabul testleri

### 4.1 P0 — `prepare_guide` / `scaffold_guide` greenfield’de tek bölüm üretiyor

**Kanıt:** `prepare_guide(slug:"probe-0232", sources:["sources/Ders_Notu_1_VİZE.pdf"], structure:"a-g")`:
- `layout[0].headings:83`, `pages:31`, `blocks:272`
- `outline.items:[]`; guide.md’de tek `::: chapter title="Ders_Notu_1_VİZE.pdf"`
- `build_guide` 4 sayfa; `audit_guide strict` → **“possible chapter opener split across pages: [2, 3]”**
- `source_profile.units.sections:0`

`scaffold_guide` aynı `::: front` bloğunu ve tek chapter’ı üretiyor. Ayrıca:
- `style:"clean", domain:"law"` → `style "clean": domain "law" is not supported`
- varsayılan `style:"workbook", structure:"irac"` → `style "workbook": structure "irac" is not supported`

**Beklenen:** Kaynak heading’lerinden çok bölümlü outline; `::: front` yerine strict audit’ten geçen normal giriş; desteklenen style/domain/structure kombinasyonlarının tool açıklamasında veya `doctor`’da listelenmesi.

**Kabul:** Yeni probe 8+ bölüm, strict audit 0 issue, `outline.items` dolu; `workbook+irac` veya `reader+irac` çalışıyor.

### 4.2 P0 — `coverage_manifest build` dijitalde TOC/gövde metnini canonical sayıyor

**Kanıt:** Dijital `build` → 93 satır:
- `origin_counts: {heading:51, toc:28, text:14}`
- `validate generated` → `claimed:55`, `missing:38`
- `review_diff` → `only_in_generated:92`, `only_in_review:40`, `review_eligible:false`
- `build adopt:true` → `adopted:false`; `coverage-manifest.review.json` yazılmıyor; ardından `review` → **“no coverage manifest to review; run coverage_manifest build”**

**Beklenen:** `toc` ve gövde `text` satırları canonical’a girmemeli; generated/review anahtarları aynı normalize uzayda olmalı; `review_eligible:false` iken ya review dosyası yazılmalı ya da “neden review edilemez” aksiyon mesajı verilmeli.

**Kabul:** Dijital `build` ≈ 41 canonical; `review_diff` yalnızca gerçek farkları gösterir; `adopt` review sonrası `coverage-manifest.json` üretir; ikinci `build` idempotent.

### 4.3 P1 — `read_source` küçük bütçede sayfayı tamamen düşürüyor

**Kanıt:** `read_source(source:"sources/is-hukuku.pdf", pages:"1-2", max_tokens:200)` →
`returned_pages:2`, `pages:[{omitted:2, returned:0, total:2}]`, `chars:4429`, `truncated:true`, metin yok.

**Beklenen:** En az ilk sayfanın metnini döndürmeli veya “bütçe çok küçük; en az N token ver” hatası vermeli. Sessiz boş dönüş kabul edilemez.

**Kabul:** `max_tokens:200` ile en az bir sayfa metni veya açık hata; `next_offset` ile devam.

### 4.4 P1 — Inline/text-origin başlıklar brief tool’larında çözülmüyor

**Kanıt:** `4.4.4` ve `10.2` `source_brief`/generated manifestte var; `read_section`, `section_brief`, `heading_brief`, `chapter_brief` → **“section not found”**. Normal `4.4` ve `12` çalışıyor.

**Beklenen:** `source_brief` ile aynı heading çözümleyicisi; numara, tam başlık ve normalize başlık eşleşmesi.

**Kabul:** `4.4.4` ve `10.2` dört tool’da da doğru bölümü döndürür.

### 4.5 P1 — `guide_status` yanlış `empty_sections` ve olmayan inventory yolu

**Kanıt:** Ana rehber `empty_sections:["3 kaynaklar","3"]`; aynı guide’ın `check_coverage` çıktısı `empty_sections:[]`. `inventory:".../data/inventory.json"` dosyası yok.

**Beklenen:** Status, coverage gate ile aynı kaynağı kullanmalı; inventory opsiyonel ise `null` veya “not generated” dönmeli.

**Kabul:** `guide_status.empty_sections == check_coverage.empty_sections`; inventory yolu ya var ya açıkça opsiyonel.

### 4.6 P1 — `visual_extract` JPEG byte’larını `.png` yoluna yazıyor

**Kanıt:** `out:"data/crops/probe-p4_1.png"` → `format:"jpg"`, `file` çıktısı **JPEG image data**, 330×93. `asset` yolu `.png`.

**Beklenen:** Uzantıya göre encode et (PNG) veya gerçek uzantıyı `.jpg` olarak döndür/raporla.

**Kabul:** `file` çıktısı ile `format` ve uzantı aynı; `visual_assets check` hash’i doğrular.

### 4.7 P1 — `verify_citations` “missing” anlamı belirsiz

**Kanıt:** `checked:5`, `missing:75`, `total:189`, `referenced:114`, `unreferenced:75`. `missing`, checked 5’in içindeki başarısız mı yoksa tüm guide’daki atıfsız claim mi belli değil.

**Beklenen:** `missing_citations_checked`, `unreferenced_total`, `checked` ayrı alanlar; veya dokümanda net tanım.

**Kabul:** Alan adları tek anlamlı; `missing <= checked` veya adı `unreferenced_total`.

### 4.8 P2 — `visual_gate:"warn"` ile `visual.status:"passed"` çelişkisi

**Kanıt:** Dijital `visual.status:"passed"`, `missing:[]`, `unresolved:[]`, `warnings:[]`; buna rağmen `visual_gate:"warn"`.

**Beklenen:** `passed` → `visual_gate:"pass"` veya `warn`’ın “advisory, engel değil” olduğu dokümante edilmeli.

### 4.9 P2 — Path/ID sözleşmeleri tool’dan tool’a değişiyor

- `read_source` / `search_source`: `sources/<file>.pdf`
- `read_blocks`: çıplak source id (`Ders_Notu_1_VİZE`)
- `visual_extract`: `image_id` (`p4_1`), `source` path
- `crop_source`: `source` path + normalize crop
- `analyze_source out`: workspace dışına yazamaz (“path escapes the workspace”)

Her biri kendi içinde savunulabilir; ama aynı oturumda sürekli format değiştirmek hata üretiyor. Tek normalize edici veya her tool’un açıklamasında tek satır örnek gerekli.

### 4.10 P2 — `folisift://authoring-spec` ve `folisift://design-system` kırık

**Kanıt:** MCP resource okuma → `I/O error at /home/yilmaz/Documents/DersNot/docs/authoring-spec.md` ve `docs/design-system.md`; dosyalar kurulu değil.

**Beklenen:** Resource’lar paketle gelmeli veya resource listesinden kaldırılmalı.

### 4.11 P2 — `visual_decision embed_crop` argüman doğrulaması iteratif

**Kanıt:** İlk çağrı `crop` ile → “requires bbox or candidate_id”; `bbox` ile → “requires --crop when no candidate is supplied”; `bbox+crop` ile çalışıyor.

**Beklenen:** Tek geçişte tüm eksikleri raporla; `crop` tek başına yeterli olsun (zaten crop normalize edilebiliyor).

### 4.12 P3 — `cards` ve `qa_guide` definition varsayılanı farklı

`qa_guide` 9/5/8 sayıyor; `cards` varsayılanı 8/5/6. `include_definitions:true` ile hizalanıyor. Varsayılanlar aynı olmalı veya `cards` çıktısı “definitions excluded” uyarısı vermeli.

### 4.13 P3 — Scaffold style/domain/structure matrisi eksik

`clean+law` ve `workbook+irac` reddediliyor; hangi kombinasyonların desteklendiği yalnızca hata mesajından öğreniliyor. `doctor` veya `style info` bu matrisi vermeli.

---

## 5. Kabul testleri

| # | Test | Sonuç | Kanıt |
|---:|---|---|---|
| A1 | Üç PDF build + strict audit + QA | **PASS** | 26/10/19; 0 issue; `qa.passed:true` |
| A2 | Coverage (curated combined) | **PASS** | 141/141, 24/24, 41/41 |
| A3 | `coverage_manifest generated` ana rehber | **PASS** | 139/139; `4.4.4`/`10.2` dahil |
| A4 | `excluded` statüsü | **PASS** | 41 canonical / 40 claimed / 1 excluded / 0 missing |
| A5 | Dijital görsel | **PASS** | 2 crop, 6 D2, 79 entry, missing 0, unresolved 0, assets 2/2 |
| A6 | Determinizm | **PASS** | build hash = qa hash; aynı sayfa sayısı; cache hit |
| A7 | Greenfield `prepare_guide`/`scaffold_guide` | **FAIL** | 1 chapter; `outline.items:[]`; strict split [2,3] |
| A8 | OCR teşhisi | **PASS** | `OCR_UNAVAILABLE` net; doctor tutarlı |
| A9 | Test artefaktları temizliği | **PASS** | Yalnızca 3 guide; probe guide/output yok |

**8 PASS / 1 FAIL.**

---

## 6. Yol haritası

### P0 — 0.23.3 hotfix
1. `prepare_guide`/`scaffold_guide`: kaynak heading’lerinden çok bölümlü outline; `::: front` yerine strict-safe giriş.
2. `coverage_manifest build`: `toc`/`text` satırlarını canonical’dan çıkar; `review_diff` anahtar uzayını hizala; `review_eligible:false` için review dosyası veya aksiyon hatası.
3. `read_source`: küçük bütçede ilk sayfayı döndür veya açık hata ver.

### P1 — 0.24
4. Inline/text-origin heading çözümleyicisi: `4.4.4`, `10.2` dört brief tool’unda.
5. `guide_status`: coverage gate ile aynı `empty_sections`; inventory opsiyonel.
6. `visual_extract`: uzantı/format tutarlılığı (PNG veya `.jpg`).
7. `verify_citations`: `missing` alanını tek anlamlı hale getir.
8. `visual_gate` ile `visual.status` eşlemesi.

### P2 — 1.0
9. Path/ID normalizasyonu ve tek satır örnekler.
10. `authoring-spec` / `design-system` resource’larını paketle.
11. `visual_decision` tek geçişli argüman doğrulaması; `crop` tek başına.
12. `cards`/`qa_guide` definition varsayılanı.
13. Scaffold style/domain/structure matrisi + `doctor` kontrolü.

---

## 7. Puan kartı

| Alan | 0.22.0 (v4) | 0.23.2 (v5) |
|---|---:|---:|
| Build / PDF | 9.5 | **9.7** |
| Audit / layout | 9.5 | **9.7** |
| QA tek çağrı | 9.8 | **9.8** |
| Coverage | 9.0 | **9.2** (generated düzeldi; build pollution sürüyor) |
| Source/context economics | 9.5 | **9.4** (inline heading + read_source bütçesi) |
| D2 | 8.5 | **9.5** (list/validate düzeldi) |
| Style | 9.2 | **9.3** |
| API tutarlılığı | 9.0 | **8.0** (greenfield FAIL + path/ID farkları) |
| Determinism | 9.0 | **9.5** |
| Dokümantasyon | 8.5 | **8.0** (resource’lar kırık) |

**Eşit ağırlıklı ≈ 9.2 / 10.**
Alt puanlar: **üretim hattı 9.7**, **greenfield 6.5**.

---

## 8. Sonuç

0.23.2, mevcut rehberleri üretme yolunda **güvenilir**: üç PDF de 26/10/19 sayfa, 141/24/41 coverage, strict audit 0 issue ve tek `qa_guide` çağrısıyla doğrulanıyor. v4’ün en kritik maddesi olan `coverage_manifest generated` artık ana rehberde 139/139; `4.4.4` ve `10.2` otomatik manifestte. `excluded`, `decorative`, `diagram list/validate`, `guide_plan` ve OCR teşhisi de kapanmış durumda.

10/10 için kalan iş **üç P0**: greenfield scaffold’un çok bölümlü üretimi, `coverage_manifest build`’in TOC/gövde kirliliği ve `read_source`’un sessiz boş dönüşü. Bunlar kapanınca Folisift yalnızca “küratörlü rehberi basan” değil, **sıfırdan rehber kuran** bir sistem olur.
