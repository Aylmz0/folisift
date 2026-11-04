# Folisift 0.17.1 — İkinci Üretim Geri Bildirimi ve 10/10 Yol Haritası

**Tarih:** 4 Kasım 2026  
**Sürüm:** Folisift 0.17.1, build 185, MCP schema 13, 35 tool  
**Önceki rapor:** `FOLISIFT-GERI-BILDIRIM.md`  
**Amaç:** 0.17.1’in neyi düzelttiğini, neyin hâlâ kırık olduğunu ve sistemi 10/10’a çıkaracak son adımları tek belgede toplamak.

---

## 0. Kısa hüküm

**0.17.1, 0.17.0’a göre belirgin iyi.** Özellikle `qa_guide`, yeni `audit_guide`, `source_brief`, `source_insights`, `visual_brief`, `citation_snippet`, `table_extract`, `style estimate`, `cards include_definitions`, `stats.tables` ve `graph query` gerçek üretimde işe yarıyor.

Ama sistem **henüz 10/10 değil**. Şu anki dürüst puanım:

**8.0 / 10**

Neden 10 değil:

1. **Coverage manifest hâlâ yanlış.** 141 satır review manifest “claimed: 141” diyor ama tüm alt başlıkları `empty_sections` olarak işaretliyor; build legacy **7/7** coverage ile geçiyor.
2. **`section_brief` / `heading_brief` token bütçesini yok sayıyor.** `max_tokens: 20` ile bile 939 token dönüyor.
3. **`chapter_brief` yanlış heading’e eşleşiyor.** `heading: "4"` → `1.4. Sosyal Güvenlik Hukuku`.
4. **`source_topics semantic:true` hâlâ lexical.** `method: "heading-terms"` dönüyor.
5. **`claims` ve `graph query` `detail` / `max_tokens`’ı yok sayıyor.** Binlerce satır / büyük bbox çıktısı.
6. **IRAC metrikleri tutarsız.** `list_guides.case_count: 0`, `guide_status.section_count: 0`.
7. **`verify_citations` snippet’i kaba.** Sayfa başlığını snippet olarak döndürüp `verified: true` diyebiliyor.
8. **D2 PATH sorunu.** `doctor` başta D2’yi bulamadı; symlink ile düzeldi.
9. **`style estimate` over_budget’i yanlış.** `accessible` 12–14, hedef 10–12, `over_budget:false`.
10. **`visual-audit` “table” taxonomisi belirsiz.** Figür-tabloya çevirme ile native tablo ayrılmıyor.
11. **`cards` sayısı qa’da eksik.** CSV 9, `qa_guide.cards` 8.
12. **IRAC scaffold hâlâ generic.** 10 case yerine 60+ kaynak başlığı üretiyor.

Bu maddeler düzelirse 10/10 gerçekçi.

---

## 0.1 Test ortamı ve yöntem

- Folisift **0.17.1**, build **185**, MCP schema **13**, **35 tool**.
- D2 **v0.9.0**; başta PATH’te yoktu, `~/.local/bin/d2` symlink’i ile düzeldi.
- Model: `potion-multilingual-128M-int8`.
- Kaynaklar: `is-hukuku.pdf` (117 s.), `Ders_Notu_1_VİZE` (31 slayt), `DijitalÇağdaÜretimYönetimi` (134 s.), `DijitalDönüşümdeÜrünYönetimi` (42 s.).
- Her testte `detail: summary`, `max_tokens` ve `fields` kullanıldı; ağır `claims`/`graph` çağrıları ayrı çalıştırıldı.
- Çıktılar `audit_guide --strict` ve `qa_guide` ile doğrulandı.

## 0.2 Test matrisi

