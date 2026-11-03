# Folisift — Üretim Deneyimi ve İyileştirme Raporu

**Tarih:** 3 Kasım 2026  
**Ortam:** Folisift 0.12.4, build 143, MCP schema 8  
**PDF motoru:** pdfium `chromium/8058`  
**D2:** v0.9.0  
**Semantic model:** `potion-multilingual-128M-int8`  
**Kaynaklar:** `sources/is-hukuku.pdf` (117 sayfa, 18 tablo); `Ders_Notu_1_VİZE` (31 slayt); `DijitalÇağdaÜretimYönetimi` (134 sayfa); `DijitalDönüşümdeÜrünYönetimi` (42 sayfa)  
**Üretilenler:** 26 sayfalık ana kılavuz, 11 sayfalık IRAC çalışma kitabı, 19 sayfalık Dijital Dönüşüm kılavuzu, 3 + 4 D2 şema, 8 + 5 + 6 Anki kartı

---

## İçindekiler

- **0–2:** Kısa hüküm, ölçümler, öncelik matrisi
- **3–6:** P0 — coverage, layout, hata modeli, D2
- **7–12:** P1 — stil, Türkçe başlık, sayfa numarası, tablo, audit, IRAC scaffold
- **12.1–28.1:** MCP yüzeyi, QA, links/graph, claims/cards, küçük sürtünmeler
- **29–36:** Neler iyi çalıştı, kök neden, kod önerileri, yol haritası, API, testler, ekler
- **37:** Agent verimliliği, token maliyeti, “beni hızlandırıyor mu?”
- **38:** Model2vec-rs / `links` değerlendirmesi
- **39:** Liteparse OCR değerlendirmesi
- **40:** Dijital Dönüşüm kılavuzu — 207 sayfa, 3 kaynak, 21 düşük metinli sayfa
- **41:** Context7 incelemesi — LiteParse, D2, Model2Vec kullanılmayan yetenekler

---

## 0. Kısa hüküm

Folisift’in çekirdeği iyi çalışıyor. Özellikle deterministik build, `audit --strict`, D2 doğrulama/üretme, kaynak okuma ve `claims`/`cards` araçları gerçek üretimde işe yaradı.

Asıl sorunlar “çekirdek”te değil, **gerçek bir ders notunu uçtan uca üretirken ortaya çıkan karar noktalarında**:

1. **Coverage gate yanlış güven veriyor.** 7 canonical bölüm “passed” diyor; gerçekte 141 başlık var.
2. **Chapter / columns / table / diagram layout sürekliliği kırılıyor.** Başlık bir sayfada, içerik başka sayfada kalabiliyor.
3. **Hata mesajları yapılandırılmış değil.** Hangi sayfa, hangi blok, hangi satır, ne yapmalı — çoğu zaman eksik.
4. **D2 karar ağaçları için dar.** 12 düğüm, 4 seviye, yatay layout, çalışmayan `width` ve `\n`.
5. **Stil değiştirmek manuel.** Config elle düzenleniyor; `style recommend` structure uyumunu filtrelemiyor.
6. **MCP yüzeyi eksik.** `audit`, `qa_guide`, `restyle_guide`, `coverage_manifest` yok; CLI’a düşülüyor.
7. **Çok kaynaklı sentez ayrı bir zorluk.** Dijital Dönüşüm kılavuzu 207 sayfa, 21 düşük metinli sayfa ve 41 canonical section; 31 slaytlık kaynak gate’e hiç girmiyor.

Aşağıda en büyük sorundan en küçük sürtünmeye kadar her şey var: yaşanan, kanıt, etki, öneri, öncelik.

---

## 0.1 Ölçümler ve sayı tutarsızlıkları

Bu oturumun en çarpıcı bulgusu, aynı kaynak için **dört farklı başlık sayısı** elde edilmesiydi:

| Ölçüm | Değer | Nereden |
|---|---:|---|
| Kaynak PDF sayfası | 117 | `analyze_source` |
| Kaynak tablo | 18 | `analyze_source` |
| `scan-is-hukuku.json` heading | **139** | liteparse taraması |
| `prepare_guide.layout.headings` | **267** | layout block sayımı |
| Manuel `topic-map.md` | **141** | 139 + elle eklenen 2 |
| `coverage.canonical_total` | **7** | canonical section seti |

Bu dört sayı aynı kaynağı anlatıyor. Hangisinin “gerçek” olduğu ürün içinde net değil. Kullanıcı “139 başlık” diyor, araç 267 diyor, manuel matris 141 diyor, gate 7 diyor. Bu, coverage güvenini ciddi biçimde sarsıyor.

Diğer ölçümler:

| Ölçüm | Değer |
|---|---:|
| Ana kılavuz sayfa | 26 |
| Ana kılavuz D2 diagram | 3 |
| Ana kılavuz `stats.tables` | 3 (oysa daha fazla tablo var) |
| Ana kılavuz bookmark | 47 |
| IRAC sayfa | 11 |
| IRAC Anki kartı | 5 |
| Ana kılavuz Anki kartı | 8 |
| `reader + forest-sage` | 11 sayfa |
| `reader + berry` | 11 sayfa |
| `accessible + high-contrast` | 14 sayfa (hedef 10–12 dışı) |
| Sayfa bütçesi iterasyonu | 25 → 22 → 24 → 25 → 26 → 27 → 26 |
| Dijital Dönüşüm kılavuzu sayfa | 19 (hedef 18–20) |
| Dijital Dönüşüm kaynak sayfası | 207 (31 + 134 + 42) |
| Dijital Dönüşüm düşük metinli sayfa | **21** (11 + 8 + 2) |
| Dijital Dönüşüm canonical section | **41** (27 üretim + 14 ürün) |
| Dijital Dönüşüm D2 diagram | 4 |
| Dijital Dönüşüm tablo | 12 |
| Dijital Dönüşüm Anki kartı | 6 |
| Dijital Dönüşüm `claims.md` satırı | 146 |

Bu tablo, “passed” kelimesinin tek başına yeterli olmadığını gösteriyor: sayılar tutarlı değilse pass de yanıltıcı olabilir.

## 1. Bu oturumda ne yapıldı?

Kısa bağlam, çünkü sorunların çoğu bu akışta ortaya çıktı:

1. 117 sayfalık birleştirilmiş İş Hukuku notu analiz edildi.
2. `prepare_guide` ile `is-hukuku` rehberi açıldı.
3. 17 ana bölümü kapsayan 26 sayfalık ana kılavuz yazıldı.
4. 3 D2 şema üretildi: yapı, fesih karar ağacı, sosyal güvenlik kolları.
5. `build_guide check=true` ve `audit --strict` ile sayfa/layout döngüsü yapıldı.
6. Ayrı bir `is-hukuku-irac` rehberi açıldı; 10 olay, 11 sayfa.
7. IRAC için farklı palet denendi: `reader + forest-sage`, `reader + berry`, `accessible + high-contrast`.

Bu akış, Folisift’in hem güçlü hem zayıf taraflarını net gösterdi.

---

### 1.1 Kırılma noktaları — kronolojik

| Aşama | Ne yaptım | Ne kırıldı? |
|---|---|---|
| Kaynak analizi | 117 sayfa, 18 tablo, 139 otomatik heading | `4.4.4` ve `10.2` eksik; `prepare_guide.layout.headings` 267 |
| Ana kılavuz | 17 bölüm, 3 D2, 26 sayfa hedefi | Coverage 7/7; underfilled `[4, 8, 19]`; sayfa 25→22→24→25→26→27→26 |
| Fesih diyagramı | 7 → 12 düğüm, dikey denemeler | 12 düğüm limiti; `direction: down` yatay; `width` etkisiz; `\n` çalışmıyor |
| C bölümü | “Kavramlar ve tanımlar” + 66 terim | Chapter kicker/title sayfa 17/18; columns bloğu sayfa 19; tabloya çevirmek gerekti |
| IRAC | 10 olay, 11 sayfa | Scaffold yanlış; `### Sorun` structure gate’e sayılmadı; 7 empty section; page 10 underfilled |
| Stil | forest-sage, berry, accessible | Config elle; `accessible` 14 sayfa; `recommend` workbook/irac uyumsuz |
| Dijital Dönüşüm | 3 kaynak, 207 sayfa, 21 low-text | 41 canonical; 31 slayt gate’te 0; ürün raporu `page: —`; Şekil 2–3 küçültüldü |
| QA | Build + audit + cards + claims | MCP’de audit/qa yok; her biri ayrı CLI/araç |

Bu tablo, “hangi sorun ne zaman ortaya çıktı?” sorusunun kısa cevabı. Aşağıdaki bölümler her birini ayrıntılandırıyor.

## 2. Öncelik matrisi

| # | Sorun | Etki | Öncelik |
|---:|---|---|---|
| 1 | Coverage gate 7/7, gerçek 141 başlık | Yanlış “tamamlandı” güveni | P0 |
| 2 | Chapter/columns/table layout bölünmesi | PDF’de başlıksız/boş sayfa | P0 |
| 3 | Yapılandırılmamış hata + yanlış satır no | Debug maliyeti | P0 |
| 4 | D2 12 düğüm / 4 derinlik / yatay layout | Karar ağacı sığmıyor | P0 |
| 5 | Stil değiştirme manuel | Kullanıcı hataları | P1 |
| 6 | Türkçe başlık ayrıştırma | Eksik konu, yanlış outline | P0/P1 |
| 7 | 0-based / 1-based sayfa karışıklığı | Yanlış referans | P1 |
| 8 | `tables` sayısı eksik | Yanlış QA raporu | P1 |
| 9 | Audit sayfa rolü tanımıyor | TOC/figure sayfası yanlış flag | P1 |
| 10 | IRAC scaffold yanlış iskelet | Manuel temizlik | P1 |
| 11 | QA araçları dağınık | Uçtan uca release zor | P1 |
| 12 | MCP’de `audit`/`qa_guide` yok | CLI’a düşme | P0/P1 |
| 13 | `build_guide check=true` draft üretmiyor | Kör debug | P1 |
| 14 | `prepare_guide` outline 267/139/141 tutarsız | Yanlış kapsam | P1 |
| 15 | `diagram status` şema tutarsızlığı | API kullanımı zor | P2 |
| 16 | `flow` dokümante değil | Yanlış kullanım | P2 |
| 17 | Links/graph heading etiketi yok | Çıktı okunaksız | P2 |
| 18 | Claims/cards gürültüsü | Kullanım keyfi düşük | P2 |
| 19 | Structure validator yalnızca `##` | `###` kullanan fail eder | P1 |
| 20 | Çok kaynaklı coverage (31 slayt gate’te 0) | Dijital Dönüşüm’de yanlış güven | P0/P1 |

---

## 3. P0 — Coverage gate yanlış güven veriyor

### Yaşanan

`prepare_guide` kaynağı analiz etti; `data/scan-is-hukuku.json` içinde **139 başlık** bulundu. Gerçekte iki başlık daha vardı:

- `4.4.4 Asıl İşveren – Alt İşveren`
- `10.2 Ulusal Bayram ve Genel Tatiller`

Bunları elle ekleyince toplam **141** oldu.

Ama `check_coverage` / `build_guide check=true` yalnızca **7 canonical section** üzerinden çalıştı:

```text
canonical_total: 7
claimed: 7
gate_total: 7
missing: 0
passed: true
```

Yani rehber 141 başlığın tamamını kapsamasa bile gate “passed” diyebiliyordu.

### Kanıt

- `data/scan-is-hukuku.json` → 139 heading
- `guides/is-hukuku/data/topic-map.md` → 141 satır
- `guides/is-hukuku/data/coverage.md` → 7 canonical bölüm
- Ana rehberin G bölümü → “141/141 başlık manuel matriste işaretlendi”
- `coverage.sources[0].sections[*].body_chars` → 7 claim için de **808**; yani birden çok claim tek başlığın gövdesini paylaşıyor

Bu sayılar aynı ürün içinde farklı şeyler söylüyor. Üstelik 7 claim’in aynı `body_chars` değerini alması, “her claim kendi gövdesine sahip” varsayımını da sarsıyor.

### Etki

En büyük etki **güven**. Kullanıcı “17 bölüm, 141 başlık, hiç konu atlanmadı” sanıyor; gate ise sadece 7 bölümü doğruluyor. Akademik/legal içerikte bu kabul edilemez.

### Öneri

1. `coverage_manifest` adında yeni MCP aracı:
   - `build`, `read`, `update`, `validate`.
2. Manifest satırı:
   ```json
   {
     "source": "is-hukuku.pdf",
     "heading": "4.4.4 Asıl İşveren – Alt İşveren",
     "page": 14,
     "guide_heading": "B1/4",
     "status": "covered",
     "evidence": "## 4. ... Asıl işveren–alt işveren ..."
   }
   ```
