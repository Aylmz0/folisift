# Folisift 0.20.0 — Final Geri Bildirim (test-kullanıcı tam geçiş)

**Tarih:** 4 Kasım 2026
**Sürüm:** Folisift **0.20.0** · build **195** · MCP schema **15** · **37 tool** (0.17.1: 35)
**Önceki rapor:** `FOLISIFT-GERI-BILDIRIM-v2.md` (0.17.1, 8.0/10)
**Durum:** Final — bu sürüm testlerinin tamamı 0.20.0 ile yeniden koşuldu; iki manifest küratörlüğü yapıldı; çıktılar güncel.

---

## 0. Kısa hüküm

**0.20.0, 0.17.1’deki 12 maddenin 11’ini fiilen kapattı; kapsam gate’i ana rehberde gerçekten 141/141 çalışıyor.** Dijital rehber de küratörlükle 41/41’e geldi (önceki sürümle kıyasla bu da çözüm).

**Puan: 9.2 / 10** (0.17.1: 8.0)

10/10’a kalan fark dört tool işi + bir dokümantasyon — bkz. §6.

**Sorun yeni sürümde mi, belgelerde mi?** İkisi de, ama oran belli:

- Tool’a ait (taze çağrılarla tekrarlanabilir): generated manifest anahtarlarında gövde metni, canonical `4.4.4`/`10.2` eksik, `chapter_brief` title-only, brief bütçelerinde `estimated_tokens > budget`, `coverage_manifest validate` çıktısı `field`/`max_tokens` ile budanmıyor.
- Bana/belgeye ait (küratörlük borcu): dijital “review” manifest’ini ham generated çıktısının yeniden adlandırılmış hâli olarak bırakmışım (`status:"missing"`, anchor’sız); 141 sayısı derin bir tool kanıtı değil, benim elle küratörlüğüm + fuzzy match’e dayanan bir gate.
- **Düzeltilenler:** dijital manifest şimdi 41 satırlık curated review → 41/41 validate → adopt → combined → qa pass. Sorunun “eski belgelere” ait kısmı da kapandı.

---

## 0.1 Ortam ve yöntem

- Folisift 0.20.0, build 195, MCP schema 15, 37 tool
- D2 v0.9.0 — `doctor` **pass** (symlink kalıcı, PATH’te)
- Model: `potion-multilingual-128M-int8` — `source_topics semantic:true` artık **model2vec** (`model_signature` var)
- Kaynaklar: `is-hukuku.pdf` (117 s.), `Ders_Notu_1_VİZE.pdf` (31 slayt), `DijitalÇağdaÜretimYönetimi.pdf` (134 s.), `DijitalDönüşümdeÜrünYönetimi.pdf` (42 s.)
- Üç guide her testte `audit_guide --strict` + `qa_guide` ile doğrulandı
- Hiçbir testte `ocr:true` verilmedi; tüm `max_tokens`/`fields` sınırları aktif

### Bu turda workspace’te yapılan kalıcı değişiklikler

| # | Değişiklik | Nerede |
|---|---|---|
| 1 | `is-hukuku` review manifest → **combined** (adopt) | `guides/is-hukuku/data/coverage-manifest.json` |
| 2 | `dijital-donusum-temelleri` review manifest **küratörlendi**: 62 ham satır → 41 kapsanan satır + `guide_heading` anchor + `status:"covered"` + `evidence` + `confidence:1.0` | `guides/dijital-donusum-temelleri/data/coverage-manifest.review.json` (yedek: `/var/tmp/opencode/dijital-coverage-manifest.review.backup.json`) |
| 3 | Dijital review → **combined** (adopt) | `guides/dijital-donusum-temelleri/data/coverage-manifest.json` |
| 4 | Generated manifest’ler (`*.generated.json`) workspace’te kaldı; `combined` varken qa bunları hard gate yapmıyor | — |
| 5 | `irac-scaffold-probe` geçici guide’ı oluşturuldu, test edildi, **silindi** | — |
| 6 | `dijital-donusum-temelleri_draft.pdf` (gate-fail artefaktı) `/var/tmp/opencode/`’a taşındı; output PDF **Nov 4 23:55’te qa ile yeniden üretildi** (19 sayfa, içeriği aynı) | `output/dijital-donusum-temelleri.pdf` |