| ID | Test | Beklenen | Gerçek | Durum |
|---|---|---|---|---|
| T01 | `qa_guide` ana rehber | 26 sayfa, audit pass | 26, pass | **PASS** |
| T02 | `qa_guide` IRAC | 10–12 sayfa, audit pass | 10, pass | **PASS** |
| T03 | `qa_guide` dijital | 18–20 sayfa, audit pass | 19, pass | **PASS** |
| T04 | `audit_guide` ana rehber | underfilled/overflow yok | yok | **PASS** |
| T05 | `audit_guide` IRAC | underfilled/overflow yok | yok | **PASS** |
| T06 | `audit_guide` dijital | underfilled/overflow yok | yok | **PASS** |
| T07 | `coverage_manifest build` ana rehber | 141 satır | **138**; 4.4.4/10.2 yok | **FAIL** |
| T08 | 141 satır review manifest `validate` | 141/141, empty yok | claimed 141, **empty 141** | **FAIL** |
| T09 | `coverage_manifest build` dijital | 41/41 | 62 satır, claimed 2, missing 60 | **FAIL** |
| T10 | `source_brief` ana rehber | 141 heading, OCR off | 139 heading; 4.4.4/10.2 yok | **PARTIAL** |
| T11 | `source_brief` dijital slayt | 11 low-text, OCR adayı | 11, adaylar var | **PASS** |
| T12 | `section_brief max_tokens=50` | <=50 token | **939 token** | **FAIL** |
| T13 | `heading_brief max_tokens=20` | <=20 token | **939 token** | **FAIL** |
| T14 | `chapter_brief heading=4` | 4 4857... | **1.4 Sosyal Güvenlik** | **FAIL** |
| T15 | `source_topics semantic=true` | model2vec | **heading-terms** | **FAIL** |
| T16 | `claims detail=summary max_tokens=1200` | kısa özet | binlerce satır | **FAIL** |
| T17 | `graph query max_tokens=800` | <=800 token | 20 node + bbox | **FAIL** |
| T18 | `list_guides` IRAC | case_count 10 | **0** | **FAIL** |
| T19 | `guide_status` IRAC | section_count 14 | **0** | **FAIL** |
| T20 | `verify_citations max_claims=5` | snippet claim cümlesi | sayfa başlığı, verified true | **PARTIAL** |
| T21 | `table_extract` dijital s.26 | native tablo | `[]` | **PARTIAL** |
| T22 | `style estimate accessible` | 12–14, over budget true | 12–14, **over_budget false** | **FAIL** |
| T23 | `cards include_definitions` | qa ile aynı | CSV 9, qa 8 | **PARTIAL** |
| T24 | `diagram status` | tek şema | ok/state/status/errors | **PASS** |
| T25 | `graph query "fesih"` | heading node | heading node’lar | **PASS** |
| T26 | `source_insights` dijital s.2 | needs_ocr + reasons | sparse-text, embedded-images | **PASS** |
| T27 | `visual_brief` dijital s.2 | rol + öneri | visual-evidence-required | **PASS** |
| T28 | `citation_snippet` ana s.14 | snippet | alt işveren metni | **PASS** |
| T29 | `cost_report` ana rehber | token/char | 59.535 source token | **PASS** |
| T30 | D2 `doctor` | D2 bulunur | symlink öncesi fail, sonrası pass | **ENV** |
| T31 | IRAC `###` build | 10 sayfa | 10 | **PASS** |
| T32 | Çıktı audit’leri | 26/10/19 | 26/10/19 | **PASS** |

## 0.3 Kullanıcı akışı testi

### Ana rehber

- 26 sayfa; kapak, TOC, A–G, 17 bölüm, 3 D2, 4 figür.
- Şekil 2 sayfa 13’te dikey karar ağacı + dal tablosu + kullanım notu ile dolu.
- C bölümü sayfa 18’de başlık + C.1 tablosu ile başlıyor; başlık kayması yok.
- `qa_guide` audit pass; `coverage` hâlâ **7/7**.
- 141 satır review manifest ayrı dosyada; build tarafından kullanılmıyor.

### IRAC

- 10 sayfa; 10 olay; her olayda `### Sorun/Kural/Uygulama/Sonuç`.
- 0.17.1 `###` beklediği için başlıklar güncellendi.
- Audit pass; sayfa 3 akışı iyi; sayfa 10 kapsam tablosu + cevaplar.
- 5 kart.

### Dijital Dönüşüm

- 19 sayfa; 3 kaynak; 21 low-text; 4 D2; 12 tablo.
- `qa_guide` visual audit 21/21 pass.
- Kaynak sayfa 26 bir **figür** (Şekil 10), native tablo değil; `table_extract []` bu yüzden doğru olabilir. `visual-audit` “table” kararı “figürü tabloya çevir” anlamında; taxonomi net değil.
- 8 kart.