3. `check_coverage` manifestteki **tüm** satırları denetlesin.
4. `prepare_guide` manifest iskeletini üretsin; eksik/yanlış başlıklar MCP’den `add`, `rename`, `merge` ile düzeltilebilsin.
5. Otomatik başlık taraması “hint” olsun; tek başına gate olmasın.
6. G bölümündeki “141/141” ifadesi manifestten **otomatik** üretilsin; elle yazılmasın.

### Kabul kriteri

- 141 başlıklı kaynakta gate `total: 141, claimed: 141, missing: []` döndürmeli.
- Bir başlık kaldırıldığında gate `missing: ["..."]` ile fail etmeli.
- `coverage.md` ile `topic-map.md` aynı sayıyı göstermeli.

---

## 4. P0 — Chapter / columns / table / diagram layout sürekliliği

### 4.1 Chapter opener iki sayfaya bölünüyor

**Yaşanan:** C bölümü “Kavramlar ve tanımlar” bir ara şöyle göründü:

- Sayfa 17’nin en altında `C–G · ORTAK BÖLÜMLER` ve `C` kicker’ı
- Sayfa 18’de “Kavramlar ve tanımlar” başlığı
- Sayfa 19’da sözlük içeriği

Yani chapter opener üç parçaya ayrıldı.

**Neden:** `flow=true` chapter’ı önceki sayfanın dibine taşıdı; `flow=false` ise başlığı tek başına bıraktı. Arada “başlığı ilk içerik bloğuyla birlikte tut” seçeneği yok.

**Öneri:**

- `chapter` için `keep_with_next=true` (varsayılan).
- `break_before`, `break_after`, `allow_split` alanları.
- Chapter opener’ın altında en az N mm içerik yoksa audit `chapter_split` uyarısı versin.
- `flow` davranışı dokümante edilsin; “flow=true başlığı önceki sayfaya taşır” açıkça yazılsın.

### 4.2 `::: columns` monolotik

**Yaşanan:** 66 terimlik sözlük `::: columns` içindeydi. Başlık sayfasına sığmayınca **tamamı** sonraki sayfaya kaçtı. Sonunda sözlüğü 4 kolonlu tabloya çevirmek zorunda kaldım.

**Etki:** Başlık ile içerik ayrı sayfalarda kalıyor; kullanıcı “başlık neden başka sayfada?” diyor.

**Öneri:**

- `columns` için `flow=true` veya otomatik sayfa bölme.
- Kolon bloğu en azından satır/section sınırında bölünebilsin.
- `columnbreak` sonrası yeni sayfa politikası net olsun.

### 4.3 `::: table` ve `::: diagram` kalan alana akmıyor

**Yaşanan:** Şekil 3, C bölümünün başındaydı. 26 sayfa olan rehber 27’ye çıktı; şekli sayfa 20’ye, C.3’ün başına taşıyınca 26’ya döndü.

**Etki:** Tek bir görsel yüzünden sayfa bütçesi bozuluyor; kullanıcı “şekil yüzünden boş sayfa” görüyor.

**Öneri:**

- `table`/`diagram` için “sığmıyorsa başlıkla birlikte taşı” politikası.
- `keep_with_next`, `allow_page_break`, `min_space_mm` alanları.
- `diagram` için `width`/`height` gerçekten uygulansın; uygulanmıyorsa hata versin.

### 4.4 `::: front` içinde `::: table` kapatma sorunu

**Yaşanan:** Front matter’da tablo varken `::: front` bloğunu kapatmayı unuttum. Hata:

```text
Markdown error at line 2: unclosed directive ::: front
```

Gerçek sorun satır 17’deydi. Satır 2, frontmatter’ın `title` satırı.

**Öneri:**

- Directive stack ile hata: hangi directive açık, hangi satırda açıldı, hangi satırda bekleniyordu.
- `::: front` örneğinde iç içe `::: table` kullanımı net gösterilsin.
- `::: front` otomatik kapansın mı? En azından “front bloğu içinde table kullanacaksan iki kapanış gerekir” dokümante edilsin.

---

## 5. P0 — Hata modeli yapılandırılmış değil

### Yaşanan hatalar

```text
Markdown error at line 2: unclosed directive ::: front
underfilled body pages: [4, 8, 19]
page count is 22, expected 24-26 (target 24-26)
structure "irac" is missing required sections: Sorun, Kural, Uygulama, Sonuç
coverage gate failed: 0/7 headings missing, 7 empty sections
```

Bunların hiçbiri makine tarafından işlenebilir değil. Hangi sayfa, kaç karakter, hangi blok, hangi section, ne yapmalı — yok.

### Etki

- Her hata için ayrı ayrı PDF render + görsel inceleme gerekiyor.
- MCP istemcisi otomatik retry/repair yapamıyor.
- Kullanıcı “ne yapacağım?” sorusuna cevap bulamıyor.

### Öneri

Yapılandırılmış hata:

```json
{
  "ok": false,
  "code": "E_UNDERFILLED",
  "message": "3 body pages are below the density threshold",
  "details": {
    "pages": [
      {
        "page": 4,
        "chars": 420,
        "median": 1800,
        "ratio": 0.23,
        "role": "toc",
        "blocks": ["toc"]
      }
    ]
  },
  "suggestions": [
    "TOC sayfasını front_matter olarak işaretle",
    "veya TOC'u önceki sayfaya taşı"
  ]
}
```

Hata kodları:

- `E_PAGE_BUDGET`
- `E_UNDERFILLED`
- `E_OVERFLOW`
- `E_STRUCTURE`
- `E_COVERAGE`
- `E_MARKDOWN`
- `E_VISUAL`
- `E_DIAGRAM`

### Kabul kriteri

- Her hata JSON döndürsün.
- `line`, `page`, `block_id`, `directive_stack` alanları olsun.
- `suggestions` en az bir somut eylem içersin.

---

## 6. P0 — D2 karar ağaçları için dar

### 6.1 12 düğüm sınırı

**Yaşanan:** Fesih karar ağacı için 15–20 düğüm gerekiyordu. `MAX_NODES = 12`, `MAX_EDGES = 20` ve tree depth 4 olduğu için 12’ye sıkıştırdım; işçi yolu ve bazı sonuçlar tek düğümde birleşti.

**Etki:** Hukuki karar ağacı ya eksik ya aşırı sıkışık.

**Öneri:**

- `MAX_NODES` 20–24.
- `kind: "decision"` veya `tree` için depth 6–8.
- Gerekirse 2 sayfalık diagram desteği.

### 6.2 `direction: "down"` her zaman dikey sonuç vermiyor

**Yaşanan:** İlk fesih diyagramı `direction: "down"` olmasına rağmen yatay/aşırı geniş çıktı. TALA aspect ratio’yu kendi seçiyor gibiydi. Ancak ana omurga tek zincir olunca dikey ve sayfayı dolduran bir sonuç aldım.

**Öneri:**

- `aspect_ratio` veya `rankdir` kontrolü.
- `layout: "elk"` / `dagre` seçeneği.
- `direction` gerçekten uygulanmıyorsa dokümante edilsin.

### 6.3 `width` etkisiz

**Yaşanan:** `::: diagram id="sosyal-guvenlik-kollari" width="60%"` ve `width="70%"` denedim; şeklin boyutu bende görünür şekilde değişmedi. `width="90%"` de aynı kaldı.

**Etki:** Sayfa doldurma/şekil küçültme kontrolü yok.

**Öneri:**

- `width`/`height` gerçekten uygulansın.
- Uygulanamıyorsa validate hatası versin.
- `max_width`, `max_height`, `fit` seçenekleri.

### 6.4 `label` içinde `\n` çalışmıyor

**Yaşanan:** Düğüm etiketlerini iki satıra bölmek için `"Haklı neden?\nm.24"` denedim; render tek satır kaldı.

**Öneri:**

- Çok satırlı label desteği.
- Alternatif olarak `label_lines: ["Haklı neden?", "m.24"]`.

### 6.5 `diagram status` şema tutarsızlığı

**Yaşanan:**

```json
{
  "id": "fesih-karar-agaci",
  "status": null,
  "state": "ok"
}
```

`jq` ile `status` okuduğumda `null` geldi; gerçek alan `state`.

**Öneri:**

- Her yerde tek alan: `state` veya `status`.
- `ok`, `state`, `status` karışıklığı kalksın.
- Şema versiyonu artırılsın; eski alanlar deprecated işaretlensin.

---

## 7. P1 — Stil sistemi fazla manuel

### 7.1 `style recommend` structure uyumunu filtrelemiyor

**Yaşanan:**

```text
structure: irac
intent: practice
→ recommended style: workbook
```

Ama `workbook` stili `irac` yapısını desteklemiyor. Öneri, uygulanamaz bir kombinasyon verdi.

**Öneri:**

- `recommend` önce `structure` + `domain` + `target_pages` uyumunu filtrelesin.
- Uyumsuzsa “en yakın uyumlu” ve “neden” döndürsün.
- `supported_structures` kontrolü zorunlu olsun.

### 7.2 `accessible` sayfa bütçesini patlatıyor

**Yaşanan:** `accessible + high-contrast` görünüm güzel; ama 10–12 hedefindeki IRAC **14 sayfa** oldu. Bunu ancak full build’de gördüm.

**Öneri:**

- `style estimate` veya `style preview` sayfa tahmini versin.
- `target_pages` ile uyumsuzsa build’den önce uyarı versin.
- `accessible` için “large print, +2–4 sayfa” gibi dokümantasyon.

### 7.3 Stil değiştirmek için config elle düzenleniyor

**Yaşanan:** IRAC’ta palet değiştirmek için `guides/is-hukuku-irac/config.json` içindeki `style.palette` alanını elle değiştirdim; sonra build ettim.

**Öneri:**

- `restyle_guide({slug, style, palette, fonts, output?})` MCP aracı.
- İçerik hash’i değişmesin; sadece stil config’i değişsin.
- `content_hash_before == content_hash_after` kontrolü.

### 7.4 `style preview` jenerik

**Yaşanan:** `style preview` “Style preview” başlıklı 4 sayfalık genel örnek üretiyor; gerçek IRAC tablo/kutu/başlık görünümünü göstermiyor.

**Öneri:**

- Preview, hedef rehberden bir bölümü kullansın.
- En az bir tablo, bir kutu, bir recall/answer ve bir chapter opener göstersin.
- Sayfa sayısı tahmini de versin.

---

## 8. P0/P1 — Türkçe başlık ayrıştırma

### Yaşanan

`prepare_guide` outline’ında bozuk başlıklar:

- `1.İş HukukununTanımı, Kavramı ve Konuları`
- `5.2.İş SözleşmesininTürleri`
- `13.4.Tutulacak Defter ve Kayıtlar`

Ayrıca iki başlık hiç gelmedi:

- `4.4.4 Asıl İşveren – Alt İşveren`
- `10.2 Ulusal Bayram ve Genel Tatiller`

### Etki

- Eksik konu riski.
- Yanlış `covers` eşleşmesi (`is-hukuku:1.İş` gibi).
- Manuel topic-map zorunluluğu.

### Öneri

- Font büyüklüğü/kalınlık + numaralı başlık regex’i + TOC eşleştirmesi birlikte kullanılsın.
- `headings.review.md` üretilsin; kullanıcı eksik/yanlışları düzeltsin.
- `heading override` API’si: `add`, `rename`, `merge`, `page`.
- Türkçe için `İ`, `ı`, `ş`, `ğ` normalizasyonu test edilsin.
- TOC varsa gövde başlıklarıyla çapraz kontrol yapılsın.

---

## 9. P1 — Sayfa numarası 0-based / 1-based karışıklığı

### Yaşanan

`coverage` çıktısında section 1 için `page: 0` görünüyor; PDF’de sayfa 1. Bazı yerlerde `page: 1`.

### Etki

- Kullanıcı yanlış sayfaya bakar.
- `covers` ve audit sayfa referansları karışır.

### Öneri

- Her yerde 1-based.
- Veya `page_index` / `page_number` ayrımı net.
- `read_source` ve `coverage` aynı sözleşmeyi kullansın.

---

## 10. P1 — `tables` sayısı eksik

### Yaşanan

Ana rehberde `stats.tables: 3` dedi; oysa:

- Şekil 2 dal tablosu
- D.4 hesap özeti
- E olgular tablosu
- G kapsam tablosu
- C.4 kavram köprüleri
- IRAC’ta olay haritası, kontrol listesi, kapsam tablosu

`tables` yalnızca `::: table` ile sarılanları sayıyor gibi. Markdown tabloları saymıyor.

### Etki

- QA raporu yanlış.
- “Kaç tablo var?” sorusuna güvenilir cevap yok.

### Öneri

