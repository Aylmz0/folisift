# Folisift 0.22.0 — Dördüncü Üretim Geri Bildirimi (v4)

**Tarih:** 5 Kasım 2026
**Sürüm:** Folisift **0.22.0** · build **230** · MCP schema **17** · **41 tool**
**Önceki rapor:** `FOLISIFT-GERI-BILDIRIM-v3.md` (0.20.0 / 0.22.0 geçişi)
**Kapsam:** 0.22.0 ile üç PDF’in (26/10/19) tam üretim geçişi; yeni glossary/figure/D2/visual akışı; kalan bug’lar ve 10/10 yol haritası.

---

## 0. Kısa hüküm

**0.22.0, 0.20.0’ın üzerine görsel üretim ve yapısal tutarlılık ekledi.** Bu turda üç PDF de yeni sisteme taşındı: `kind="glossary"` tabloları, `::: figure` gömüleri, `visual_decision` ile eşleşen görsel denetim kayıtları, D2 diyagramları ve temiz audit.

**Puan: 9.5 / 10** (0.20.0: 9.2; 0.22.0 ilk tur: 9.4)

10/10’a kalan tek gerçek bug **`coverage_manifest generated` (17/138)**; yanında `validate` çıktı bütçesi, `hint`’in birleşik başlık körlüğü, `excluded` statüsü ve iki MCP path hatası var. Bunlar kapanınca 10/10.

---

## 1. Bu turda üretilen PDF’ler

| PDF | Sayfa | Coverage | Audit (strict) | Görsel | Kart |
|---|---:|---|---|---|---:|
| `output/is-hukuku.pdf` | 26 | **141/141** | **pass**, 0 issue | 3 D2 spec / 6 asset, 18 visual entry, missing 0 | 9 |
| `output/is-hukuku-irac.pdf` | 10 | **24/24** | **pass**, 0 issue | — | 5 |
| `output/dijital-donusum-temelleri.pdf` | 19 | **41/41** | **pass**, 0 issue | 6 D2 + **2 gömülü kaynak figürü**, 79 entry, missing 0, unresolved 0 | 8 |

### Dijital PDF’te düzeltilen bozulmalar

1. **Sözlük taşması (bozuk sürümde s.17–18)** — `::: columns` + 4 sütunlu tablolar content box’ı taşıyordu. Çözüm: tek `::: table kind="glossary"` + `widths="18,32,18,32"`.
2. **Çift B3 bölüm başlığı** — DPP bölümünün önünde yanlış konumlanmış ikinci `chapter` bloğu kaldırıldı.
3. **s.2 boş / chapter-opener split** — `::: front` bloğu normal `## Bu kitapçık hakkında` girişine çevrildi; künye tablosu aynı akışa alındı.
4. **Sayfa: 19 → 18 (sözlük sıkıştırma) → 19 (hızlı tekrar)** — hedef 18–20; audit temiz.
5. **F bölümünde eksik karşılaştırma matrisi** eklendi (üç kaynak × konu).

### Yeni sistem öğeleri (üretimde doğrulandı)

- **`::: table kind="glossary"`**: 4 sütun (Terim/Tanım ×2) + **tam genişlik, renkli grup satırı** (C.1/C.2/C.3). Ana rehberde 66 terim, dijitalde 30 terim bu yapıya taşındı.
- **`::: figure src=... width="NN%"`**: `crop_source` ile üretilen iki kaynak figürü gömüldü (dijital s.6 bileşenler, s.7 kâr formülü); caption + alt + kaynak atfı zorunlu.
- **`visual_decision` + `embed_crop`**: gömülen iki sayfa `describe` yerine `embed_crop` olarak işaretlendi; `asset`, `crop`, `alt`, `caption`, `hash` alanları dolduruldu → `embedded_crops: 2`, `unresolved: 0`.
- **D2**: ana rehberde 3 spec (fesih karar ağacı, yapı, sosyal güvenlik kolları), dijitalde 6 spec; `diagram status` tümü `ok`.
- **Audit**: üç PDF’te `overflow`, `content_box`, `chapter_split`, `underfilled`, `empty` = 0.