### Stil

- `reader + forest-sage` 10 sayfa.
- `reader + berry` 10 sayfa.
- `accessible + high-contrast` estimate **12–14**; hedef 10–12; `over_budget:false` **hatalı**.
- `style estimate reader` 9–10; doğru.

### Kart / graph / link

- Ana 9 kart (definitions dâhil), IRAC 5, dijital 8.
- `qa_guide.cards` 8; definitions kartını saymıyor.
- `graph` heading node’ları ve query çalışıyor; çıktı büyük.
- `links` `blocks:1/3` hâlâ belirsiz.

---

## 1. 0.17.1’de gerçekten düzelenler

### 1.1 `qa_guide` — tek release gate

Artık tek çağrı:

- build
- coverage
- audit
- visual
- cards
- claims
- diagrams

döndürüyor. Bu, önceki rapordaki en büyük workflow şikâyetini çözüyor.

### 1.2 `audit_guide` — sayfa rolleri ve yeni dedektörler

Yeni audit çıktısı:

- `page_roles`: `front_matter`, `toc`, `chapter_opener`, `body`, `visual`
- `chapter_split_pages`
- `figure_only_pages`
- `table_only_pages`
- `line_chars_plain_p50/p90`
- `font_size_notes` (“1.0 pt metadata” açıklaması)
- `intentional_pages`

Bu, önceki rapordaki TOC/figure/ chapter-split şikâyetlerini ciddi biçimde karşılıyor.

### 1.3 `source_brief` / `source_insights`

- `source_brief`: sayfa, kelime, heading listesi, low-text, OCR politikası.
- `source_insights`: LiteParse complexity, `needs_ocr`, `reasons`, `column_count`, `figure_count`, `structure_nodes`, `vector_graphics`.
- OCR politikası artık doğru: **“off; auto explicit opt-in”**.

Özellikle Dijital Dönüşüm’de 21 low-text sayfanın nedenlerini görmek için güçlü.

### 1.4 `visual_brief` / `citation_snippet` / `table_extract`

- `visual_brief`: düşük metinli sayfa için rol + öneri.
- `citation_snippet`: sayfa + sorgu → snippet.
- `table_extract`: header/rows/cell bbox.

### 1.5 `style estimate`

IRAC için:

```json
{"estimated_pages": "9-10", "over_budget": false, "reason": "2336 words, 1 wrapped tables, 0 diagrams"}
```

Bu, önceki “accessible 14 sayfa” sürprizini büyük ölçüde engeller.

### 1.6 `cards include_definitions` + varsayılan aralıklar

- `include_definitions: true` → ana rehber 9 kart, dijital 8 kart.
- `review_intervals_days: [1,3,7,14,30]` artık varsayılan.

### 1.7 `stats.tables` nesnesi

Artık:

```json
{"markdown": 3, "wrapped": 1, "captioned": 0, "glossary": 0, "total": 4}
```

Bu, önceki “tables: 3” belirsizliğini çözüyor.

### 1.8 `build_guide` incremental raporu

```json
{"changed_headings": [...], "layout_recomputed": true, "rebuilt_sections": 10, "reused_sections": 5}
```

Bu, önceki “her build her şeyi yeniden mi yapıyor?” sorusunu yanıtlıyor.

### 1.9 `diagram status` şema birleşmesi

Artık `ok`, `state`, `status`, `errors`, `warnings`, `schema_version` birlikte ve tutarlı.

### 1.10 `graph` heading node + query

`graph` artık `heading` node’ları ve `query` action’ı veriyor. `query "fesih"` başlıkları döndürüyor.

### 1.11 `doctor` style compatibility

`style-authority` ve `styles` kontrolleri gelmiş; schema-3 `style.aesthetic` otoritesi doğru.

### 1.12 IRAC structure düzeltmesi

0.17.1, IRAC `Sorun/Kural/Uygulama/Sonuç` için **level 3 (`###`)** bekliyor. Bu, önceki “### kabul edilmiyor” sorununun tersi; artık `###` doğru. Rehberi güncelledim; IRAC 11 → 10 sayfa oldu.