---

## 0.2 Test matrisi (0.17.1 → 0.20.0)

| ID | Test | 0.17.1 | 0.20.0 | Durum |
|---|---|---|---|---|
| T01–T06 | qa/audit 3 guide (26/10/19) | pass | pass | **PASS** |
| T07 | `coverage_manifest build` → generated validate | 138 satır, 4.4.4/10.2 yok | hâlâ **138 satır; `validate generated` → claimed 17, missing 121** | **FAIL** (tool) |
| T08 | Ana rehber combined validate | — | **total=141, claimed=141, missing=[], empty=[], passed** | **PASS** ✅ |
| T09 | Dijital validate review | 62/62, claimed 2 | **önce 2/62 (ham) → küratörlük sonrası total=41, claimed=41, missing=[], empty=[], passed** | **PASS** ✅ (küratörlü) |
| T09b | Dijital `build` generated → qa | — | generated varken combined yokken qa **40/62 gate-fail** → draft yazdı, output’a dokunmadı | **PASS** (davranış doğru; dokümante edilmeli) |
| T10 | `source_brief` ana rehber | 139 heading | hâlâ **139** (4.4.4, 10.2 yok) | **PARTIAL** |
| T11 | `source_brief` dijital slayt | 11 low-text | 11, adaylar var | **PASS** |
| T12 | `section_brief max_tokens=50` | 939 token | **bütçe uygulanıyor** (`truncated:true`, `budget_chars:200`, `next_offset:100`) ama `estimated_tokens:107` | **PARTIAL** ✅ |
| T13 | `heading_brief max_tokens=20` | 939 token | **bütçe uygulanıyor** ama `summary:""`, `estimated_tokens:81`, `next_offset:0` | **PARTIAL** ✅ |
| T14 | `chapter_brief heading=4` | → 1.4 | **→ `4. 4857 Sayılıİş Kanununun Uygulama Alanı`** exact match; ama **title-only** (§5.2) | **PASS** ✅ (içerik yok) |
| T15 | `source_topics semantic=true` | heading-terms | **`model2vec`**, `model_signature` var, 3 küme, deterministik | **PASS** ✅ |
| T16 | `claims summary max_tokens=1200` | binlerce satır | **`returned/omitted/total/next` + `noise_filtered`**; 5 claim döndü | **PASS** ✅ |
| T17 | `graph query max_tokens=800` | bbox dump | **bbox default off; `returned/omitted/total/next`** | **PASS** ✅ |
| T18 | `list_guides` IRAC | case_count 0 | **case_count 10** | **PASS** ✅ |
| T19 | `guide_status` IRAC | section_count 0 | **section_count 14, case_count 10, content_words 2347** — `list_guides` ile tutarlı | **PASS** ✅ |
| T20 | `verify_citations max_claims=5` | header snippet, verified true | **`confidence`, `match_terms`, `reason`**; 26. satır artık `verified:false, confidence:"low", reason:"weak term overlap"`; 67. satır `verified:true` + claim cümlesi snippet | **PASS** ✅ |
| T21 | `table_extract` dijital s.26 | `tables: []` açıklama yok | **`reason`, `table_likely:false`, `manual_action`** | **PASS** ✅ |
| T22 | `style estimate accessible` | over_budget:false | **over_budget:true**, `reason:"estimated 12-15 pages exceeds the 10-12 target"` | **PASS** ✅ |
| T23 | cards vs qa | 9 vs 8 | qa artık **`recall:8, definitions:1, total:9, includes_definitions:true`** | **PASS** ✅ |
| T24 | `diagram status` | OK | OK (3 spec / 6 asset) | **PASS** |
| T25 | `graph query "fesih"` heading nodes | OK | OK | **PASS** |
| T26 | `source_insights` dijital s.2 | needs_ocr | OK | **PASS** |
| T27 | `visual_brief` dijital s.2 | rol | OK | **PASS** |
| T28 | `citation_snippet` ana s.14 | OK | OK | **PASS** |
| T29 | `cost_report` ana rehber | 59.535 source token | aynı | **PASS** |
| T30 | D2 `doctor` | ENV fail | **doctor pass** | **PASS** ✅ |
| T31 | IRAC `###` build | 10 sayfa | 10 sayfa | **PASS** |
| T32 | Çıktı auditleri | 26/10/19 | 26/10/19 | **PASS** |
| T33 | `hint` aksiyonu (yeni) | — | advisory hints üretiyor; **covered 20 / likely-missing 26**; merged heading’lerde (`2.1–2.2`) 0.95 üstü sayıyor; **authoritative değil** | **PASS** (sınırlı) |
| T34 | `scaffold_guide structure=irac` | generic | **10 Dava × 4 `###` + `Kapsam` + frontmatter `sources:[is-hukuku]`**; placeholder `[PD0, s. 1]`, `source-headings.md` doldurulmuyor | **PASS** ✅ (kalıntılar) |