- `tables` tüm Markdown tablolarını saysın.
- `captioned_tables` ayrı alan olsun.
- `::: table` sayısı `wrapped_tables` olarak ayrılsın.

---

## 11. P1 — Audit sayfa rolü tanımıyor

### İyi taraf

`audit --strict` gerçekten değerli:

- underfilled
- overflow
- orphan heading
- bookmark mismatch
- missing glyph
- tagged PDF
- figure captions

### Sorun

Underfilled eşiği sadece metin karakterine bakıyor.

- TOC sayfası (s. 4) underfilled diye flaglendi; oysa kasıtlı.
- Sadece büyük şekil içeren sayfa bazen flaglanmıyor.
- “Şekil 2 tek başına sayfada” durumunu audit yakalamadı; ben görsel bakarak buldum.
- Chapter opener’ın kicker/title split’ini audit yakalamadı.

### Öneri

- `page_roles`: `front_matter`, `toc`, `chapter_opener`, `figure_page`, `table_page`, `body`.
- `intentional_pages` / `layout_exceptions` manifesti.
- `chapter_split` detector.
- `figure_only` detector: sayfada sadece bir diagram ve caption varsa ayrı uyarı.
- Underfilled eşiği role göre değişsin.

---

## 12. P1 — IRAC scaffold yanlış iskelet üretiyor

### Yaşanan

`prepare_guide` IRAC için:

- 60+ kaynak başlığını `##` olarak scaffold etti.
- `sources: []` yazdı.
- IRAC `Sorun/Kural/Uygulama/Sonuç` başlıklarını en sona ekledi.

Bu, IRAC çalışma kitabı için yanlış iskelet. Ben dosyayı tamamen değiştirdim.

### Öneri

- Structure’a özel scaffold:
  - `irac` → 10 case placeholder, her case’de Sorun/Kural/Uygulama/Sonuç.
  - Kaynak başlıkları ayrı `source-headings.md` dosyasına.
- `sources` frontmatter’a otomatik yazılsın.
- `prepare_guide` “mevcut guide” durumunda içerik üzerine yazmasın; `--force` istesin.

### Structure validator yalnızca `##` kabul ediyor

**Yaşanan:** IRAC case’lerinde `### Sorun`, `### Kural`, `### Uygulama`, `### Sonuç` kullandım. Build:

```text
structure "irac" is missing required sections: Sorun, Kural, Uygulama, Sonuç
```

`###` başlıkları structure gate’e sayılmıyor. `##`’a çevirince geçti.

**Sorun:** Dokümantasyon “başlık” diyor; “yalnızca h2” demiyor. `###` kullanan herkes aynı hataya düşer.

**Öneri:**
- Structure validator `##` ve `###` kabul etsin; veya `heading_level` alanı olsun.
- Dokümanda “structure başlıkları h2 olmalı” açıkça yazılsın.
- Hata mesajı “Sorun başlığı `##` seviyesinde olmalı” desin.

---

## 12.1 P0/P1 — MCP yüzeyi: eksik araçlar ve isim uyumsuzlukları

Bu oturumda birçok işi MCP üzerinden yaptım; ama bazı kritik işler için CLI’a düşmek zorunda kaldım. Bu, “MCP ile uçtan uca üretim” iddiasını zayıflatıyor.

### Eksik MCP araçları

| İş | MCP’de var mı? | Ne yaptım? |
|---|---|---|
| `audit --strict` | **Yok** | `folisift audit` CLI |
| `doctor` | **Yok** | `folisift doctor` CLI |
| `qa_guide` / release gate | **Yok** | Build + audit + cards + claims ayrı ayrı |
| `restyle_guide` | **Yok** | `config.json` elle düzenledim |
| `coverage_manifest` | **Yok** | `topic-map.md` elle yazdım |
| `graph query/path/explain` | **Yok** | `document-graph.json` + `jq` |
| `table` extraction | **Yok** | CLI `folisift table` |
| `diagram build` | MCP’de `save`/`render_plan` var; CLI’da `build` | CLI `folisift diagram build` |
| `style apply` | **Yok** | Config elle |

### İsim / şema uyumsuzlukları

- CLI `diagram build` ↔ MCP `diagram save` / `render_plan`. Hangi action’ın gerçekten SVG ürettiği net değil.
- CLI `styles` ↔ MCP `style`. Tekil/çoğul farkı.
- `diagram validate` → `items[].ok`
- `diagram build` → `items[].cache_hit`
- `diagram status` → `items[].state`
- `build_guide` → `passed`, `layout_passed`, `genre_passed`
- `audit` → `passed`, `issues`
- `coverage` → `passed`, `missing`, `empty_sections`
- `prepare_guide` `style` parametresi açıklaması: “Aesthetic/layout id: paper or clean” — oysa `reader`, `accessible`, `journal`, `workbook` de var.

### `prepare_guide` outline sorunları

- `outline` yalnızca ilk 10 satırı döndürüyor gibi; `total`/`next` yok.
- `covers` alanında `is-hukuku:1.İş` gibi malformed anahtarlar var.
- `layout.headings: 267` ile `scan` 139 arasındaki fark açıklanmıyor.
- `existing: false` var; mevcut guide üzerine yazma davranışı ve `force` yok.
- `scaffold_guide` ile `prepare_guide` neredeyse aynı; hangisi ne zaman kullanılmalı belirsiz.

### `build_guide` check davranışı

- `check=true` başarısız olduğunda **yeni PDF üretmiyor**; debugging için `check=false` ile ikinci build gerekiyor.
- `check=true` yalnızca coverage/layout gate çalıştırıyor; `audit --strict`’in tamamını çalıştırmıyor.
- Hata durumunda “draft PDF” yolu döndürülmüyor.

**Öneri:** `build_guide` başarısız olsa bile `output/_draft.pdf` üretsin ve `draft_path` döndürsün. `qa_guide` tek çağrıda build + coverage + audit + visual + cards yapsın.

---

## 13. P1 — QA araçları dağınık

### Yaşanan

Bir rehberi “hazır” saymak için:

1. `build_guide check=true`
2. `audit --strict`
3. `check_coverage`
4. `cards`
5. `claims`
6. `visual-audit.json`
7. `diagram status`

Ayrı ayrı çalıştırmak gerekiyor. `build_guide check=true` audit’in tamamını çalıştırmıyor; `audit` ayrı.

### Öneri

`qa_guide({slug, expected_pages, strict})` tek araç:

```json
{
  "build": {"pages": 26, "passed": true},
  "coverage": {"total": 141, "claimed": 141, "missing": []},
  "audit": {"underfilled": [], "overflow": [], "bookmarks": "ok"},
  "visual": {"low_text_pages": 0, "decisions": 0},
  "cards": {"count": 8},
  "claims": {"referenced": 117, "unreferenced": 80},
  "diagrams": {"ok": 3},
  "passed": true
}
```

Bu, “release gate” olur.

---

## 14. P2 — `flow` dokümante değil

### Yaşanan

`chapter` attribute listesinde `flow` yok; kodda var. Ben `flow=true`’yu deneme yanılma ile buldum.

### Öneri

- `flow`, `keep_with_next`, `break_before`, `break_after` dokümante edilsin.
- `flow=true`’nun “chapter’ı önceki sayfaya taşır” etkisi örnekle gösterilsin.
- `subs=false`’ın TOC’a etkisi net yazılsın.

---

## 15. P2 — Links / graph çıktısı okunaksız

### Yaşanan

`links build`:

```json
{
  "blocks": 1,
  "sections": 354,
  "links": 186
}
```

`blocks: 1` neyi sayıyor belli değil.

`links read` çıktısı:

```json
{
  "from": "is-hukuku:p44:s131",
  "to": "is-hukuku:p47:s137",
  "confidence": "SIMILARITY",
  "relation": "semantically_similar_to"
}
```

Heading adı yok; insan için `p44:s131` okunaksız.

### Öneri

- `from_heading`, `to_heading`, `guide_section` alanları.
- `confidence` için insan okunur etiket: `high`, `medium`, `low`.
- `blocks` yerine `source_blocks`, `guide_sections`, `links`.
- `graph` MCP’de `query`, `path`, `explain` action’ları.
- `section` node türü ve `contains` edge’i.

---

## 16. P2 — Claims / cards gürültüsü

### Yaşanan

- `cards` recall bloğu yoksa sessizce `0` dönüyor.
- `claims` MCP özeti yalnızca sayı veriyor; `claims.md` satır numarası ve kaynak sayfası içeriyor ama **section/severity** yok. “Unreferenced” 80+ çıkıyor; kısa geçiş cümleleri ve soru listeleri de claim sayılabiliyor.
- `review_intervals_days: []` varsayılanı boş.

### Öneri

- `cards`: recall yoksa “0 recall block; add ::: recall” uyarısı.
- `definition`/`must-know` bloklarından da kart üretme seçeneği.
- `claims`: severity + section + “missing citation” ayrımı; kısa geçiş cümlelerini filtrele.
- Varsayılan aralıklar: 1, 3, 7, 14, 30 gün.

---

## 17. P2 — Tablo numaralandırma

### Yaşanan

Şekil 2’den sonra gelen tablo “Tablo 1” oldu. Ana rehberde başka tablolar da var. Numaralandırma, kaynak tablo numaralarıyla karışıyor.

### Öneri

- `number=false` / `caption_number=false`.
- `id` ile sabit referans.
- “Tablo 1” yerine “Şekil 2’nin dalları” gibi caption-only seçenek.

---

## 18. P2 — `visual-audit` ve düşük metinli sayfa tanımı

### Yaşanan

Kaynakta 0 düşük metinli sayfa olduğu için `visual-audit.json` boş geçerliydi. Ama rehberde sadece şekil içeren sayfa sorunu vardı. `visual-audit` rehber PDF’ini değil, kaynağı denetliyor.

### Öneri

- `visual-audit` hem kaynak hem output PDF için çalışsın.
- `figure_only` sayfaları ayrı role alsın.
- `diagram` sayfaları için `visual` flag’i.

---

## 19. P2 — `style` uyumluluk matrisi

### Yaşanan

IRAC + law için uyumlu stiller:

- `reader`
- `accessible`

`paper`, `journal`, `clean`, `workbook`, `revision` gibi stiller `irac` veya `law` kombinasyonunu desteklemiyor. Bu, `style list` çıktısında görünüyor ama `recommend` bunu filtrelemiyor.

### Öneri

- `style list` çıktısında “bu structure/domain için uyumlu mu?” kolonu.
- `recommend` uyumsuzları hiç önermesin.
- `style info` “supported_structures” ve “supported_domains”ı açıkça göstersin.

---

## 20. P2 — `subs=false` semantiği

### Yaşanan

IRAC’ta 10 case ve 40 IRAC alt başlığı vardı. `subs=false` ile TOC’u kısalttım; ama hangi başlıkların gizlendiği net değil.

### Öneri

- `subs=false` “bu chapter’ın altındaki tüm h2’leri TOC’tan gizle” mi?
- `toc_depth` veya `toc_level` alanı.
- `toc_include` / `toc_exclude` listesi.

---

## 21. P2 — `::: table` width doğrulaması

### Yaşanan

`widths="24,38,38"` gibi toplam 100 olan değerler sorunsuz. Ama hatalı toplamda hata mesajı görmedim; denemedim. Yine de validate mesajı net olmalı.

### Öneri

- `widths` toplamı 100 değilse `E_TABLE_WIDTHS` ve hangi değerler olduğu.
- `widths` sayısı kolon sayısıyla uyuşmuyorsa ayrı hata.

---

## 22. P2 — `read_source` / `search_source` deneyimi

### İyi

- `read_source` bounded chunk + `next_offset` iyi.
- `search_source` sayfa numaralı snippet veriyor.
- Cache hızlı.

### Sorun

- `search_source` çok geniş sonuç döndürebiliyor: bir sorguda 109 sayfa eşleşti, 20 sonuç döndü.
- `next_pages`/`next_offset` bazen aynı sayfaya işaret ediyor.
- `max_chars` kesmesi sayfa ortasında olabiliyor; “bu sayfa yarım” bilgisi yok.

### Öneri

- `truncated_pages` alanı.
- `search_source` için `rank`, `section`, `heading` alanları.
- `read_source` “page complete” bayrağı.

---

## 23. P2 — `build_guide` page budget deneyimi

### Yaşanan

- 25 → 22 → 24 → 25 → 26 → 27 → 26 sayfa arasında gidip geldim.
- Her seferinde “page count is X, expected 24-26” dedi; hangi bölümün ne kadar yer kapladığını söylemedi.

### Öneri

- `page_budget` raporu: bölüm başına sayfa, tablo/şekil payı.
- “Şu bölüm 0.4 sayfa kısaltılırsa hedefe girer” gibi öneri.
- `target_pages` aralığında `min`/`max` ve “hangi sayfalar fazla” bilgisi.