---

## 2. 0.17.1’de hâlâ kırık olanlar

## 2.1 Coverage manifest — en kritik

### Ana rehber

`coverage_manifest build`:

- **138 satır** üretti.
- `4.4.4` ve `10.2` hâlâ yok.
- `validate` doğal olarak `claimed: 0`.

Ben 141 satırlık review manifest yazdım:

`guides/is-hukuku/data/coverage-manifest.review.json`

`validate`:

```json
{"claimed": 141, "missing": [], "empty_sections": ["... 141 satır ..."], "passed": false}
```

Yani **tüm alt başlıklar empty** görünüyor.

Kök neden: `ManifestRow`’da `body_chars` yok; `report()` `body_chars.get(&number)` diyor. `number`, kaynak heading numarası (`1.1`, `4.4.3`); `guide_body_chars` ise yalnızca guide’ın `## 1.` … `## 17.` başlıklarını anahtarlıyor. `1.1` anahtarı yok → 0 → empty.

Sonuç:

- Manifest dosyasını `coverage-manifest.review.json` olarak tuttum.
- Ana build legacy **7/7** coverage ile geçiyor.
- `qa_guide` de **7/7** gösteriyor.
- Yani “141/141” iddiası hâlâ gate tarafından doğrulanmıyor.

### Dijital Dönüşüm

`coverage_manifest build`:

- **62 satır** üretti.
- `validate`: `total: 62, claimed: 2, missing: 60`.
- `empty: 0`.

Bu da manifest’in guide heading eşlemesini yapamadığını gösteriyor. Review dosyası olarak ayırdım.

### 10/10 için gereken

1. `ManifestRow`’a `guide_section` veya `body_chars` ekle.
2. `report()`, `body_chars`’ı `row.guide_heading` üzerinden hesaplasın.
3. `coverage_manifest build` manuel eklenen `4.4.4`, `10.2` gibi heading’leri de içersin.
4. `validate` 141/141, `empty_sections: []` versin.
5. `qa_guide` manifest varsa onu kullansın; yoksa legacy’ye düşsün.
6. Dijital gibi çok kaynaklı guide’da her kaynak ayrı manifest + combined summary.

**Kabul:** 141 satır, `claimed=141`, `missing=[]`, `empty=[]`, `passed=true`; `qa_guide.coverage.total=141`.

---

## 2.2 `section_brief` / `heading_brief` token bütçesi yok sayılıyor

Test:

```json
section_brief({heading:"4.4.3", max_tokens:50})
→ chars: 3754, estimated_tokens: 939
```

```json
heading_brief({heading:"4.4.3", max_tokens:20})
→ chars: 3754, estimated_tokens: 939
```

Yani `max_tokens` yalnızca etiket; çıktıyı kesmiyor.

**10/10 için:**

- `max_tokens` gerçekten uygulanmalı.
- `truncated: true`, `next_offset`, `full_chars` dönmeli.
- `section_brief` 150–300, `heading_brief` 80–150, `chapter_brief` 300–600 token hedefi.
- `max_tokens` aşılırsa `E_BRIEF_BUDGET` veya `truncated` uyarısı.

**Kabul:** `max_tokens: 50` → `estimated_tokens <= 50` veya `truncated: true`.

---

## 2.3 `chapter_brief` yanlış eşleşiyor

Test:

```json
chapter_brief({heading:"4", max_tokens:200})
→ heading: "1.4. Sosyal Güvenlik Hukuku"
```

`4` → `1.4` eşleşmesi kabul edilemez.

**10/10 için:**

- Önce tam eşleşme: `4`, `4.`, `4 `, `4 4857...`.
- Sonra prefix; fuzzy skor yalnızca fallback.
- Birden çok eşleşmede `ambiguous: true` + candidates döndür.
- `heading` yerine `heading_id` kullanılabilsin.

**Kabul:** `heading:"4"` → `4 4857...` veya ambiguity; asla `1.4`.

---

## 2.4 `source_topics semantic:true` hâlâ lexical

Test:

```json
source_topics({semantic:true})
→ method: "heading-terms"
```

