# folisift

**Folisift**, ders slaytlarını Türkçe, paper-like, sınav odaklı çalışma
kılavuzlarına dönüştüren ders-bağımsız bir üretim hattıdır.

İlk uygulama: **BMÜ325 Programlama Dilleri, PD0–PD1** — 22 sayfa, 260/260 slayt.

## Yapı

```
folisift/
├── engine/          # ders-bağımsız motor: build, tools, template, CSS
├── guides/          # her ders/materyal için bir guide bundle
│   └── pd0-pd1/
│       ├── guide.html
│       ├── config.json
│       ├── assets/
│       └── data/
├── docs/            # mimari, yazım, tasarım, ADR, ajan ayarları
├── sources/         # PDF'ler (git dışı)
└── output/          # üretilen PDF'ler
```

- `engine/build.py` — guide config’inden PDF üretir; `--check` kapsam kapısını çalıştırır.
- `engine/tools/` — envanter, görsel sayfa, kapsam denetimi, yeni guide iskeleti.
- `engine/templates/guide.html` — A–G HTML iskeleti.
- `engine/style.css` — paper tasarım sistemi.
- `guides/<slug>/` — içerik, config, guide’a özel SVG’ler, üretilmiş veri.
- `docs/authoring-spec.md` — her ders/konu için ana spesifikasyon.
- `docs/architecture.md` — katmanlar ve veri akışı.
- `docs/authoring.md` — adım adım yazım rehberi.
- `docs/design-system.md` — palet, tipografi, bileşenler.

## Hızlı başlangıç

```bash
# Mevcut guide
python3 engine/build.py --guide guides/pd0-pd1 --check

# veya
make build
make verify
```

PDF: `output/PD0-PD1_Calisma_Ozeti.pdf`

## Yeni ders / materyal

```bash
python3 engine/tools/scaffold.py \
  --slug pd2 \
  --title "Bölüm 2: ..." \
  --course "BMÜ325 Programlama Dilleri" \
  --sources sources/PD2.pdf

python3 engine/build.py --guide guides/pd2 --check
```

Yeni ders = yeni `guides/<slug>/`; `engine/` değişmez.

## Kaynak PDF politikası

`sources/*.pdf` kişisel/ders materyalidir ve **git tarafından izlenmez**.
Yeni bir makinede PDF’leri `sources/` altına elle koyun. Repo public yapılabilir;
`output/` içindeki türev PDF’lerin haklarını ayrıca değerlendirin.

## Gereksinimler

```bash
python3 -m pip install --user -r requirements.txt
```

Poppler araçları (`pdfinfo`, `pdftotext`, `pdftoppm`) sistemde kurulu olmalıdır.

## Dokümantasyon

- [Architecture](docs/architecture.md)
- [Authoring](docs/authoring.md)
- [Authoring specification](docs/authoring-spec.md)
- [Design system](docs/design-system.md)
- [ADRs](docs/adr/)
- [Agent configuration](docs/agents/)