---

## 24. P2 — `diagram status` ve manifest

### Yaşanan

`diagram status` çıktısında `manifest` ve `spec` yolları var; `state: ok`. Ama `status` alanı yok. `diagram build` `cache_hit` döndürüyor; `validate` `ok`. Üç ayrı şema.

### Öneri

- Ortak `DiagramResult`:
  ```json
  {
    "id": "...",
    "ok": true,
    "state": "ok",
    "cached": false,
    "svg": "...",
    "errors": []
  }
  ```

---

## 25. P2 — `prepare_guide` / `scaffold_guide` tekrarı

### Yaşanan

İki araç neredeyse aynı işi yapıyor; `prepare_guide` daha çok şey döndürüyor. Hangisini ne zaman kullanacağım belirsiz.

### Öneri

- `prepare_guide` = analiz + scaffold + plan.
- `scaffold_guide` = sadece dosya iskeleti.
- Dokümanda fark net yazılsın.
- `prepare_guide` mevcut guide’ı ezmesin; `force` istesin.

---

## 26. P2 — `workspace_info` / `list_guides`

### İyi

- Hızlı, net.
- `valid` alanı var.
- `content_words` ve `section_count` faydalı.

### Küçük sorun

- `guides: 3` ile `source_count: 5` farklı şeyler; açıklama yok.
- `section_count` IRAC’ta 54 (10 case × 4 IRAC + ekler); “10 olay” bilgisi yok.

### Öneri

- `guide_kind` / `case_count` alanı.
- `source_count` ile `guide_count` ayrımı dokümante edilsin.

---

## 27. P2 — `doctor`

### İyi

- pdfium, cache, output, guides, fonts, structures, model, D2 hepsi tek raporda.
- `ok: true` net.

### Öneri

- `doctor` çıktısına “önerilen sonraki komut” zaten var; güzel.
- `style` uyumluluk kontrolü de eklensin: “bu guide’ın stili structure’ı destekliyor mu?”

---

## 28. P2 — Dokümantasyon boşlukları

- `::: front` içinde `::: table` örneği belirsiz.
- `chapter` `flow` alanı yok.
- `subs=false` etkisi net değil.
- D2 `width` davranışı yok.
- `diagram status` şeması yok.
- `style recommend` uyumluluk matrisi yok.
- `cards` 0 davranışı yok.
- `tables` sayım kuralı yok.

### Öneri

- Her directive için “minimal örnek + kapanış” gösteren bir `docs/authoring-cookbook.md`.
- “Sık yapılan hatalar” bölümü.
- “Hangi araç ne zaman?” karar ağacı.

---

## 28.1 Küçük sürtünmeler (ama gerçek)

Bunlar tek başına projeyi durdurmaz; ama günlük kullanımda “neden böyle?” dedirten küçük pürüzler:

- `audit.line_chars_p50: 4` — tablo hücreleri yüzünden anlamsız; düz metin satır uzunluğu ayrı ölçülmeli.
- `audit.font_sizes` içinde `size: 1.0` ve 2.241 karakter — gizli/metadata metni mi, yoksa gerçek mi? Rapor açıklamıyor.
- `workspace_info` “3 guide, 5 source” — iki sayının neyi saydığı dokümante değil.
- `list_guides` IRAC için `section_count: 54` — 10 olay olduğu görünmüyor; `case_count` alanı yok.
- `build_guide.style_rationale` varsayılanı “Scaffold default for reader; the host must refine...” — her build’de taşınıyor; config’te elle düzeltmek gerekiyor.
- `config.json` içinde hem `theme` hem `style.aesthetic` var; ikisi de aynı olmalı. Hangisi otoriter, dokümante değil.
- `prepare_guide` `style` parametresi açıklaması hâlâ “paper or clean”; `reader`/`accessible`/`journal`/`workbook` var.
- `diagram` MCP action adları (`save`, `render_plan`) ile CLI (`build`) uyuşmuyor.
- `cards` `review_intervals_days: []` — varsayılan aralık yok.
- `claims` “unreferenced” kısa geçiş cümlelerini ve soru listelerini de sayıyor; gürültü.
- `links` `confidence: "SIMILARITY"` — insan için `high/medium/low` daha okunur.
- `graph` düğüm türlerinde `heading`/`section` yok; `block`, `citation`, `page`, `source`, `table_cell` var.
- `read_source` `max_chars` kesmesi sayfa ortasında olabiliyor; “page complete” bayrağı yok.
- `search_source` geniş sorguda 109 sayfa eşleştiriyor; `rank`/`heading` alanı yok.
- `::: table widths` toplamı 100 değilse hata mesajı net değil; test edilmedi ama validate edilmeli.
- `toc` sayfası underfilled flag’leniyor; front matter istisnası yalnızca ilk 3 sayfa.
- `::: front` içinde `::: table` kullanımı dokümante değil; iki kapanış gerekiyor.
- `chapter flow` dokümante değil; `flow=false`’ın boş opener ürettiği yazmıyor.
- `subs=false`’ın TOC’tan hangi seviyeyi gizlediği belirsiz.
- `table` caption otomatik numarası “Tablo 1”i Şekil 2’den sonra veriyor; `number=false` yok.
- `build_guide` `stats.tables` yalnızca `::: table` sayıyor; Markdown tabloları saymıyor.
- `build_guide check=true` başarısızken draft PDF üretmiyor; debugging için ikinci build gerekiyor.

---

## 29. Neler iyi çalıştı

Bunlar korunmalı:

1. **`audit --strict`** — underfilled, overflow, orphan heading, bookmark, glyph. Bu olmasa 26 sayfayı temizleyemezdim.
2. **D2 validate/build/status** — deterministik, cache’li, hata mesajları çoğunlukla net.
3. **`read_source` / `search_source`** — kaynağa bağlı kalmayı kolaylaştırıyor.
4. **`claims` / `cards`** — CSV temiz; recall/answer eşleşmesi doğru.
5. **`doctor`** — hızlı teşhis.
6. **`list-guides` / `workspace_info`** — pratik.
7. **`style preview`** — paletleri görmek için hızlı.
8. **Yerel çalışma** — model2vec ve D2 dış servis gerektirmiyor.
9. **`build_guide check=true`** — hızlı geri bildirim.
10. **`visual-audit.json`** — doğru fikir; kaynak için 0 sayfa olduğunda boş geçerli olması iyi.
11. **Çok kaynaklı sentez** — Dijital Dönüşüm 3 kaynak, 207 sayfa, 21 düşük metinli sayfa, 4 D2, 12 tablo ile 19 sayfada tamamlandı; audit geçti. Zor ama başarıldı.

---

## 29.1 Kök neden analizi

Aşağıdaki tablo, gördüğüm semptomların muhtemel teknik kök nedenlerini ve bakılacak dosyaları özetler. Kesin kod teşhisi değil; ürün ekibine hızlı yön vermek için.

| Semptom | Muhtemel kök neden | Bakılacak yer |
|---|---|---|
| Coverage 7/7, gerçek 141 | `source_candidates` yalnızca ilk seviye bölümleri alıyor; `scan_claims` bir `covers` yorumunu bir sonraki `##` başlığına bağlıyor; tam heading manifesti yok | `crates/folisift-parse/src/coverage.rs` |
| 7 claim aynı `body_chars: 808` | Birden çok claim tek başlığa bağlanınca aynı gövdeyi paylaşıyor; her claim için ayrı heading/body eşleşmesi yok | `scan_claims`, `finalize` |
| `empty_sections` 400 karakter | `MIN_SECTION_CHARS = 400`; “empty” semantik değil, karakter eşiği | `coverage.rs` |
| Chapter kicker/title split | Typst `chapter` bloğu atomik; `flow` sadece pagebreak aç/kapa; `keep_with_next` yok | `crates/folisift-render/src/markdown.rs`, `templates/paper.typ` |
| `columns` bloğu bölünmüyor | Kolon container’ı tek grid/block; sayfa bölme politikası yok | `markdown.rs`, template |
| TOC underfilled flag | `detect_underfilled` ilk 3 sayfayı atlıyor, sonra karakter medyanına bakıyor; TOC rolü yok | `crates/folisift-parse/src/audit.rs` |
| Figure-only sayfa flag’lenmiyor | Yoğunluk yalnızca metin karakteri; diagram/vektör sayfası rolü yok | `audit.rs` |
| D2 12 node | `MAX_NODES = 12`, `MAX_EDGES = 20`, tree depth 4 | `crates/folisift-render/src/diagram.rs` |
| `direction: down` yatay | TALA aspect ratio’yu kendi seçiyor; `direction` tek başına yetmiyor | `diagram.rs`, D2 çağrısı |
| `width` etkisiz | Directive width’i SVG/template’e taşınmıyor veya clamp ediliyor | `markdown.rs`, template |
| `\n` label çalışmıyor | D2 quote/escape sırasında newline kaybı | `d2_quote`, `graph_source` |
| `status` null / `state` ok | Farklı sonuç struct’ları | `diagram.rs` |
| `style recommend` workbook/irac | Uyumluluk filtresi yok | style engine / `style recommend` |
| `accessible` 14 sayfa | Stil sayfa tahmini yok; font/margin değişimi build’de görülüyor | style tokens / build |
| IRAC scaffold yanlış | `prepare_guide` structure’a özel scaffold üretmiyor | `prepare_guide` |
| Page 0 | 0-based/1-based karışık | `coverage.rs`, `parse` |
| `tables: 3` | Yalnızca `::: table` sayılıyor; Markdown tabloları sayılmıyor | build stats |
| Hata string’leri | `Result<_, String>` / düz mesaj | CLI + MCP |
| MCP audit yok | Tool register edilmemiş | `crates/folisift-mcp/src/lib.rs` |

## 29.2 Kod seviyesinde öneriler

Bunlar “şu dosyaya şu tip eklensin” düzeyinde somut öneriler:

1. **`HeadingManifest`**
   ```rust
   struct HeadingEntry {
       source: String,
       heading: String,
       page: usize,          // 1-based
       guide_heading: Option<String>,
       status: CoverageStatus,
       evidence: Option<String>,
   }
   ```
   `coverage_manifest` bunu okur/yazar; gate bunu kullanır.

2. **`Diagnostic` / `ErrorCode`**
   ```rust
   enum ErrorCode { PageBudget, Underfilled, Overflow, Structure, Coverage, Markdown, Visual, Diagram }
   struct Diagnostic {
       code: ErrorCode,
       message: String,
       page: Option<usize>,
       line: Option<usize>,
       block_id: Option<String>,
       suggestions: Vec<String>,
   }
   ```

3. **`PageRole`**
   ```rust
   enum PageRole { FrontMatter, Toc, ChapterOpener, FigurePage, TablePage, Body }
   ```
   `audit` underfilled/overflow kararını role göre versin.

4. **`ChapterOptions`**
   ```rust
   struct ChapterOptions {
       keep_with_next: bool,
       break_before: bool,
       break_after: bool,
       allow_split: bool,
   }
   ```

5. **`ColumnsOptions`**
   ```rust
   struct ColumnsOptions { flow: bool, min_rows_before_break: usize }
   ```

6. **`DiagramOptions`**
   ```rust
   struct DiagramOptions {
       max_nodes: usize,      // 24
       layout: DiagramLayout, // Tala | Elk | Dagre
       direction: Direction,
       aspect_ratio: Option<f32>,
       allow_multiline_labels: bool,
   }
   ```

7. **`QaReport`**
   ```rust
   struct QaReport {
       build: BuildReport,
       coverage: CoverageReport,
       audit: AuditReport,
       visual: VisualReport,
       cards: CardReport,
       diagrams: DiagramReport,
       passed: bool,
   }
   ```

8. **`StyleApply`**
   ```rust
   struct StyleApply { slug: String, style: String, palette: String, fonts: String, output: Option<String> }
   ```
   İçerik hash’i değişmiyorsa `content_unchanged: true` döndürsün.

9. **`GraphQuery`**
   ```rust
   enum GraphAction { Query(String), Path { from: String, to: String }, Explain(String) }
   ```

---

## 30. Önerilen yol haritası

### P0 — Bir sprint’te

1. **Coverage manifest + gate**
   - `coverage_manifest` tool.
   - 141/141 zorunlu.
   - `prepare_guide` manifest iskeleti.
2. **Layout sürekliliği**
   - `chapter.keep_with_next`.
   - `columns.flow`.
   - `table`/`diagram` `keep_with_next`.
   - `chapter_split` audit.
3. **Yapılandırılmış hata + draft**
   - `code`, `details`, `suggestions`, doğru `line`/`page`.
   - `check=true` fail olsa bile `draft_path` üret.
4. **D2 genişletme**
   - 20–24 node.
   - `decision` kind.
   - `direction`/`aspect_ratio`.
   - `width`/`\n` desteği.