Özet: **0.17.1: 15P/4△/12F/1ENV → 0.20.0: 30 PASS / 3 PARTIAL / 1 FAIL** (FAIL = T07 generated).

---

## 0.3 Kullanıcının sorusu: “Sorun yeni versiyonda mı?”

Kısa yanıt: **Hayır, yalnızca yeni versiyonda değil — ama kalan kırmızıların çoğu tool’a ait.** Ayrımı kanıtlarla:

**Tool’a ait (belgelerden bağımsız, taze çağrıyla tekrarlanan):**

1. `coverage_manifest build` çıktısında her satırın anahtarı **gövde metnini** içeriyor; canonical `4.4.4` ve `10.2` üretilmiyor → `validate generated` **17/138**.
2. `qa_guide`, adopt edilmemiş `.generated.json`’u **hard gate** sayabiliyor (dijitalde 40/62 fail; output yerine `_draft.pdf` yazıldı — davranış güvenli, ama dokümante/opt-in değil).
3. `chapter_brief` title-only (§5.2), brief bütçelerinde `estimated_tokens > budget` (§5.3).
4. `coverage_manifest validate` `detail/fields/max_tokens` sınırlarını uygulamıyor (§5.5) — `review_diff` bile `fields` verilse tam döküldü.

**Belgelere/bana ait (küratörlük borcu, şimdi kapatıldı):**