Yani Model2Vec semantik kümeleme devrede değil.

**10/10 için:**

- `semantic:true` → `method: "model2vec"` veya `"model2vec+heading"`.
- Model yoksa `fallback: "heading-terms"` + `warning`.
- Deterministik seed/tie-break.
- `k` küme sayısı; her küme için temsilci heading + page range + terms.

**Kabul:** `semantic:true` → `method` `model2vec`; aynı girdi aynı kümeler.

---

## 2.5 `claims` ve `graph query` detail/max_tokens yok sayıyor

`claims`:

- `detail:"summary"`, `max_tokens:1200` verildi.
- Binlerce satır döndü.

`graph query`:

- `max_tokens:800` verildi.
- 20 node + bbox’lar döndü.

**10/10 için:**

- `detail:"summary"` varsayılan; `full` opt-in.
- `max_tokens` kesin uygulanmalı.
- `omitted`, `total`, `next` alanları.
- `claims` için `max_claims`, `min_severity`; `graph` için `node_limit` zaten var ama output budget yok.

**Kabul:** `claims` summary < 2.000 token; `graph query` < 1.000 token; aşarsa `truncated`.

---

## 2.6 `list_guides` / `guide_status` IRAC metrikleri tutarsız

- `list_guides`: `case_count: 0` — 10 olay var.
- `guide_status`: `section_count: 0` — 14 section görünüyor.
- `content_words`: 3223 / 2347 farklı.

**10/10 için:**

- IRAC için `case_count` = `## Olay` sayısı.
- `section_count` = guide h2/h3 sayısı; `list_guides` ile aynı.
- `content_words` tek kaynaktan.
- `guide_kind` `irac` ise `case_count` zorunlu.

**Kabul:** IRAC `case_count=10`, `section_count=14`; iki araç aynı.

---

## 2.7 `verify_citations` snippet’i kaba

Bir claim için snippet:

> “İş ve Sosyal Güvenlik Hukuku - Birleştirilmiş Ders Notları ...”

yani sayfa başlığı. `verified: true` dedi.

**10/10 için:**

- Snippet, claim’deki anahtar terimleri içeren **en iyi cümle** olmalı.
- `verified`, snippet’in claim’i desteklemesine bağlı olmalı; sadece sayfa eşleşmesi yetmez.
- `confidence` ve `match_terms` döndür.
- Header/footer metni snippet olarak seçilmemeli.

**Kabul:** `verify_citations` snippet’i claim’i destekleyen cümle; `verified=false` yanlış eşleşmede.

---

## 2.8 D2 PATH sorunu

`doctor` başta:

> “set FOLISIFT_D2 or install d2 for diagram build”

D2 sidecar’da vardı (`~/.local/lib/folisift/d2`), PATH’te yoktu. Symlink ile düzeldi.

**10/10 için:**

- Folisift, sidecar D2’yi otomatik bulmalı.
- Bulamazsa `doctor` net komut vermeli: `export FOLISIFT_D2=...`.
- Kurulum dokümanında PATH/sidecar davranışı yazmalı.

---

## 2.9 `visual-audit` “table” kararı ile `table_extract` uyuşmuyor

Dijital Dönüşüm, kaynak sayfa 26:

- `visual-audit` kararı: **table** (“Şekil 10’daki özel satın alma hedefleri listesi tabloya dönüştürüldü”).
- `table_extract` sonucu: **`tables: []`**.

Kaynak sayfayı görsel olarak kontrol ettim: sayfa 26 bir **figür** (Şekil 10, kutu listesi), native tablo değil. Yani `table_extract []` doğru; `visual-audit` “table” kararı “figürü tabloya çevir” anlamında.

Sorun: `visual-audit` decision taxonomisi `table` ile `redraw/figure` arasındaki farkı net ayırmıyor.

**10/10 için:**

- `visual-audit` decision: `table` = native table; `redraw` = figure-to-table/visual redraw.
- `table_extract`, `tables: []` dönerken `reason: "no table block"` / `table_likely` versin.
- `source_insights.complexity.layout.ruled_table_count` ile çapraz kontrol.
- Gerekirse `table_extract` `mode: "native" | "layout"` seçeneği.

