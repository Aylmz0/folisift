# folisift

**Folisift**, ders slaytlarını Türkçe, paper-like, sınav odaklı çalışma
kılavuzlarına dönüştüren bir üretim hattıdır.

İlk uygulama: **BMÜ325 Programlama Dilleri, PD0–PD1** — 22 sayfa, 260/260 slayt.

## Depo yapısı

```
folisift/
├── sources/        # kaynak PDF'ler (PD0–PD8)
├── output/         # üretilen PDF'ler
└── kitap/          # üretim hattı, şablonlar, içerik ve veri
```

- `kitap/özet.txt` — her ders/konu için yeniden kullanılabilir ana spesifikasyon
- `kitap/README.md` — ayrıntılı kullanım ve üretim akışı
- `kitap/pilot.html` — PD0–PD1 kılavuzunun HTML kaynağı
- `kitap/tools/` — kaynak envanteri, görsel sayfa hazırlama, kapsam denetimi, iskelet
- `kitap/config/` — kılavuz bazlı yollar
- `kitap/data/` — üretilmiş kaynak envanteri ve kapsam raporu

## Hızlı başlangıç

```bash
cd kitap
make analyze      # kaynak envanteri
make pages        # düşük metinli slaytları görsel inceleme
make pilot        # kapsam denetimi + PDF
```

PDF çıktısı: `output/PD0-PD1_Calisma_Ozeti.pdf`

## Yeni ders

```bash
cd kitap
python3 tools/scaffold.py \
  --slug pd2 \
  --title "Bölüm 2: ..." \
  --sources ../sources/PD2.pdf

python3 build.py --config guides/pd2/config.json --check
```

## Notlar

- `sources/*.pdf` kişisel/ders PDF'leridir ve **git tarafından izlenmez**;
  yerel kalır. Repo public yapılabilir.
- `output/` üretilen PDF'leri içerir. Public yapmadan önce bu çıktıların
  içerik/haklar durumunu ayrıca değerlendirin.
- PDF üretimi için: `python3 -m pip install --user weasyprint`.