### P1 — Sonraki sprint

5. `restyle_guide` + `style estimate` + uyumlu `recommend`.
6. Türkçe başlık tespiti + TOC çapraz kontrolü.
7. `qa_guide` tek release gate + `audit_guide` MCP aracı.
8. `tables` sayımı + `page` 1-based.
9. IRAC scaffold düzeltmesi.
10. `diagram status` şema birleştirme.
11. `doctor` ve `table_extract` MCP araçları.

### P2 — Keyif işleri

12. Graph `query/path/explain`.
13. Claims/cards filtreleri.
14. Dokümantasyon cookbook.
15. Tablo numaralandırma seçenekleri.
16. `subs`/`toc_depth`.

---

## 31. Önerilen MCP API yüzeyi

```text
coverage_manifest({
  slug, action: "build"|"read"|"update"|"validate",
  entries?: [{source, heading, page, guide_heading, status, evidence}]
})

read_section({
  source, heading, mode: "full"|"summary"|"claims"|"tables"
})

section_brief({
  source, heading, max_tokens?
})

source_topics({
  source, k?, max_tokens?
})

guide_plan({
  slug, target_pages, source_map?
})

visual_brief({
  source, page
})

citation_snippet({
  source, page, query?
})

verify_citations({
  slug, strict?
})

cost_report({
  slug
})

restyle_guide({
  slug, style, palette, fonts, output?
})

style_estimate({
  slug, style, palette, fonts
})

qa_guide({
  slug, expected_pages, strict?
})

audit_guide({
  slug, pdf?, expected_pages, strict?
})

doctor({
  root?
})

table_extract({
  source, pages?, out?
})

graph({
  guide, action: "query"|"path"|"explain"|"build"|"status"|"read",
  query?, from?, to?, limit?
})

diagram({
  guide, action: "validate"|"save"|"status"|"render_plan"|"list",
  spec?
})
```

---

## 32. Kabul testleri

| Test | Beklenen |
|---|---|
| 141 başlıklı kaynak | `coverage.total=141`, `claimed=141`, `missing=[]` |
| Bir başlık silinmiş | `missing=["..."]`, `passed=false` |
| `chapter flow=true` + tablo | Kicker/title/ilk blok aynı sayfada |
| `columns` uzun sözlük | Başlık + ilk kolon aynı sayfada |
| `diagram` 20 node | Geçerli, dikey, tek sayfa |
| `diagram width=60%` | Görünür şekilde küçülür |
| `build` underfilled | JSON `E_UNDERFILLED`, page/ratio/blocks |
| `tables` | Tüm Markdown tabloları sayılır |
| `page` | Her yerde 1-based |
| `restyle_guide` | `content_hash` değişmez, sadece stil değişir |
| `accessible` | `target_pages` uyarısı build’den önce |
| `cards` recall yok | `0` + uyarı |
| `build check=true` fail | `draft_path` yine de döner |
| `qa_guide` | tek çağrıda build + coverage + audit + cards |
| `### Sorun` | Kabul edilir veya “h2 olmalı” der |
| `audit` MCP | `audit_guide` aracı çalışır |
| TOC sayfası | `page_role=toc`, underfilled değil |
| `prepare_guide` outline | `total`, `next`, tam liste |
| `style recommend` irac+law | `workbook` önermez |
| 3 kaynaklı guide | her kaynak ayrı manifest; 31 slayt 0’da kalmaz |
| `graph query` | `query("fesih")` section node’ları döner |
| `links read` | `from_heading` / `to_heading` dolu |

---

## 33. Son not

Folisift’in en büyük değeri **deterministik ve yerel** olması. Bu korunmalı.

En büyük risk, “passed” kelimesinin fazla şey vaat etmesi. Coverage 7/7 derken 141 başlık bekleyen kullanıcı, “tamam” sanıyor. Layout “audit passed” derken başlık iki sayfaya bölünebiliyor. D2 “valid” derken karar ağacı 12 düğüme sıkışıyor.

Bu üç şey düzeltilirse — **tam coverage, sürekli layout, yapılandırılmış hata** — Folisift “çalışan bir araç”tan “güvenilir bir üretim hattı”na geçer.

Benim için en kritik üçlü:

1. **Coverage manifest** — “eksiksiz” iddiasının tek kaynağı.
2. **Chapter/columns keep-with-next** — PDF’in kırık görünmemesi.
3. **D2 karar ağacı desteği** — hukuk/medicine gibi alanlarda asıl değer burada.

Geri kalan her şey bu üçünün üstüne konfor katmanı.

---

## 34. Ek: bu oturumda üretilen dosyalar

| Dosya | Ne için |
|---|---|
| `output/is-hukuku.pdf` | 26 sayfalık ana kılavuz |
| `output/is-hukuku-irac.pdf` | 11 sayfalık IRAC çalışma kitabı |
| `output/is-hukuku-irac-forest-sage.pdf` | IRAC forest-sage varyantı, 11 sayfa |
| `output/is-hukuku-irac-berry.pdf` | IRAC berry varyantı, 11 sayfa |
| `output/preview-reader-forest-sage.pdf` | 4 sayfalık stil önizlemesi |
| `output/preview-reader-berry.pdf` | 4 sayfalık stil önizlemesi |
| `output/preview-accessible-high-contrast.pdf` | 4 sayfalık erişilebilir önizleme |
| `output/is-hukuku-cards.csv` | 8 Anki kartı |
| `output/is-hukuku-irac-cards.csv` | 5 Anki kartı |
| `guides/is-hukuku/guide.md` | Ana kılavuz kaynağı |
| `guides/is-hukuku/data/topic-map.md` | 141 satırlık manuel konu matrisi |
| `guides/is-hukuku/diagrams/fesih-karar-agaci.json` | 12 düğümlü dikey karar ağacı |
| `guides/is-hukuku-irac/guide.md` | 10 olaylık IRAC kaynağı |
| `guides/is-hukuku/data/claims.md` | Ana kılavuz claim raporu |
| `guides/is-hukuku-irac/data/claims.md` | IRAC claim raporu |

## 35. Ek: tekrar için minimal komutlar

```bash
# Kaynak analizi
folisift analyze --source sources/is-hukuku.pdf --out data/scan-is-hukuku.json

# Build + coverage
folisift build --guide is-hukuku --check

# Audit
folisift audit --pdf output/is-hukuku.pdf --guide is-hukuku --expected-pages 26 --strict

# D2
folisift diagram validate --guide is-hukuku --all
folisift diagram build --guide is-hukuku --all
folisift diagram status --guide is-hukuku

# Kart / claim
folisift cards --guide is-hukuku-irac --out output/is-hukuku-irac-cards.csv
folisift claims --guide is-hukuku --out guides/is-hukuku/data/claims.md

# Stil keşfi
folisift styles list
folisift styles palettes
folisift styles fonts
folisift styles recommend --structure irac --domain law --intent practice --target-pages 10-12

# Semantik köprü / graph
folisift links build --guide is-hukuku
folisift links read --guide is-hukuku --limit 20
folisift graph build --guide is-hukuku
folisift graph status --guide is-hukuku
```

## 36. Ek: kısa “ne yapardım?” listesi

Eğer Folisift ekibinde bir haftam olsaydı, sırayla şunları yapardım:

1. **Coverage manifest** + gate’i 141 başlığa bağla.
2. **`chapter.keep_with_next`** + `columns.flow` + `table/diagram keep_with_next`.
3. **`Diagnostic` enum** ve MCP’de `audit_guide` + `qa_guide`.
4. **D2 20 node / decision kind / direction / width**.
5. **`restyle_guide`** + `style estimate`.
6. **Türkçe heading parser + TOC reconciliation**.
7. **Page 1-based + tables count + page roles**.
8. **Graph query/path/explain**.
9. **Docs cookbook** ve “sık hatalar”.

Bu dokuz madde, bu oturumda yaşadığım sorunların %90’ını çözer.

---

## 37. Agent verimliliği, token maliyeti ve “beni hızlandırıyor mu?”

### 37.1 Kısa cevap

**Kısmen.** Folisift, bir agent’ın PDF’i elle parse etmesini, LaTeX/Typst kurmasını, sayfa sayfa render etmesini ve audit yapmasını engelliyor. Bu büyük bir kazanç.

Ama MCP’nin ana işi “agent’ı hızlandırmak” ise, henüz **agent-first bir katman** değil; daha çok **deterministik bir üretim backend’i**. Çünkü agent’ın en pahalı iki işi olan **kaynak okuma/sentez** ve **layout iterasyonu** hâlâ büyük ölçüde agent’a kalıyor.

Kısaca:

- PDF üretimi/audit: **Folisift çok iyi**.
- Kaynak anlama: **hâlâ pahalı**.
- Coverage: **manuel**.
- Layout: **deneme yanılma**.
- Stil: **tam build denemesi**.
- Token görünürlüğü: **yok**.

### 37.2 Nerede gerçekten tasarruf sağladı?

| İş | Folisift olmasa | Folisift ile | Kazanç |
|---|---|---|---|
| PDF metin çıkarma | Python/pdfium/ocr | `analyze_source`, `read_source` | Yüksek |
| Sayfa sayfa görsel okuma | Her sayfayı render + image token | Metin katmanı + audit | Yüksek |
| PDF üretimi | Typst/LaTeX elle | `build_guide` | Çok yüksek |
| Layout audit | Gözle kontrol | `audit --strict` | Yüksek |
| D2 şema | Elle SVG/Graphviz | `diagram validate/build` | Orta/yüksek |
| Kart/claim | Elle CSV | `cards`, `claims` | Orta |
| Kaynak referansı | Elle sayfa arama | `search_source` | Orta |
| Stil keşfi | Deneme yanılma | `style preview` | Orta |

Bu tablo, “Folisift gereksiz” demek değil. Tam tersi: **altyapı olarak çok değerli**.

### 37.3 Nerede tasarruf sağlamıyor?

#### 1. Kaynak okuma hâlâ büyük maliyet

`read_source` sayfa aralığı veriyor; 117 sayfalık kaynağı anlamak için 6–8 büyük chunk okudum. Her 40.000 karakterlik chunk, ciddi token demek. MCP bana “şu bölümün özeti, ana kavramları, sayıları, tabloları ve çelişkileri”ni vermiyor.

**Olan:** ham metin + sayfa numarası.  
**Gereken:** section-level extractive brief.

#### 2. Coverage manuel

141 satırlık `topic-map.md` elle yazıldı. Bu, MCP’nin en çok zaman kazandırması gereken yerlerden biriydi; tam tersi oldu. 7 canonical section gate’i yüzünden agent “acaba atladım mı?” diye ayrıca matris kuruyor.

#### 3. Layout iterasyonu pahalı

C bölümü başlığı 3 sayfaya bölündü; Şekil 2 tek başına sayfada kaldı; Şekil 3 yüzünden 26 sayfa 27 oldu. Her biri için PDF render + görsel okuma gerekti. `audit` bunların bir kısmını yakalamadı.

**Olan:** “underfilled pages” gibi kısa hata.  
**Gereken:** “chapter opener ile ilk blok ayrıldı; şu bloğu şuraya taşı” gibi layout teşhisi.

#### 4. Stil denemesi tam build

`accessible` 14 sayfa olduğunu ancak full build’de gördüm. `style preview` jenerik. Her stil denemesi ayrı build + audit + görsel kontrol.

#### 5. MCP çıktıları LLM için fazla büyük

Örnekler:

- `style info` kocaman `tokens`, `palette`, `strings`, `fonts` döndürüyor; ben sadece palette ve font istiyordum.
- `coverage` 7 section’ı tekrar tekrar, tam başlıklarla döndürüyor.
- `links read` 20 linkte bir sürü alan döndürüyor; heading adı yok.
- `build_guide` coverage objesi büyük; özet yok.

**Gereken:** her araçta `detail: "summary" | "normal" | "full"`, `max_tokens`, `fields`, `format: "md" | "json"`.

#### 6. Token bütçesi görünmüyor

MCP bana “bu çağrı kaç token tuttu?” demiyor. Agent, context penceresini bütçeleyemiyor. `prepare_guide` “target_pages” alıyor ama “token_budget” veya “effort” almıyor.

#### 7. Tek çağrıda QA yok

Build + audit + cards + claims + visual ayrı ayrı. Her araç çağrısı overhead; her çıktı context’e giriyor. `qa_guide` tek çağrıda kompakt rapor vermeli.

### 37.4 Maliyet tablosu

