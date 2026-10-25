# Çalışma Kılavuzu Üretim Hattı

`kitap/`, ders slaytlarından **paper-like, Türkçe, sınav odaklı** çalışma
kılavuzları üretmek için ortak altyapıyı içerir. İlk uygulama:
`PD0–PD1_Calisma_Ozeti.pdf` (22 sayfa).

## Hızlı başlangıç

```bash
# 1) Kaynak envanterini çıkar (bir kez; sonra cache'ten oku)
cd kitap
python3 tools/analyze_sources.py ../sources/PD0.pdf ../sources/PD1.pdf \
  --out data/pd0-pd1.source.json \
  --md  data/pd0-pd1.source.md

# 2) Düşük metinli / görsel slaytları incele
python3 tools/prepare_pages.py ../sources/PD0.pdf \
  --source-json data/pd0-pd1.source.json --filename PD0.pdf \
  --out data/pages/pd0 --cols 2 --rows 2

# 3) Kapsamı denetle
python3 tools/check_coverage.py \
  --source-json data/pd0-pd1.source.json \
  --html pilot.html \
  --out data/pd0-pd1.coverage.md

# 4) PDF üret (kapsam denetimiyle birlikte)
python3 build.py --config config/pd0-pd1.json --check
```

`make` kısayolları:

```bash
make analyze   # PD0 + PD1 envanteri
make pages     # PD0 görsel sayfaları
make check     # başlık kapsamı
make pilot     # check + build
```

## Klasör yapısı

```
folisift/
├── sources/                     # PD0–PD8 kaynak PDF'leri
├── output/                      # üretilen PDF'ler
└── kitap/
    ├── özet.txt                 # ana spesifikasyon / tekrar kullanılabilir prompt
    ├── build.py                 # generic PDF builder
    ├── style.css                # paper tasarım sistemi
    ├── pilot.html               # PD0–PD1 içeriği (HTML)
    ├── config/
    │   └── pd0-pd1.json         # bu kılavuzun yolları
    ├── data/
    │   ├── pd0-pd1.source.json  # otomatik kaynak envanteri
    │   ├── pd0-pd1.source.md    # okunabilir envanter
    │   └── pd0-pd1.coverage.md  # otomatik kapsam raporu
    ├── templates/
    │   └── guide_skeleton.html
    ├── tools/
    │   ├── analyze_sources.py
    │   ├── prepare_pages.py
    │   ├── check_coverage.py
    │   └── scaffold.py
    └── assets/                  # SVG şemalar
```

## Yeni ders / yeni PDF için

1. `özet.txt` içindeki **CONFIG** bloğunu güncelle.
2. Yeni bir config oluştur: `config/<slug>.json`.
3. İstersen iskeleti otomatik oluştur:
   `python3 tools/scaffold.py --slug pd2 --title "..." --sources ../sources/PD2.pdf`
4. `tools/analyze_sources.py` ile kaynak envanterini çıkar (scaffold bunu yapar).
   `content.html` iskeletini kaynak sırasına göre doldur; bölümleri tek seferde
   değil, parça parça yaz ve arada build al.
5. `build.py --config ... --check` ile PDF üret.
6. `data/<slug>.coverage.md` **GEÇTİ** olmadan “tam kapsam” deme.
7. G bölümündeki sayıları `source.json` ile karşılaştır.

## Tasarım profili

`style.css` içindeki paper paleti:

- kâğıt `#FAF6EC`, mürekkep `#211E19`
- tuğla `#8A3E2F`, adaçayı `#42594F`, eski altın `#B08A4F`
- çizgi `#D8CEBD`, kod zemini `#F1EADB`
- Noto Serif gövde, Inter başlık, DejaVu Sans Mono kod

Tasarımı bölüm bazında değiştirmeyin; tüm kılavuz aynı dili konuşmalı.

## Önemli notlar

- **Sayfa numarası = slayt numarası** bu projede. Kaynak referansları
  `[PD0, s. 3]` biçimindedir.
- Düşük metinli slayt sayısı otomatik envanterden gelir; elle saymayın.
  PD0–PD1 için güncel değer: **55** (PD0: 40, PD1: 15).
- `check_coverage.py` yalnızca başlık düzeyini denetler. Tablo, şekil, formül ve
  örnek kapsamı G bölümünde ayrıca elle doğrulanmalıdır.
- `sources/*.pdf` git tarafından izlenmez; kaynak PDF'leri yeni makinede
  `sources/` altına elle koyun.
- WeasyPrint gerekir: `python3 -m pip install --user weasyprint`.

## Bilinen sınırlar

- `pdftotext` bazı slaytlarda metni eksik çıkarır; bu yüzden `prepare_pages.py`
  ile görsel kontrol yapılır.
- Başlık çıkarımı sezgiseldir; `source.md` mutlaka gözden geçirilmelidir.
- PDF’deki tablo/şekil sayıları için kaynak slayt görüntüsü esastır.