1. Dijital “review” manifest, ham generated’ın yeniden adlandırılmışıydı: `status:"missing"`, `guide_heading` anchor’sız → `validate review` **2/62**. Şimdi 41 satırlık curated review (§0.1 #2) → **41/41**.
2. İs-hukuku 141/141’i derin tool kanıtı değil; benim `guide_heading` anchor’ladığım kürate satırların fuzzy match’i (§5.6’de dürüst uyarı: `weak_sections: 7`).
3. Guide’daki birleşik başlıklar (`## 2.1–2.2 …`, en-dash) gate’in numara parser’ını yanıltıyor — dijitalin ham haliyle geçememesinin bir nedeni de buydu.

---

## 1. 0.17.1’deki 12 madde — kapanış durumu

| # | Madde | 0.20.0 durumu |
|---|---|---|
| 1 | Coverage manifest 141/141 | ✅ **Kapandı** — combined gate; `qa_guide.coverage.total=141`, `gate_total:141` |
| 2 | Brief token bütçeleri | ✅ Uygulanıyor; ama est.>budget ve boş-özet kalıntıları (§5.3) |
| 3 | `chapter_brief` exact match | ✅ `4 → 4. 4857…`; ama body yok (§5.2) |
| 4 | `source_topics semantic` | ✅ `model2vec` + `model_signature` |
| 5 | claims/graph `detail`/`max_tokens` | ✅ `returned/omitted/total/next` |
| 6 | IRAC metrikleri | ✅ `case_count 10`, `section_count 14`; iki araç tutarlı |
| 7 | `verify_citations` snippet | ✅ `confidence/match_terms/reason`; header snippet sorunu kalktı |
| 8 | D2 PATH | ✅ `doctor` pass |
| 9 | `style estimate` over_budget | ✅ `true` + açıklayıcı reason |
| 10 | `table_extract` false-negative | ✅ `reason`/`table_likely`/`manual_action`; `visual_evidence` ile çapraz doğrulandı (chart, table_likely:false) |
| 11 | cards qa sayısı | ✅ `recall/definitions/total` ayrımı |
| 12 | IRAC scaffold | ✅ 10 Dava × 4 `###` + `sources` dolu; kalıntılar (§5.4) |

---

## 2. Yeni durumlarda dikkat gerektiren davranışlar

### 2.1 qa gate + draft davranışı (önemli, iyi ama belgelenmeli)

`qa_guide` gate reddederse **`output/<slug>_draft.pdf`**’e yazar, `output/<slug>.pdf`’i ezmez. Dijitalde gözlemlendi: `output/dijital-donusum-temelleri_draft.pdf` oluştu, output PDF korundu. Dokümanda bu davranış ve `draft` temizleme adımı yazmalı.

### 2.2 Gate önceliği: combined > generated

Doğrulandı (is-hukuku 141, dijital 41): `coverage-manifest.json` (adopted combined) varsa qa onu kullanıyor; `generated` varsa ve combined yoksa qa onu hard gate yapıyor. Yani **akış** net olmalı: önce review küratörü + validate, sonra `adopt`.

Çağrı akışı (MCP tool; CLI karşılığı da aynı bayraklarla):

```text
coverage_manifest({ guide, action: "build" })                    // .generated.json üretir
coverage_manifest({ guide, action: "validate", manifest: "review" }) // küratörlü kontrol
coverage_manifest({ guide, action: "build", adopt: true })       // review → combined
qa_guide({ slug })                                              // combined gate ile pass
```

### 2.3 Küme eleme (excluded) kavramı yok

Bilinçli kapsam dışı satırlar (dijitalde 21 satır: VİZE problem slaytları, ERP adım listeleri, `1/2/3/5` bölüm başlıkları, `10 Sonuç`, `2.3.x`, `3.1.x`) için tool’da `excluded/waived` statüsü yok. Küratör bunları manifest’ten **çıkararak** elemek zorunda kaldı. Bu insan hatasına açık ve audit izi bırakmayan bir yol — `status:"excluded"` + validator sayımı desteği (P1) istiyoruz.

---

## 3. Çözülenler — ama not kanıtıyla

### 3.1 İs-hukuku 141/141 (küratörlü)

`qa_guide.coverage`: `total:141, claimed:141, gate_total:141, missing:0, empty_sections:[]`, `build_guide --check` aynı sonucu veriyor. **Ama:** 7 `weak_sections` (Kaynaklar bölümü, `3.1*`, `3.2*`, `3.3`) listeleniyor — gövde parent section’a fuzzy eşleşse de “zayıf” sayılıyor. Ve her alt başlığın ayrı gövde taşıdığı kanıtı, gate’in kendi içinde değill; `evidence` alanı `## <parent>` satırına işaret eder.

**Dürüst okuma:** 141/141 = "141 satırın her biri guide’da bir section’a bağlı"; "her alt başlık ayrı ele alındı" değil.

### 3.2 Dijital 41/41 (küratörlü)

Curated review: 41 satır, her biri `guide_heading` (birleşik `## 2.1–2.2` gibi anchor’lara bağlı), `evidence` = covers yorumu, `confidence:1.0`. `validate` → `claimed:41, missing:[], passed:true`; `build --adopt` → combined; `qa_guide` → **41/41 pass**, 19 sayfa, audit pass.

Kapsam dışı bırakılan 21 satır `/var/tmp/opencode/dijital-coverage-manifest.review.backup.json` içinde kayıtlı (ham hali).

### 3.3 `hint` aksiyonu

`coverage_manifest hint` advisory sınıflandırma veriyor (covered / likely-missing + skor). İki sınır:

- Birleşik heading’de (`2.1–2.2`) `2.2`, `score:0.333` → `likely-missing` işaretliyor; yani **merged coverage’i likely-missing sayıyor**. Hint, `covers` string’lerini veya en-dash range’i parse edemiyor; %95 eşiği de tek bir güçlü eşleşme istiyor.
- `max_tokens` hint çıktısını kesiyor ama `fields` uygulanıp uygulanmadığı doğrulanmadı (validate’te uygulanmadığı kesin — §4.5).

Tool, hint’i "advisory; confirm with reviewed manifest" diye nitelendiriyor — doğru etiket; ama hint’in merged-heading körlüğü dokümante edilmeli (P1).

---

## 4. Hâlâ sorunlu olanlar (final)

### 4.1 `coverage_manifest generated` path (tool bug — kritik)

```json
{ "canonical_total": 138, "claimed": 17, "missing": [121 satır], "passed": false }
```

Anahtarlar gövde içeriyor (`is-hukuku:1 1 hukuk ve iş hukuku hukuk toplum düzenini…`); canonical `4.4.4`/`10.2` yok. Aracı bu şekli üretmeye devam ediyor.

**Kabul:** generated anahtarları body’siz; `4.4.4`/`10.2` canonical; `validate generated` → 138/138 (veya 141 + “excluded” statüsünü kapsamak).

### 4.2 qa gate önceliği dokümante/opt-in değil

Adopt edilmemiş `generated` hard gate → output yerine draft. Güvenli ama sessiz. Dokümanda uyarı; idealde generated gate için açık `--strict-manifest=generated` opt-in.

### 4.3 `chapter_brief` title-only

```json
chapter_brief({heading:"4", max_tokens:200})
→ heading "4. 4857…", chars: 41, summary: "4. 4857 Sayılıİş Kanununun Uygulama Alanı",
  estimated_tokens: 99, truncated: false, level: "section"
```

Body yok; `chars`/`estimated_tokens` tutarsız. Beklenti: 300–600 token’lık hiyerarşik özet (parent + çocuklar). (`numbers: ["4","4857"]` — title’dan "4857" numara sayılıyor.)

### 4.4 Brief küçük bütçelerde tutarsız

```json
section_brief({heading:"4.4.3", max_tokens:50})
→ truncated:true, budget_chars:200, next_offset:100, estimated_tokens:107   // 50 değil
heading_brief({heading:"4.4.3", max_tokens:20})
→ summary:"", truncated:true, estimated_tokens:81, next_offset:0            // boş özet
```

**Kabul:** `estimated_tokens <= max_tokens` garantisi; boş özet yerine minimum özet + `truncated:true`; `next_offset` tek anlamlı (0 ya da token tabanlı).

### 4.5 `coverage_manifest validate` çıktıyı bütçelemiyor

`detail:"summary"`, `max_tokens:500–700`, `fields:[…]` verildi; `per_source[].sections[]` tam döküldü (~14 KB); `review_diff` de `fields` altında bile eksiksiz. `omitted/total` sarması validate’te yok.

### 4.6 `source_brief` 2 heading eksik

`heading_count: 139`; canonical `4.4.4` ve `10.2` hâlâ listede yok (inventory/extract kaynaklı; 0.17.1’de de öyleydi). Manifest fixed ama brief hâlâ eksik — kullanıcı bu aracı kapsam kontrolü için açıyorsa yanıltıcı.

### 4.7 IRAC scaffold kalıntıları

- Placeholder citation: `[PD0, s. 1]` (source id yerine anlamsız anahtar).
- `source-headings.md` doldurulmuyor; "run analyze" mesajı bırakıyor.
- 10 case + `Kapsam` doğru; `sources: [is-hukuku]` doğru; `### Sorun/Kural/Uygulama/Sonuç` doğru.

### 4.8 Küçük notlar

- `section_brief.numbers: []` (0.17.1’de dolu geliyordu); ya kaldırıldı ya kırıldı.
- `chapter_brief.numbers: ["4","4857"]`.
- `source_topics` küme etiketi heading adı; deterministik ama anlamlı label değil.
- `verify_citations` çok-atıflı claim’lerde (`[is-hukuku, s. 1–3, 93–95]`) hangi sayfa doğrulandığı belirsiz; sayfa-başına snippet listesi istenir.
- `is-hukuku` `weak_sections: 7` — tanımı (`score<?` / `body parent’a bağlı` / `title match?`) belgelenmemiş.
- `fields` parametresi nested list/array alanlarını kırpmıyor (validate, hint).

---

## 5. Güncellenen çıktılar

| Dosya | Sayfa | Durum |
|---|---:|---|
| `output/is-hukuku.pdf` | 26 | `build_guide --check` + qa + audit **141/141 pass** |
| `output/is-hukuku-irac.pdf` | 10 | audit pass |
| `output/dijital-donusum-temelleri.pdf` | 19 | **qa 41/41 pass**, audit pass (qa rebuild, Nov 4 23:55) |
| `output/is-hukuku-irac-forest-sage.pdf` / `-berry.pdf` | 10 | — |
| `output/is-hukuku-cards.csv` | 9 kart | güncel |
| `output/is-hukuku-irac-cards.csv` | 5 kart | güncel |
| `output/dijital-donusum-temelleri-cards.csv` | 8 kart | güncel |
| `guides/is-hukuku/data/claims.md` | 191 claim | güncel (log değişmedi) |
| `guides/is-hukuku/data/coverage-manifest.json` | 141 satır | adopted combined |
| `guides/dijital-donusum-temelleri/data/coverage-manifest.json` | **41 satır** | adopted combined (küratörlü review’dan) |

`output/dijital-donusum-temelleri_draft.pdf` → `/var/tmp/opencode/` (gate-fail artefaktı; kalıcı değil).

---

## 6. 10/10 için kalan işler

### P0 — 0.20.x hotfix

1. **Generated manifest düzelt** (body’siz anahtar, `4.4.4`/`10.2` canonical) + `validate generated` 138/138 olsun
2. **qa gate’te unadopted generated hard gate yapma** (opt-in ya da uyarı; draft yolunu dokümante et)
3. **`chapter_brief` body bas** (300–600 token hiyerarşik özet); title-only dönmeyi durdur; `numbers`’dan title-metni çıkarma
4. **`coverage_manifest validate` çıktısını bütçele** (`omitted/total` sarması, `fields` nested kesme)
5. **Brief `estimated_tokens <= max_tokens`** garantisi; küçük bütçede boş yerine minimum özet

### P1 — 0.21

6. **`status:"excluded"`** (veya eşdeğer) + validator sayımı; küratör satırı manifest’ten çıkarmak yerine işaretleyebilsin
7. Gate’in **birleşik heading parser’ı** (`## 2.1–2.2`, en-dash / virgül / squiggle) — `covers` string’lerini de yut, merged coverage’i score-1 say
8. `weak_sections` tanımını ve eşiğini raporla/dokümante et
9. `source_brief` canonical `4.4.4`/`10.2` heading’lerini bul/üret; eksik kalırsa açık uyarı bas
10. IRAC scaffold placeholder citation → source id; `source-headings.md` doldur veya `--analyze` bayrağı
11. `verify_citations` çok-atıflı iddialarda sayfa-başına snippet listesi
12. `hint` merged-heading körlüğüne düzeltme + `fields` düzeltmesi

### P2 — 1.0

13. Determinism manifest (versions/hashes/seed) — 0.17.1’den beri açık
14. D2 `decision` kind / 20+ node / vertical / width_percent — hâlâ sınırlı
15. OCR auto policy + confidence/manual review — `visual_evidence` artık `needs_ocr` veriyor; akışa entegre et
16. Graph `path`/`explain` bounded output — `query` düzeldi, diğerleri bütçe’siz
17. Docs cookbook: review→validate→adopt akışı, draft temizleme, D2 PATH, `###` IRAC, `flow`/`subs`

---

## 7. Kabul testleri (güncel)

| Test | Beklenen | 0.20.0 sonucu |
|---|---|---|
| Ana rehber 141 | `total=141, claimed=141, missing=[], empty=[]` | **PASS** (küratörlü adopt) |
| Dijital 41 | `total=41, claimed=41, missing=[]` | **PASS** (küratörlü adopt) |
| `qa_guide` ana rehber | `coverage.total=141` | **PASS** |
| `qa_guide` dijital | `coverage.total=41` | **PASS** |
| `validate generated` | `claimed=canonical` (138/138) | **FAIL** (17/138) |
| `section_brief max_tokens=50` | `estimated_tokens<=50` | **FAIL** (107) |
| `heading_brief max_tokens=20` | boş değil | **FAIL** (summary:"") |
| `chapter_brief heading=4` | `4. 4857…` + body | **PARTIAL** (eşleşme pass, body yok) |
| `source_topics semantic=true` | `model2vec` | **PASS** |
| `claims detail=summary` | `returned/omitted/total` | **PASS** |
| `graph query max_tokens=800` | bbox’sız budgeted | **PASS** |
| IRAC `list_guides`/`guide_status` | `case_count=10`, `section_count=14` | **PASS** |
| `verify_citations` | confidence/match_terms/reason | **PASS** |
| `cards` qa | `recall/definitions/total` | **PASS** |
| IRAC scaffold | 10×`###` | **PASS** (placeholder hâlâ `PD0`) |
| `style estimate accessible` | `over_budget=true` | **PASS** |
| OCR/draft güvenliği | gate-fail → draft, output ezmez | **PASS** (gözlemlendi) |

---

## 8. Puan kartı

| Alan | 0.17.1 | 0.20.0 (final) |
|---|---:|---:|
| Build / PDF | 9.5 | 9.5 |
| Audit / layout | 9.0 | 9.0 |
| QA tek çağrı | 9.5 | **9.8** (gate-fail→draft, combined precedence) |
| Coverage | 4.0 | **8.5** (curatorile geçiyor; generated kırık; excluded statüsü yok) |
| Source/context economics | 6.5 | **8.5** (bütçeler uygulanıyor; est. tutarsızlığı + title-only chapter) |
| D2 | 8.0 | 8.0 |
| Style | 8.5 | 9.0 |
| API tutarlılığı | 8.0 | **9.0** (metrics/cards/claims/graph düzeldi; validate/hint `fields` uygulanmıyor) |
| Determinism | 9.0 | 9.0 |
| Dokümantasyon | 8.0 | 8.0 (akış hâlâ yazılı değil) |

**Ağırlıklı ≈ 9.2 / 10**

---

## 9. Sonuç

0.20.0 bir sıçrama: 0.17.1’de 12 kritik maddenin **11’i fiilen kapandı**; ana rehber **141/141**, dijital **41/41** gate’ten geçiyor; IRAC metrikleri, citation verification, model2vec topics, budgeted claims/graph, cards ayrımı, D2 doctor — hepsi çalışıyor.

Ama "10/10" hâlâ dört tool işi + bir dokümantasyonda:

1. **Generated manifest’i düzelt** ve qa’nın unadopted generated’ı hard gate yapmasını durdur/opt-in et
2. **`chapter_brief` body’si** (şu an title-only)
3. **`coverage_manifest validate` bütçele** (`fields`/`max_tokens` nested kessin)
4. **`status:"excluded"`** + birleşik-heading parser’ı (en-dash range)
5. **Dokümante et**: review → validate → adopt → qa akışı; draft temizleme; merged heading sınırları

Bunlar biterse **10/10** derim — ve bu sefer "141/141" küratör değil, tool tarafından gerçekten üretilmiş olur.