---

## 2. 0.22.0 test matrisi (özet)

| # | Test | Sonuç |
|---|---|---|
| T01–T03 | Üç PDF build + audit strict | **PASS** (26/10/19, 0 issue) |
| T04 | Ana rehber coverage | **PASS** 141/141 (curated combined) |
| T05 | IRAC coverage | **PASS** 24/24 (curated review → adopt) |
| T06 | Dijital coverage | **PASS** 41/41 |
| T07 | `coverage_manifest generated` validate | **FAIL** — 17/138, anahtarlarda gövde metni, 4.4.4/10.2 yok |
| T08 | `coverage_manifest validate` bütçe | **FAIL** — `fields`/`max_tokens` `per_source.sections[]`’a uygulanmıyor |
| T09 | `hint` merged heading | **FAIL** — `2.1–2.2` altında `2.2` “likely-missing” |
| T10 | `source_brief` heading count | **PASS** 141 (4.4.4/10.2 dahil) |
| T11 | `section_brief max_tokens=50` | **PASS** `estimated_tokens=50` |
| T12 | `heading_brief max_tokens=20` | **PASS** `estimated_tokens=20`, boş değil |
| T13 | `chapter_brief heading=4` | **PASS** doğru başlık + body + alt bölüm özetleri |
| T14 | `source_topics semantic=true` | **PASS** `model2vec` |
| T15 | `claims detail=summary` | **PASS** `returned/omitted/total/next` |
| T16 | `graph query max_tokens=800` | **PASS** bbox’sız, budgeted |
| T17 | `verify_citations` | **PASS** `confidence`, `match_terms`, `reason` |
| T18 | `style estimate accessible` | **PASS** `over_budget=true` |
| T19 | `style restyle` content_hash | **PASS** before == after (`content_unchanged: true`) |
| T20 | `table_extract` figür sayfası | **PASS** `reason`, `table_likely:false` |
| T21 | `visual_candidates` / `visual_decision` | **PASS** 18 + 79 entry; `decorative` → unresolved (aşağıda) |
| T22 | `crop_source` / `render_pages` / `visual_extract` | **PASS** (dosya, hash, DPI, kalite) |
| T23 | `analyze_source` | **PASS** inventory JSON/MD + headings.review |
| T24 | `prepare_guide` | **PASS** source/domain/learner profili, visual rules, links planı |
| T25 | `graph explain` / `path` | **PASS** (EXTRACTED/INFERRED, 5-hop path) |
| T26 | `links explore` | **PASS** 186 link, potion-multilingual |
| T27 | `read_section` | **FAIL** — `root/<slug>` ve `blocks/sources/*.json` path hatası |
| T28 | `diagram list` / `validate` (MCP) | **FAIL** — `I/O error ... /is-hukuku`, “diagram needs guide”, “invalid slug” |
| T29 | `guide_plan source_pages` | **FAIL** — tüm bölümlerde `source_pages: []` |
| T30 | `source_insights ocr:true` | **FAIL** — “tesseract feature is disabled; no --ocr-server-url” |
| T31 | `qa_guide` gate-fail → draft | **PASS** output PDF korunuyor, draft yazılıyor |
| T32 | Glossary/figure yeni sözdizimi | **PASS** (üretimde görsel doğrulandı) |
| T33 | `cards include_definitions` dijital | **PASS** 8 kart (6 recall + 2 definitions) |

**Dağılım: 26 PASS / 7 FAIL.** FAIL’lerin 6’sı tool bug’ı, 1’i ortam (OCR sunucusu yok).

---

## 3. 0.22.0’da iyi çalışanlar (kanıtlı)