| Aşama | MCP katkısı | Agent’a kalan | Verdict |
|---|---|---|---|
| Kaynak analizi | Envanter, sayfa aralığı | 117 sayfayı anlamak, sentez | **Pahalı** |
| Kapsam | 139 heading | 141’e tamamlama, matris | **Çok pahalı** |
| Yazma | Scaffold (zayıf) | 7.000 kelime yazma | **Kaçınılmaz** |
| Layout | Build/audit | Görsel kontrol, taşıma | **Pahalı** |
| D2 | Validate/build | 3 kez yeniden tasarım | **Orta/pahalı** |
| Stil | Preview | Full build denemesi | **Orta** |
| QA | Audit | Ayrı araçlar, yorumlama | **Orta** |
| Kart/claim | CSV/MD | Az | **İyi** |

### 37.5 Agent-first öneriler

Bunlar doğrudan token/zaman tasarrufu sağlar:

1. **`read_section`** — sayfa değil, heading ile okuma:
   ```json
   {"source":"is-hukuku.pdf","heading":"4.4.4","mode":"full|summary|claims"}
   ```
2. **`section_brief`** — her heading için 300–600 token:
   - ana kavramlar
   - sayılar/tarihler
   - tablo satırları
   - çelişkiler (“Not 1 / Not 2”)
   - kaynak sayfası
   - önerilen guide başlığı
3. **`guide_plan`** — section başına kelime/sayfa bütçesi:
   ```json
   {"section":"B3/12","target_words":900,"source_pages":"55-69","must_cover":["m.24","m.25","kıdem"]}
   ```
4. **`qa_guide`** — tek çağrıda build + coverage + audit + cards + diagrams; kompakt özet.
5. **`build` draft** — check fail olsa bile `_draft.pdf` ve yapılandırılmış tanılama.
6. **`coverage_manifest`** — otomatik 141 satır; manuel matris yok.
7. **`style_estimate`** — yazmadan önce “bu stil 14 sayfa tutar” uyarısı.
8. **`restyle_guide`** — içerik hash’i değişmeden palet/stil değiştir.
9. **`diagram_fit`** — “bu diagram şu sayfaya sığar mı, kaç sayfa kaplar?” önizlemesi.
10. **`citation_snippet`** — “şu claim için kaynak cümlesi” döndür; agent tekrar sayfa okumasın.
11. **`verify_citations`** — claim ↔ kaynak sayfa eşleşmesi; `claims` sadece “missing” demesin.
12. **`incremental_build`** — sadece değişen section’ı yeniden işle; section hash.
13. **`cost_report`** — kaynak karakteri, okunan karakter, guide kelimesi, tool call, build sayısı, tahmini token.
14. **`effort` / `token_budget`** — `prepare_guide({effort:"fast"})` daha az çıktı, `deep` daha çok.

### 37.6 Başarı metriği ne olmalı?

MCP’nin başarısı “kaç tool var?” değil, şu metriklerle ölçülmeli:

| Metrik | Neden |
|---|---|
| `time_to_first_draft` | Agent ne kadar hızlı ilk PDF’i görüyor? |
| `time_to_pass` | İlk draft’tan audit pass’e ne kadar? |
| `tool_calls_per_section` | Gereksiz çağrı var mı? |
| `tokens_in / tokens_out` | Context maliyeti |
| `source_chars_read / source_chars_total` | Kaynağın ne kadarını okumak zorunda kaldı? |
| `manual_fix_count` | Agent kaç elle müdahale yaptı? |
| `build_iterations` | Layout/stil kaç deneme istedi? |
| `coverage_manual_rows` | Kaç satır elle yazıldı? |
| `visual_page_reads` | Kaç sayfa görsel okundu? |

Bu oturumda bu metriklerin çoğu kötüydü: coverage manuel, build iterasyonu çok, visual page read var, token görünürlüğü yok.

### 37.7 Sonuç

Folisift **altyapı olarak** agent’ın işini kolaylaştırıyor: PDF üretimi, audit, D2, kart/claim.

Folisift **agent deneyimi olarak** henüz yeterince hızlı değil: kaynak okuma, coverage, layout ve stil denemeleri hâlâ pahalı.

Eğer hedef “agent bir PDF’i alıp 1–2 saatte, düşük token’la, az manuel müdahaleyle bitirsin” ise, gereken şey daha fazla tool değil; **daha akıllı, daha kompakt, section-aware bir bağlam katmanı**.

Benim için ideal akış:

```text
analyze_source
  → section_brief (her heading için 300–600 token)
  → guide_plan (section başına kelime/sayfa)
  → host yazar
  → build (draft + diagnostics)
  → qa_guide (tek kompakt rapor)
  → restyle_guide (opsiyonel)
```

Bu akışta agent 117 sayfayı sayfa sayfa okumaz; **section brief** ile çalışır, gerektiğinde `read_section` ile derine iner. Folisift’in asıl vaadi bu olmalı.

---

## 38. Model2vec-rs / `links`: ne işe yaradı, ne yaramadı?

### 38.1 Ne çalıştı?

`links build` modeli:

```text
potion-multilingual-128M-int8
186 links
354 sections
```

Yerel, deterministik ve hızlı. Dış servis yok; bu iyi.

Bana önerdiği eşleşmelerden bazıları gerçekten anlamlıydı:

- Asıl işveren–alt işveren ↔ SGK’da alt işveren (s. 14 / 101)
- Fesih ↔ sendikal güvence (s. 60–63 / 77)
- Fazla çalışma ↔ ücret/zamanaşımı (s. 31 / 39–40)
- Hafta tatili/UBGT/yıllık izin ↔ ücret (s. 43–47)
- TİS ↔ grev/lokavt (s. 81–92)
- İş kazası ↔ SGK/GSS (s. 104–107 / 114–117)

Bu köprüleri F bölümünde kullandım; her birini kaynak sayfasından doğruladım.

### 38.2 Ne yaramadı?

Model2vec **anlam çıkarma** aracı değil; **benzerlik** aracı. Bana “bu iki blok birbirine benziyor” diyor, “bu blokta ne anlatılıyor?” demiyor.

Eksikler:

1. **Özet yok.** 117 sayfalık kaynağı anlamak için hâlâ sayfaları okumam gerekti.
2. **Heading etiketi yok.** Çıktı `is-hukuku:p44:s131` gibi; hangi başlık olduğunu bilmiyorum.
3. **Önem sırası yok.** 186 link arasında hangisi kritik, hangisi gürültü, belli değil.
4. **Cluster/topic map yok.** “Bu 10 bölüm aynı konuyu anlatıyor” demiyor.
5. **Coverage’a katkı yok.** 141 başlığın hangisi guide’da eksik, göstermiyor.
6. **Çelişki tespiti yok.** “Not 1 / Not 2” farkını yakalamıyor.
7. **“Sırada ne okumalıyım?” yok.** Agent için okuma kuyruğu üretmiyor.
8. **Açıklama yok.** `relation: semantically_similar_to` dışında neden benzer olduğunu söylemiyor.

### 38.3 Maliyet etkisi

| Alan | Model2vec etkisi |
|---|---|
| Kaynak okuma | **Yok denecek kadar az** |
| Bölümler arası köprü | **Orta** |
| F bölümü yazımı | **Orta** |
| Coverage | **Yok** |
| Layout | **Yok** |
| Token maliyeti | Düşük overhead, düşük tasarruf |
| Güven | Manuel doğrulama gerektiriyor |

Yani model2vec-rs **“nice-to-have” bir inceleme yardımcısı**; kaynak anlama maliyetini düşüren ana motor değil.

### 38.4 Daha iyi kullanımı

Model2vec’i “okuyucu” değil, **router** olarak kullanmak gerekir:

1. **`source_topics`** — heading’leri embedding ile cluster’la; her cluster için temsilci başlıklar + sayfa aralığı.
2. **`section_brief`** — her guide section için en benzer 5–10 source bloğunu seç; sonra extractive özet çıkar:
   - ana kavramlar
   - sayılar/tarihler
   - tablo satırları
   - çelişkiler
   - kaynak sayfaları
3. **`coverage_hint`** — her source heading’i guide heading’lerine eşle; eksik kalanları göster.
4. **`query_source`** — semantik arama + heading filtresi; `search_source`’un ham sayfa sonuçlarından daha iyi.
5. **`links read`** — `from_heading`, `to_heading`, `guide_section`, `why` alanları.
6. **`cluster_map`** — D2 ile “kaynak konu haritası”; agent tek bakışta hangi bölümlerin birlikte okunacağını görsün.
7. **`read_next`** — “bu guide section için sıradaki 3 kaynak bloğu” önerisi.

### 38.5 Sonuç

Model2vec-rs bu oturumda **faydalıydı ama marjinaldi**. Bana birkaç iyi köprü verdi; fakat 117 sayfayı anlama maliyetini düşürmedi. Çünkü çıktı “benzerlik” düzeyinde kaldı; “anlam” düzeyine çıkmadı.

Doğru kullanım: **embedding ile seç, extractive/LLM ile özetle, agent’a kısa brief ver.** O zaman model2vec gerçekten token tasarrufu sağlar.

---

## 39. Liteparse OCR bu kaynakta fayda sağlar mıydı?

### 39.1 Kısa cevap

**Bu kaynak için hayır.** Hatta muhtemelen **zarar** verirdi.

Çünkü `analyze_source` çıktısı:

```text
117 sayfa
36.439 kelime
18 tablo
0 düşük metinli sayfa
```

Yani PDF’in **gömülü metin katmanı** vardı. OCR, var olan metni yeniden tanımak olurdu; yeni bilgi eklemezdi.

### 39.2 OCR ne zaman faydalı olur?

| Durum | OCR faydası |
|---|---|
| Taranmış PDF, metin katmanı yok | **Yüksek** |
| Düşük metinli sayfa (`low_text_pages > 0`) | **Yüksek** |
| Tablo/görsel içindeki metin | **Orta** |
| El yazısı, kaşe, imza | **Orta/düşük** |
| Bozuk encoding, eksik harf | **Orta** |
| Döndürülmüş / eğik sayfa | **Orta** |
| Bu kaynak: gömülü metin var | **Yok** |

### 39.3 Neden bu kaynakta fayda sağlamazdı?

1. **Metin katmanı zaten var.** OCR aynı metni ikinci kez üretir; duplicate/çelişki riski.
2. **Türkçe karakter riski.** `İ/ı/ş/ğ/ç` OCR’da sık karışır; hukuki metinde anlam değişir.
3. **Sayı/tarih riski.** `4857`, `5510`, `270 saat`, `6 iş günü` gibi kritik değerler OCR hatasına açık.
4. **Tablo yapısı.** OCR tablo hücrelerini her zaman doğru ayırmaz; 18 tablo için ek doğrulama gerekir.
5. **Maliyet.** 117 sayfa render + OCR = ciddi CPU/GPU/zaman; agent için ek token ve gecikme.
6. **Asıl sorun OCR değil.** Bizim sorunlarımız heading parsing, coverage, layout, D2 idi. OCR bunların hiçbirini çözmez.

### 39.4 OCR hangi durumda işe yarardı?

Sadece **hedefli** kullanım:

- `low_text_pages` listesindeki sayfalar için OCR.
- Tablo/görsel bölgeleri için ayrı table OCR.
- OCR metnini text layer ile karşılaştır; düşük güvenli kısımları işaretle.
- Türkçe sözlük + sayı/tarih doğrulaması.
- OCR’ı tüm PDF’e değil, sadece eksik sayfalara uygula.

### 39.5 Liteparse için öneri

`analyze_source` şu politikayı desteklesin:

```json
{
  "ocr": "auto",
  "ocr_policy": {
    "low_text_pages_only": true,
    "min_chars_per_page": 200,
    "table_regions": true,
    "language": "tr"
  },
  "ocr_report": {
    "pages_ocr": 0,
    "pages_skipped": 117,
    "confidence": null
  }
}
```

Varsayılan `auto` olsun:

- Metin katmanı yeterliyse OCR’ı atla.
- Sadece düşük metinli sayfalarda OCR çalıştır.
- OCR metnini text layer ile birleştirirken duplicate temizliği yap.
- Her OCR sayfası için güven skoru ve “manual review” bayrağı ver.

### 39.6 Sonuç

OCR, **pikseli metne** çevirir. Benim maliyetim pikselde değildi; **metni anlamak, kapsamı doğrulamak ve layout’u düzeltmekteydi**.

Bu kaynakta OCR açmak:

- daha yavaş,
- daha pahalı,
- daha riskli,

olurdu; buna karşılık anlama maliyetini düşürmezdi.

Doğru yatırım OCR değil; **`section_brief` + `read_section` + `coverage_manifest` + `qa_guide`**.

---

## 40. Dijital Dönüşüm kılavuzu: 207 sayfa, 3 kaynak, 21 düşük metinli sayfa

İş Hukuku tek ve homojen bir kaynaktı. Dijital Dönüşüm kılavuzu ise üç farklı türde kaynağı birleştirdi:

- `Ders_Notu_1_VİZE` — 31 slayt
- `DijitalÇağdaÜretimYönetimi` — 134 sayfa
- `DijitalDönüşümdeÜrünYönetimi` — 42 sayfa

Toplam **207 kaynak sayfası**, **21 düşük metinli sayfa**, **41 canonical section**, **4 D2 diagram**, **12 tablo**, **40 terimli sözlük** ve **19 sayfalık** kılavuz.

Bu, tek kaynaklı İş Hukuku’ndan çok daha zor bir sentezdi.

### 40.1 Kaynak heterojenliği

Üç kaynak aynı konuyu farklı granularity ile anlatıyor:

- Slaytlar: tanım, bileşen, yol haritası; az metin, çok görsel.
- Kitap: operasyonel mekanizmalar; uzun alt bölümler, tablolar, örnekler.
- Rapor: ürün yönetimi ve kültür; kavramsal çerçeve, metrikler, “karar borcu” gibi özgün kavramlar.

Zorluklar:

1. **Tekrarları ayıklamak.** Üç kaynakta da “dijital dönüşüm nedir?” anlatılıyor; aynı tanımı üç kez yazmamak gerekiyor.
2. **Kaynak sırasını korumak.** Slayt → kitap → rapor sırası korunmalı; ama konu bütünlüğü de bozulmamalı.
3. **Terminoloji birleştirmek.** DPP, ESPR, ERP, PLT, MVP, KPI, UX, IoT, CPS, DIKW gibi kısaltmalar farklı kaynaklarda farklı açıklanıyor.
4. **Kaynak içi farkları korumak.** Örneğin DPP takvimi: 2026 genel hazırlık, 18 Şubat 2027 batarya pasaportu fiili zorunluluk. Bu fark “Açıklayıcı not” ile korundu.
5. **Slayt/görsel sayfaları metne çevirmek.** 21 düşük metinli sayfa; bazıları bölüm ayırıcı, bazıları dekoratif, biri tablo (s. 26).

### 40.2 21 düşük metinli sayfa ve görsel audit maliyeti

`visual-audit.json` 21 karar içeriyor:

- `Ders_Notu_1_VİZE`: 11 sayfa (2, 6, 12, 15, 16, 19, 21, 23, 25, 27, 31)
- `DijitalÇağdaÜretimYönetimi`: 8 sayfa (1, 2, 5, 7, 26, 61, 133, 134)
- `DijitalDönüşümdeÜrünYönetimi`: 2 sayfa (7, 33)

Her biri için:

- `decision`: `text`, `table`, `decorative` vb.
- `reason`: neden bu karar?
- `priority`: high/low

Bu, **21 sayfa görsel okuma** demek. Agent için her sayfa render + image token + yorum demek. İş Hukuku’nda 0 düşük metinli sayfa vardı; burada 21.

**Önemli ayrım:** Bu 21 sayfanın çoğu dekoratif veya bölüm ayırıcı. OCR hepsini “metne” çevirse bile “bu sayfa sınav için gerekli mi?” kararını vermez. OCR yalnızca metin çıkarır; **görsel rolü** anlamaz.

### 40.3 Coverage: 41 canonical, ama 31 slayt gate dışında

`coverage.md`:

- `Ders_Notu_1_VİZE` → **0 canonical section**, 11 low-text slide
- `DijitalÇağdaÜretimYönetimi` → 27 section, 8 low-text
- `DijitalDönüşümdeÜrünYönetimi` → 14 section, 2 low-text
- Toplam **41 canonical**, gate “GEÇTİ”

Yani 31 slaytlık kaynak **coverage gate’e hiç girmiyor**. G bölümünde “Kapsandı” deniyor; ama gate 0 section görüyor. Bu, İş Hukuku’ndaki 7/141 sorununun çok kaynaklı versiyonu.

Ayrıca:

- `DijitalDönüşümdeÜrünYönetimi` 14 section’ın tamamında `page: —`. Yani kaynak sayfa eşleşmesi yok; citation’lar `[s. x]` görünse de coverage’da sayfa yok.
- `DijitalÇağdaÜretimYönetimi` section’larında `body_chars` tekrar ediyor: 1394, 1680, 1448, 1000, 1515, 899, 1482, 607, 927, 1409. Yani birden çok heading aynı gövdeyi paylaşıyor; “her heading kendi içeriğine sahip” değil.

Bu üçü birleşince “41/41 geçti” yine yanıltıcı oluyor.

### 40.4 Layout ve sayfa bütçesi

Hedef **18–20 sayfa**; final **19 sayfa**. Ama süreç kolay değildi:

- **Şekil 2 ve 3 fazla büyüktü.** Küçültülmeleri gerekti; yoksa sayfa bütçesi ve akış bozuluyordu.
- **Kapak/başlık aralığı** düzeltildi; başlık ve alt başlık arasındaki boşluk görsel olarak sorunluydu.
- **4 D2 diagram** sayfa içine yerleştirildi; bazıları sayfayı dolduruyor, bazıları küçültülüyor.
- **12 tablo** var; biri (s. 26, özel satın alma hedefleri) düşük metinli sayfadan tabloya çevrildi.

Bu, İş Hukuku’ndaki “şekil sayfayı kaplıyor / sayfa boş kalıyor” sorununun çok kaynaklı ve daha görsel versiyonu.

### 40.5 Kaynak sayfa numarası ve citation sorunu

- Slayt kaynağında “slayt numarası = PDF sayfası” varsayımı var; bu tutarlı.
- Kitap kaynağında sayfa numaraları 40–122 aralığında; coverage’da var.
- Ürün raporunda 14 section’ın tamamında `page: —`; citation’ların sayfa eşleşmesi belirsiz.
- Bu, “hangi bilgi hangi sayfada?” sorusunu zayıflatıyor. Hukukta olduğu gibi dijital dönüşümde de kaynak izlenebilirliği önemli.

### 40.6 Dijital Dönüşüm’de OCR faydalı olur muydu?

İş Hukuku’nun aksine, burada **hedefli OCR/vision** faydalı olabilirdi:

- 21 düşük metinli sayfanın bir kısmı gerçekten metin içeriyor olabilir.
- Özellikle slayt ve rapor sayfalarındaki görsel metinler OCR ile çıkarılabilir.
- Ama çoğu dekoratif/bölüm ayırıcı; OCR “bu sayfa gerekli mi?” kararını vermez.
- Tam PDF OCR yine gereksiz; **sadece `low_text_pages` + tablo bölgeleri** için OCR.

Yani:

```text
İş Hukuku: OCR gereksiz (metin katmanı var)
Dijital Dönüşüm: hedefli OCR/vision faydalı olabilir (21 low-text)
```

Ama OCR tek başına yeterli değil; **visual role classification** gerekir.

### 40.7 Dijital Dönüşüm için öneriler

1. **`source_brief`** — her kaynak ve section için 300–600 token özet.
2. **`visual_brief`** — her düşük metinli sayfa için:
   - OCR metni (varsa)
   - sayfa rolü (`chapter_divider`, `decorative`, `table`, `diagram`, `content`)
   - sınav için gerekli mi?
   - öneri: `text`, `table`, `redraw`, `skip`
3. **`coverage_manifest`** — her kaynak için ayrı; 31 slayt da gate’e girsin.
4. **`source_page_map`** — özellikle `page: —` olan 14 section için sayfa eşleşmesi.
5. **`source_topics`** — üç kaynağın tekrar eden konularını cluster’la; “hangi kaynak hangi konuda otoriter?” göster.
6. **`figure_fit`** — D2 diagram’ın sayfaya sığip sığmayacağını önceden söyle; Şekil 2–3’ü küçültmeden önce.
7. **`page_budget`** — 18–20 hedefi için bölüm başına sayfa tahmini.

### 40.8 Dijital Dönüşüm’den çıkan ders

- Çok kaynaklı sentez, tek kaynaktan **çok daha pahalı**.
- 21 düşük metinli sayfa, görsel audit’i zorunlu kılıyor.
- Coverage gate her kaynağı ayrı görmeli; “41/41” yeterli değil.
- Sayfa numarası eşleşmeyen kaynak (`page: —`) citation güvenini düşürüyor.
- OCR/vision, düşük metinli sayfalarda **hedefli** kullanılmalı; tam OCR değil.

Dijital Dönüşüm kılavuzu sonuçta **19 sayfa** ve audit’ten geçti; ama bunun maliyeti İş Hukuku’ndan yüksekti: 3 kaynak, 21 görsel sayfa, 41 canonical section, 4 D2, 12 tablo.

---

## 41. Context7 incelemesi: LiteParse, D2, Model2Vec — kullanmadığımız yetenekler

Context7’de incelediğim kütüphaneler:

- **LiteParse** — `/run-llama/liteparse`
- **D2** — `/websites/d2lang_tour`, `/d2lang/d2-docs`
- **Model2Vec** — `/minishlab/model2vec`
- **FastEmbed-rs** — `/anush008/fastembed-rs` (Model2Vec için Rust tarafında alternatif/reranker)

Not: Context7, `model2vec-rs` Rust crate’ini ayrı bir kaynak olarak indekslemiyor. Model2Vec’in Python dokümanları + FastEmbed-rs + resmi Rust README’si üzerinden değerlendirdim.

### 41.1 LiteParse: kullanmadığımız en değerli özellikler

Folisift `LiteparseParser` içinde şunları kullanıyor:

```rust
LiteParseConfig {
    ocr_enabled: false,
    output_format: OutputFormat::Json,
    extract_blocks: true,
    quiet: true,
    max_pages: 10_000,
    ..Default::default()
}
```

Yani `extract_blocks` zaten açık. Ama LiteParse’ın daha birçok yeteneği var:

| LiteParse yeteneği | Ne yapar? | Folisift’te kullanılıyor mu? | Bizim için faydası |
|---|---|---|---|
| `is_complex` / `lit is-complex` | Sayfa başına `needs_ocr`, `reasons`, `layout` sinyalleri | **Hayır** | OCR’ı sadece gerekli sayfalara yönlendirme |
| `extract_blocks` | heading/paragraph/list/table/figure blokları + bbox | **Evet** | Zaten iyi; table cell/bbox daha iyi kullanılabilir |
| `extract_text_metadata` | `font_height`, `font_weight`, `fill_color`, `rotation`, `mcid` | **Hayır** | Heading/başlık tespiti, tablo/figure ayrımı |
| `extract_vector_graphics` | Shape/line + bbox | **Hayır** | Figure-only sayfa, tablo çizgileri, görsel audit |
| `extract_images` | Image bytes/metadata; `is_complex` ayrıca `image_coverage` verir | **Hayır** | Dekoratif vs içerik görseli ayrımı |
| `extract_structure_tree` | Tagged-PDF mantıksal ağaç | **Hayır** | Başlık hiyerarşisi, tablo yapısı |
| `extract_content_bounds` | Sayfadaki gerçek içerik bbox’ı | Kısmen | Underfill/overflow, figure-only sayfa |
| `target_pages` | Sadece belirli sayfaları parse et | **Hayır** | Hedefli OCR/analiz |
| `ocr_language="tur"` | Türkçe OCR | **Hayır** (OCR kapalı) | Gerekirse doğru Türkçe OCR |
| `ocr_server_url` | Harici OCR sunucusu | **Hayır** | Ağır OCR’ı ayırma |
| `dpi`, `num_workers` | OCR render/paralellik | **Hayır** | Maliyet/kalite ayarı |
| `continue_on_page_error` / `page_errors` | Bozuk sayfayı atla, raporla | **Hayır** | Dayanıklılık |
| `output_format="markdown"` | Heading/tablo/list/görsel/link Markdown | **Hayır** | LLM’e doğrudan yapısal metin |
| `keep_headers_footers` | Header/footer koru | Varsayılan false | Citation temizliği |
| `extract_annotations` / form fields | Annotation/form | **Hayır** | Hukuk formları için ileride |
| `preserve_very_small_text` | Küçük metni koru | **Hayır** | Dipnot/küçük yazı |

Folisift’te `read_blocks` MCP aracı da var; biz kaynağı `read_source` ile sayfa sayfa okuduk. `read_blocks` heading/table/figure bloklarını bbox ile döndürüyorsa, `section_brief` için daha iyi bir omurga olurdu. Bu oturumda kullanılmadı.

#### En kritik ikisi

**1. `is_complex`**

LiteParse’ın `is_complex` çıktısı şunları veriyor:

- `needs_ocr`
- `reasons`: `scanned`, `no-text`, `sparse-text`, `embedded-images`, `garbled`, `vector-text`, `annotation-text`
- `layout`: `column_count`, `ruled_table_count`, `ruled_table_coverage`, `figure_count`, `figure_coverage`, `is_complex`, `reasons` (`multi-column`, `table-likely`, `dense-graphics`)
- `text_coverage`, `image_coverage`, `largest_image_coverage`, `full_page_image`, `uncovered_vector_area`, `is_garbled`