---

## 2.10 `style estimate` over_budget’i yanlış hesaplıyor

IRAC için:

```json
style: "accessible", target_pages: "10-12"
→ estimated_pages: "12-14", over_budget: false
```

`12–14`, `10–12` hedefinin üstünde; `over_budget` **true** olmalı.

**10/10 için:**

- `over_budget`, tahminin **üst sınırı** hedefin üst sınırını aşıyorsa true olmalı.
- `estimated_pages` tek sayı değil aralık; `over_budget_reason` versin.
- `accessible` için “+2–4 sayfa” uyarısı.

**Kabul:** `accessible`, 10–12 hedefi → `over_budget: true`, `warning`.

---

## 2.11 `cards` sayısı `qa_guide` ile uyuşmuyor

- `cards include_definitions: true` → 9 kart.
- `qa_guide.cards.count` → 8.

`qa_guide` definitions kartlarını saymıyor.

**10/10 için:**

- `qa_guide.cards` içinde `recall`, `definitions`, `total` ayrımı.
- `include_definitions` policy’si qa’da da görünmeli.

---

## 2.12 IRAC scaffold / structure

0.17.1 `###` bekliyor; rehberi güncelledim. Ama `prepare_guide` hâlâ IRAC için doğru iskelet üretmiyor:

- 60+ kaynak başlığı `##` olarak geliyor.
- `sources: []`.
- IRAC bölümleri sona ekleniyor.

**10/10 için:**

- `irac` scaffold: 10 case placeholder, her case `## Olay`, `### Sorun/Kural/Uygulama/Sonuç`.
- Kaynak başlıkları `source-headings.md`.
- `sources` frontmatter otomatik.
- `force` olmadan mevcut guide’ı ezme.

---

## 3. Güncellenen çıktılar

| Dosya | Sayfa | Durum |
|---|---:|---|
| `output/is-hukuku.pdf` | 26 | audit geçti |
| `output/is-hukuku-irac.pdf` | 10 | audit geçti |
| `output/dijital-donusum-temelleri.pdf` | 19 | audit geçti |
| `output/is-hukuku-irac-forest-sage.pdf` | 10 | audit geçti |
| `output/is-hukuku-irac-berry.pdf` | 10 | audit geçti |
| `output/is-hukuku-cards.csv` | 9 kart | güncellendi |
| `output/is-hukuku-irac-cards.csv` | 5 kart | güncellendi |
| `output/dijital-donusum-temelleri-cards.csv` | 8 kart | güncellendi |

IRAC 11 → 10 sayfa: `###` structure düzeltmesi.

Ana rehber coverage hâlâ **7/7**; 141 review manifest ayrı dosyada:

`guides/is-hukuku/data/coverage-manifest.review.json`

Dijital review manifest:

`guides/dijital-donusum-temelleri/data/coverage-manifest.review.json`

Bunlar build tarafından kullanılmıyor; manifest bug’ı düzelince kullanılmalı.

---

## 4. 10/10 puan kartı

| Alan | Puan | Neden |
|---|---:|---|
| Build / PDF üretimi | 9.5 | Hızlı, deterministik, staging, incremental |
| Audit / layout | 9.0 | Page roles, chapter split, figure-only, font notes |
| QA tek çağrı | 9.5 | `qa_guide` büyük kazanım |
| Coverage | 4.0 | 7/7 vs 141; manifest alt başlıklarda bozuk |
| Source/context economics | 6.5 | `source_brief` iyi; `section_brief` token bütçesi bozuk |
| D2 | 8.0 | Status/şema iyi; PATH ve width/decision hâlâ sınırlı |
| Style | 8.5 | `estimate` iyi; `restyle`/preview hâlâ sınırlı |
| API tutarlılığı | 8.0 | Çok iyi; bazı metrik/limit tutarsızlıkları |
| Determinism | 9.0 | Yerel, cache, hash; OCR/reranker politika netleşmeli |
| Dokümantasyon | 8.0 | İyi; IRAC `###`, D2 PATH, manifest bug eksik |

**Ağırlıklı:** ~8.0/10.

---

## 5. 10/10 için öncelik sırası

### P0 — 0.17.2 hotfix