1. **Brief bütçeleri gerçekten uygulanıyor.** `section_brief(50)` → 50 token; `heading_brief(20)` → 20 token; `chapter_brief` artık body + child özetleri + `next_offset` döndürüyor.
2. **`source_brief` 141 heading** — 4.4.4 ve 10.2 artık var (0.20.0’da 139’du).
3. **`style restyle` içerik hash’ini koruyor** — before == after; yalnızca görünüm değişiyor.
4. **`visual_candidates` + `visual_decision` + `crop_source` + `visual_extract`** üretimde çalışıyor; provenance (hash, DPI, kalite) geliyor.
5. **`prepare_guide`** tek çağrıda source/domain/learner profili, visual kuralları, link planı ve prompt fragment’ı veriyor.
6. **`graph explain/path`** ve **`links explore`** deterministik ve budgeted.
7. **`qa_guide`** gate-fail durumunda output’u ezmiyor; `_draft.pdf` yazıyor.
8. **Glossary tablosu** (`kind="glossary"`) renkli grup satırlarıyla doğru render ediliyor.
9. **Figure gömme** (`::: figure ... width="NN%"`) caption/alt/kaynak ile çalışıyor.

---

## 4. Kalan bug’lar ve kabul testleri

### 4.1 `coverage_manifest generated` (kritik)

`validate generated` → `canonical_total: 138, claimed: 17, missing: 121`.
Anahtarlar gövde metnini içeriyor; `4.4.4` ve `10.2` üretilmiyor.

**Kabul:** generated anahtarları body’siz; canonical 4.4.4/10.2 dahil; `validate generated` → 138/138 (veya 141 + excluded).

### 4.2 `coverage_manifest validate` çıktı bütçesi

`detail:"summary"`, `fields`, `max_tokens` verilse de `per_source[].sections[]` tam dökülüyor; `review_diff` bile `fields` altında tam geliyor.

**Kabul:** `max_tokens` ve `fields` nested dizilere uygulanmalı; `omitted/total` sarması geri gelmeli.

### 4.3 `hint` birleşik başlık körlüğü

`2.1–2.2 Üretimde dijital dönüşüm...` altında `2.2` satırı `score:0.333 → likely-missing` görünüyor; `covers` yorumları ve en-dash range parse edilmiyor.

**Kabul:** `2.1–2.2` gibi birleşik başlıklar her iki satırı da `covered` saymalı; `covers` string’leri hint’e girdi olmalı.

### 4.4 `status:"excluded"` yok

Bilinçli kapsam dışı satırlar manifest’ten **silinerek** eleniyor; audit izi kalmıyor. Dijitalde 21 satır bu şekilde çıkarıldı.

**Kabul:** `status:"excluded"` + `reason`; validator `excluded` sayısını raporlasın; `missing`’e eklemesin.

### 4.5 `read_section` path hatası

`guide` slug’ı `root/<slug>` olarak çözülüyor; blocks dosyası `data/blocks/sources/<source>.pdf.json` aranıyor. Doğrusu `guides/<slug>/data/blocks/<source>.json`.

**Kabul:** `read_section` `read_blocks` ile aynı çözümleyiciyi kullansın.

### 4.6 `diagram list` / `validate` (MCP) path hatası

`guide:"is-hukuku"` → `I/O error at .../is-hukuku`; `guide:"guides/is-hukuku"` → `invalid slug`; `validate` bazen “diagram needs guide”. CLI `diagram build` çalışıyor.

**Kabul:** MCP `diagram` `status` ile aynı slug çözümlemesini kullansın; `validate`/`list`/`render_plan` tutarlı olsun.

### 4.7 `guide_plan source_pages` boş

24/14/28 bölüm için `source_pages: []`. Rehber bölümleri kaynak sayfalarına bağlanmıyor.

**Kabul:** `covers` yorumlarından veya heading eşlemesinden `source_pages` doldurulsun.

### 4.8 OCR pratikte kapalı