Bu, İş Hukuku’nda “0 low-text, OCR gerekmez” kararını **kanıtla** verirdi. Dijital Dönüşüm’de 21 sayfanın neden low-text olduğunu (`scanned`, `sparse-text`, `embedded-images`, `dense-graphics`) söylerdi. Biz bunu `visual-audit.json` ile elle yaptık.

**2. `extract_text_metadata` + `extract_vector_graphics` + `extract_images`**

Bu üçü birlikte:

- Başlıkları font büyüklüğü/kalınlığıyla ayırmaya,
- Figure-only sayfaları tespit etmeye,
- Tablo çizgilerini/bbox’larını bulmaya,
- Dekoratif görselleri içerik görsellerinden ayırmaya

yardım eder. Özellikle:

- İş Hukuku’nda 267/139/141 heading tutarsızlığı,
- Dijital Dönüşüm’de 21 low-text sayfa,
- Şekil 2/3’ün sayfayı kaplaması

bu yeteneklerle daha deterministik çözülürdü.

### 41.2 LiteParse tablo çıkarma: kullanmadığımız altın

`extract_blocks` tablo bloklarında:

- `header`
- `rows`
- her hücrede `text` + `bbox`

veriyor. Bu, 18 İş Hukuku tablosu ve 12 Dijital Dönüşüm tablosu için **hücre hücre** doğrulama demek.

Biz tabloları çoğunlukla metinden özetledik. `folisift table extract` CLI’ı var; MCP’de yok. Bu araç kullanılsaydı:

- Tablo hücreleri kaynak sayfayla eşleşir,
- “Tablo 1” numaralandırma sorunu azalır,
- Tablo verisi guide’a daha sadık aktarılır,
- `stats.tables` daha doğru olur.

### 41.3 D2: sadece TALA değil, çok daha fazlası var

Context7 D2 dokümanlarına göre D2’de üç layout engine var:

- `dagre` — hızlı, katmanlı/hiyerarşik
- `elk` — olgun, container width/height destekli
- `tala` — D2’nin kendi motoru; `near`, `top`, `left`, per-container direction

Folisift şu an yalnızca `tala` kullanıyor ve spec’te `layout` başka değeri reddediyor.

Kullanmadığımız D2 yetenekleri:

| D2 yeteneği | Ne yapar? | Folisift’te var mı? | Bizim için faydası |
|---|---|---|---|
| `dagre` / `elk` layout | Farklı graph layout’ları | **Hayır** | Karar ağacı/akış için daha iyi aspect |
| Per-container `direction` | Container içinde ayrı yön | **Kısmen** (groups var, direction yok) | İşçi/İşveren lane’leri; dikey gövde + yatay dal |
| `width` / `height` | Node/container boyutu | **Hayır** | Düğüm boyutu, sayfa doldurma |
| `near` | Node’u başka node’a yaklaştır | **Hayır** | Sonuç düğümlerini dallara yaklaştırma |
| `top` / `left` | Node’u sabitle | **Hayır** | Root’u sayfa üstüne sabitleme |
| `classes` | Node stilleri (renk, border) | **Hayır** (class diagram hariç) | Haklı/geçersiz dalları renklendirme |
| `tooltip` | Hover açıklaması | **Hayır** | Kaynak sayfa/citation |
| `link` | Tıklanabilir link | **Hayır** | Kaynak PDF sayfasına link |
| `shape: sql_table` | Tablo şekli | Kısmen (`tables`) | Karşılaştırma matrisi |
| `shape: class` | UML class | Var (`classes`) | Kavram hiyerarşisi |
| `shape: sequence_diagram` | Sekans | Var (`kind: sequence`) | İşe iade/grev süreci |
| `grid` | Grid container | **Hayır** | Fesih türleri matrisi |
| `vars` / `imports` / `theme` | Tema, değişken, import | **Hayır** | Palet entegrasyonu, tekrar kullanım |
| `sketch` / `icons` | El çizimi/ikon | **Hayır** | Görsel çeşitlilik (hukukta ölçülü) |

#### En kritik ikisi

**1. Per-container direction + `near`/`top`/`left`**

Fesih karar ağacı 12 düğümle zor sığdı. Eğer:

- `İşçi` ve `İşveren` için iki container,
- her container içinde `direction: down`,
- root için `top: 0`,
- sonuç düğümleri için `near`,

olsaydı, dikey ve sayfayı dolduran bir ağaç çıkardı. Biz bunu TALA’nın global `direction`’ı ile denedik; yatay/aşırı geniş sonuç aldık.

**2. `classes` ile anlamsal renk**

Haklı/geçersiz/geçerli dalları renklendirmek, 12 düğümlü karmaşık ağacı okunur kılar. Folisift `apply_graph_style` ile her şeye aynı accent’i veriyor; `classes` yok.

### 41.4 Model2Vec: sadece `links` değil, clustering/classification/retrieval

Context7’ye göre:

- `potion-multilingual-128M` **101 dil** destekliyor.
- LaBSE’nin **%90.86** performansına ulaşıyor.
- **Classification** ve **clustering** görevlerinde güçlü.
- Statik embedding: 50x küçük, 500x hızlı.
- `int8` quantization, PCA, SIF, mean pooling.
- `encode_as_sequence` ile token embedding.

Biz yalnızca `links` için pairwise similarity kullandık. Kullanmadığımız yetenekler:

| Model2Vec yeteneği | Folisift’te var mı? | Bizim için faydası |
|---|---|---|
| Clustering | **Hayır** | `source_topics`: 141 başlığı konu kümelerine ayır |
| Classification | **Hayır** | Heading → guide section eşlemesi; coverage hint |
| Retrieval | Kısmen (`links`) | `section_brief` için top-k source block |
| `encode_as_sequence` | **Hayır** | Terim/başlık hizalama |
| `max_length` / truncation | **Hayır** | Uzun hukuk bölümlerini chunk’lama |
| `int8` / PCA / SIF | Model int8 zaten | Daha küçük/hızlı; kalite ayarı |
| Reranking (FastEmbed-rs) | **Hayır** | Top-k retrieval’ı daha isabetli yapma |

`model2vec-rs` Rust crate’i ayrıca:

- Python bağımlılığı yok.
- Rust’ta yaklaşık **23x** daha hızlı inference.
- `StaticModel::from_pretrained(path_or_repo, token, normalize, subfolder)`
- `encode(&sentences)` / `encode_with_args(&sentences, max_length, batch_size)`
- `max_length` ve `batch_size` ile token/maliyet kontrolü

Context7 bu Rust crate’ini ayrı indekslemedi; resmi README/docs.rs üzerinden doğruladım.

#### En kritik ikisi

**1. Clustering → `source_topics`**

141 başlığı embedding ile cluster’layıp:

- “Bu 8 başlık fesih ailesi”
- “Bu 6 başlık sosyal güvenlik kolları”
- “Bu 5 başlık toplu iş hukuku”

diyebilirdi. Biz bunu elle yaptık.

**2. Classification → `coverage_hint`**

Her source heading’i guide heading’ine eşleyip “bu başlık hangi bölümde?” diyebilirdi. Coverage manifestinin otomatik omurgası olurdu.

### 41.5 FastEmbed-rs: kullanmadığımız reranker

FastEmbed-rs:

- ONNX ile yerel embedding/reranking
- `TextRerank` / `try_new_from_user_defined`
- Offline çalışma
- Model2Vec için alternatif/reranker

Biz kullanmadık. Faydası:

- `section_brief` için top-k 20 source bloğu seç,
- reranker ile ilk 5’e indir,
- LLM’e/agent’a daha az ama daha isabetli bağlam ver.

Maliyeti: ek model, ek latency. Sadece “top-k çok gürültülü” olduğunda kullanılmalı.

### 41.6 Determinizm ve bütçe için önerilen “agent-fast” profili

Kullanıcı “deterministik, tasarruflu, optimize” istiyor. Bunun için tek bir profil öneriyorum:

```json
{
  "profile": "agent-fast",
  "liteparse": {
    "ocr": "auto",
    "ocr_language": "tur",
    "complexity": true,
    "extract_blocks": true,
    "extract_text_metadata": true,
    "extract_vector_graphics": true,
    "extract_images": true,
    "extract_content_bounds": true,
    "target_pages": "auto",
    "continue_on_page_error": true
  },
  "d2": {
    "layout": "tala",
    "allow_layouts": ["tala", "elk", "dagre"],
    "groups": true,
    "per_container_direction": true,
    "node_width_height": true,
    "near_top_left": true,
    "classes": true
  },
  "model2vec": {
    "model": "potion-multilingual-128M-int8",
    "clustering": true,
    "classification": true,
    "retrieval_top_k": 8,
    "max_length": 512
  },
  "outputs": {
    "detail": "summary",
    "max_tokens": 1200,
    "format": "md+json"
  }
}
```

### 41.6.1 Determinizm guardrail’leri

“Deterministik” istiyorsak, yeni yetenekleri açarken şunlar zorunlu:

- `liteparse_version`, `d2_version`, `model_hash`, `layout_engine` rapora yazılsın.
- OCR açılacaksa `tesseract_version` ve `ocr_language` kaydedilsin.
- `section_brief` **extractive** olsun; generative LLM özeti deterministik değil.
- Extractive özet için sabit tie-break (skor eşitse sayfa/paragraf sırası).
- `source_brief` cache anahtarı: `source_digest + model_hash + brief_version`.
- D2 layout engine ve versiyon pinlensin; aynı spec aynı SVG’yi versin.
- `int8`/float quantization sabitlensin; model değişimi rapora yazılsın.
- Reranker kullanılıyorsa model hash’i ve top-k kaydedilsin.

### 41.7 Kullanmamamız gerekenler

Her yeteneği kullanmak iyi değil:

- **Tam PDF OCR** — metin katmanı varsa gereksiz; Türkçe karakter/sayı riski.
- **LLM generative summarization** — determinizm istiyorsak extractive/embedding tabanlı özet.
- **D2 sketch/icons** — hukuk kılavuzunda güvenilirliği düşürebilir; ölçülü kullanılmalı.
- **Reranker her zaman** — sadece top-k gürültülüyse; yoksa ek maliyet.
- **Ağır `extract_text_metadata` her kaynakta** — çıktı boyutu artar; sadece heading/table/figure sorunlarında.

### 41.8 Sonuç: “son damlasına kadar” kullanım

Folisift şu an LiteParse’ın:

- `extract_blocks` + `content_bounds` + low-text char threshold’unu kullanıyor.

Ama şunları kullanmıyor:

- `is_complex` / `complexity`
- `extract_text_metadata`
- `extract_vector_graphics`
- `extract_images`
- `extract_structure_tree`
- `target_pages`
- `ocr_language="tur"`
- `output_format="markdown"`
- `table` hücre/bbox çıkarma

D2’de:

- Yalnızca `tala` + basit node/edge/group.
- `dagre`/`elk`, per-container direction, `width`/`height`, `near`/`top`/`left`, `classes`, `tooltip`/`link`, `grid`, `sequence`/`class`/`sql_table` örnekleri kullanılmıyor.

Model2Vec’te:

- Yalnızca pairwise `links`.
- Clustering, classification, retrieval, `max_length`, reranker kullanılmıyor.

Bunların hepsi deterministik biçimde kullanılabilir. En büyük kazanç sırası:

1. **LiteParse `is_complex` + `extract_text_metadata` + `extract_vector_graphics`** → heading/page role/visual audit.
2. **D2 `groups` + per-container direction + `near`/`top`/`left` + `classes`** → sayfayı dolduran karar ağacı.
3. **Model2Vec clustering + classification + retrieval** → `source_topics`, `coverage_hint`, `section_brief`.
4. **LiteParse `table` hücre/bbox** → tablo doğruluğu.
5. **FastEmbed-rs reranker** → sadece retrieval kalitesi gerektiğinde.

Böylece “her şeyi son damlasına kadar kullan” hedefi, rastgele her özelliği açmak değil; **doğru profilde, ölçülü ve deterministik** kullanmak olur.

### 41.9 Kaynaklar

Context7 üzerinden incelenen kaynaklar:

- LiteParse: `/run-llama/liteparse` — <https://github.com/run-llama/liteparse>
- D2: `/websites/d2lang_tour`, `/d2lang/d2-docs` — <https://d2lang.com/tour/layouts>
- Model2Vec: `/minishlab/model2vec` — <https://github.com/minishlab/model2vec>
- FastEmbed-rs: `/anush008/fastembed-rs` — <https://github.com/anush008/fastembed-rs>
- `model2vec-rs`: <https://github.com/MinishLab/model2vec-rs> (Context7’de ayrı indekslenmedi; resmi README/docs.rs)