1. **Coverage manifest body mapping**
   - `body_chars` `guide_heading` üzerinden.
   - 141/141, `empty=[]`.
   - `qa_guide` manifest kullansın.
2. **`section_brief` / `heading_brief` token budget**
   - Gerçekten kes; `truncated`, `next_offset`.
3. **`chapter_brief` exact match**
   - `4` → `1.4` hatası.
4. **`source_topics semantic:true`**
   - Model2Vec gerçekten çalışsın; fallback uyarısı.
5. **`claims` / `graph` detail/max_tokens**
   - Summary varsayılan; kesin budget.
6. **IRAC metrikleri**
   - `case_count=10`, `section_count=14`.
7. **`verify_citations` snippet**
   - Header değil, claim cümlesi.

### P1 — 0.18

8. `restyle_guide` / `style estimate` tam entegrasyon.
9. `prepare_guide` IRAC scaffold.
10. `table_extract` false-negative reason.
11. D2 PATH auto-discovery.
12. `cards` qa count ayrımı.
13. `source_brief` 141 heading (4.4.4/10.2).
14. `qa_guide` cost report + token metrics.

### P2 — 1.0

15. D2 `decision` kind / 20+ node / width / vertical.
16. LiteParse `source_insights` full entegrasyon (vector/image/structure).
17. OCR auto policy + confidence/manual review.
18. Graph `path/explain` bounded output.
19. Docs cookbook: IRAC `###`, manifest, D2 PATH, `flow`, `subs`, `table` numbering.
20. Determinism manifest: versions/hashes/seed.

---

## 6. 10/10 kabul testleri

| Test | Beklenen |
|---|---|
| Ana rehber manifest | `total=141`, `claimed=141`, `missing=[]`, `empty=[]` |
| `qa_guide` ana rehber | `coverage.total=141` |
| Dijital manifest | 3 kaynak, her biri nonzero; combined 41 |
| `section_brief max_tokens=50` | `estimated_tokens<=50` veya `truncated=true` |
| `heading_brief max_tokens=20` | aynı |
| `chapter_brief heading=4` | `4 4857...` veya ambiguity; `1.4` değil |
| `source_topics semantic=true` | `method=model2vec` |
| `claims detail=summary` | < 2.000 token |
| `graph query max_tokens=800` | < 1.000 token |
| IRAC list | `case_count=10`, `section_count=14` |
| `verify_citations` | snippet claim cümlesi; header değil |
| `style estimate accessible` | `over_budget=true`, 12–14 vs 10–12 |
| `visual-audit` | `table` native, `redraw` figure; ayrı |
| `cards include_definitions` | qa’da `definitions` ayrı |
| IRAC scaffold | 10 case × 4 `###` |
| D2 | 20 node, dikey, width 60% bbox değişir |
| `restyle_guide` | `content_hash` değişmez |
| `qa_guide` | tek çağrıda build+coverage+audit+visual+cards+claims+diagrams+cost |
| Determinism | iki run aynı manifest/brief/cluster |

---

## 7. Sonuç

0.17.1, “çalışan araç”tan “güvenilir üretim hattı”na giden yolda büyük adım.

Ama 10/10 için son on iki şey şart:

1. **Coverage manifest’i gerçekten 141/141 yap.**
2. **Brief token bütçelerini gerçekten uygula.**
3. **`chapter_brief` exact match.**
4. **`source_topics` semantic gerçekten çalışsın.**
5. **`claims`/`graph` çıktısını bütçele.**
6. **IRAC metriklerini düzelt.**
7. **`verify_citations` snippet’ini iyileştir.**
8. **D2 PATH’i otomatik bul.**
9. **`style estimate` over_budget’i düzelt.**
10. **`visual-audit` table/redraw taxonomisini ayır.**
11. **`cards` sayısını qa’da definitions ile göster.**
12. **IRAC scaffold’u 10 case üretsin.**

Bunlar düzelirse:

- Ana rehber 141/141,
- IRAC 10 olay,
- Dijital 41/41,
- `qa_guide` tek gerçek release gate,
- agent maliyeti ölçülebilir ve düşük,
- çıktı deterministik

olur. O zaman **10/10** derim.