`source_insights ocr:true` → “OCR enabled but no --ocr-server-url provided and tesseract feature is disabled”. Tesseract 5.5.3 kurulu ama binary’de feature kapalı.

**Kabul:** yerel tesseract yolu veya `--ocr-server-url` dokümante edilsin; en azından `doctor` “OCR unavailable” desin.

### 4.9 Küçük notlar

- `visual_decision` `decorative` kararı “unresolved” sayılıyor; kapak/ön söz için `describe` kullanmak gerekiyor.
- `visual_decision embed_crop` için `crop` alanı zorunlu; eksikse unresolved.
- `cards`: dijital guide’a 6 `::: recall` + 6 `::: answer` eklendi → 8 kart (6 recall + 2 definitions).
- `qa_guide` `visual_gate` “warn” döndürüyor; `missing:0` olsa bile “pass” olmuyor. Anlamı dokümante edilmeli.
- `fields` parametresi bazı tool’larda nested alanları kırpmıyor (validate, hint).
- `source_insights` image_id döndürmüyor; `visual_extract` için `read_blocks` figure_id kullanmak gerekiyor.

---

## 5. 10/10 yol haritası

### P0 — 0.22.x hotfix
1. `coverage_manifest generated` body’siz anahtar + canonical 4.4.4/10.2.
2. `coverage_manifest validate` `max_tokens`/`fields` nested kesme.
3. `hint` merged-heading + `covers` desteği.
4. `read_section` ve `diagram list/validate` path çözümlemesi.
5. `guide_plan source_pages` doldurma.

### P1 — 0.23
6. `status:"excluded"` + validator sayımı.
7. OCR yerel tesseract veya server dokümantasyonu; `doctor` kontrolü.
8. `visual_gate` “warn/pass” anlamı ve `decorative` davranışı.
9. `cards` için `recall` blokları silinirse `warning` üret.
10. `source_insights` image_id listesi (visual_extract entegrasyonu).

### P2 — 1.0
11. D2 `validate`/`list`/`render_plan` tam MCP paritesi; `decision` kind.
12. `graph path/explain` bounded output dokümantasyonu.
13. Determinism manifest (version/hash/seed).
14. Docs cookbook: glossary/figure sözdizimi, review→adopt→qa akışı, draft temizleme.

---

## 6. Puan kartı

| Alan | 0.20.0 | 0.22.0 (v4) |
|---|---:|---:|
| Build / PDF | 9.5 | 9.5 |
| Audit / layout | 9.0 | **9.5** (üç PDF 0 issue) |
| QA tek çağrı | 9.8 | 9.8 |
| Coverage | 8.5 | **9.0** (141/24/41 pass; generated kırık) |
| Source/context economics | 8.5 | **9.5** (brief bütçeleri, chapter body, 141 heading) |
| D2 | 8.0 | **8.5** (status/render_plan ok; list/validate MCP kırık) |
| Style | 9.0 | **9.2** (restyle hash korunuyor) |
| API tutarlılığı | 9.0 | **9.0** (path bug’ları, validate/hint fields) |
| Determinism | 9.0 | 9.0 |
| Dokümantasyon | 8.0 | **8.5** (glossary/figure yeni; akış hâlâ yazılı değil) |

**Ağırlıklı ≈ 9.5 / 10**

---

## 7. Sonuç

0.22.0 ile üretim hattı **görsel olarak tamamlandı**: glossary tabloları, kaynak figür gömüleri, D2 diyagramları ve visual-audit kayıtları aynı sistemde çalışıyor. Üç PDF de audit-clean, coverage pass ve tek `qa_guide` çağrısıyla doğrulanıyor.

10/10 için kalan iş **tek bir kritik bug** (`generated` manifest) + **yedi orta bug** (validate bütçesi, hint, `read_section` path, `diagram` path, excluded, OCR, guide_plan). Bunlar kapanınca “küratörlü geçen” değil, **tool’un kendi ürettiği** 141/141’e ulaşılır.
