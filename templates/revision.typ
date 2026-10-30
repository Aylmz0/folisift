// Folisift revision template. Guide content is generated from Markdown and
// must not inject raw Typst. This layout is for exam revision sheets and
// one-page knowledge organisers, not for long-form readers.

// ---- scale and palette --------------------------------------------------
#let body-size = __BODY_SIZE__
#let h1-size = __H1_SIZE__
#let h2-size = __H2_SIZE__
#let h3-size = __H3_SIZE__
#let lead-size = __LEAD_SIZE__
#let table-size = __TABLE_SIZE__
#let caption-size = __CAPTION_SIZE__
#let label-size = __LABEL_SIZE__
#let source-size = __SOURCE_SIZE__
#let code-size = __CODE_SIZE__
#let toc-title-size = __TOC_TITLE_SIZE__
#let toc-sub-size = __TOC_SUB_SIZE__
#let toc-page-size = __TOC_PAGE_SIZE__
#let leading = __LEADING__
#let par-spacing = __PAR_SPACING__
#let list-spacing = __LIST_SPACING__

#let ink = rgb("#211E19")
#let muted = rgb("#5E574B")
#let soft = rgb("#6B6152")
#let faint = rgb("#75664E")
#let brick = rgb("#8A3E2F")
#let sage = rgb("#42594F")
#let gold = rgb("#B08A4F")
#let paper_bg = rgb("#FAF6EC")
#let linen = rgb("#D8CEBD")
#let cream = rgb("#F4ECDE")
#let sand = rgb("#F1EADB")
#let zebra = rgb("#F7F1E5")
#let headerfill = rgb("#EFE7D8")

// ---- shared helpers -----------------------------------------------------
#let tr_upper_str(text) = upper(text.replace("i", "İ").replace("ı", "I"))

#let tr_upper(body) = [
  #show "i": "İ"
  #show "ı": "I"
  #upper(body)
]

#let doc_lang = state("doc-lang", "tr")

#let ui_upper_str(text) = context {
  if doc_lang.get() == "tr" { tr_upper_str(text) } else { upper(text) }
}

#let ui_upper(body) = context {
  if doc_lang.get() == "tr" { tr_upper(body) } else { upper(body) }
}

#let source_ref(body) = box(text(
  font: "Inter",
  size: source-size,
  style: "normal",
  fill: faint,
  tracking: 0.02em,
)[[#body]])

#let fitted_image(path, alt: none) = layout(size => {
  let wide = image(path, width: size.width, alt: alt)
  block(width: size.width)[
    #align(center, if measure(wide).height > 16cm {
      image(path, height: 16cm, alt: alt)
    } else {
      wide
    })
  ]
})

#let fig_label(number, word: "Şekil") = text(
  font: "Inter",
  size: label-size,
  tracking: 0.1em,
  fill: brick,
)[#upper(word + " " + number)]

#let toc_entry(level: 1, body: none, part: none, subs: true) = [
  #metadata((level: level, body: body, part: part, subs: subs)) <toc-entry>
]

#let sheet_title = state("sheet-title", "")

#let running_entry(text) = [
  #metadata((text: text)) <running-head>
]

#let running_header(left_text, right_text) = {
  set text(font: "Inter", size: 7.5pt, fill: rgb("#6F6454"), tracking: 0.12em)
  grid(
    columns: (1fr, auto),
    align(start)[#ui_upper_str(left_text)],
    align(end)[#text(tracking: 0.04em, fill: faint)[#right_text]],
  )
}

// ---- tables -------------------------------------------------------------
#let table_block(caption: none, source: none, label: "Tablo", body) = block(
  breakable: false,
  above: 3mm,
  below: 4mm,
)[
  #if caption != none [
    #context {
      let number = counter("table").get().first() + 1
      counter("table").step()
      block(above: 0pt, below: 1.5mm)[
        #text(font: "Inter", size: caption-size, weight: "semibold", fill: ink)[#label #number]
        #text(font: "Inter", size: caption-size, fill: ink)[ #caption]
      ]
    }
  ]
  #body
  #if source != none [
    #v(1mm)
    #text(font: "Inter", size: source-size, fill: faint)[#source]
  ]
]

#let data_table(columns: 1, header: (), rows: (), widths: none) = {
  let cols = if widths == none or widths.len() == 0 { range(columns).map(_ => 1fr) } else { widths }
  let body = rows.flatten()
  let cells = if header.len() > 0 {
    (table.header(..header.map(cell => text(
      font: "Inter",
      size: table-size,
      weight: "semibold",
    )[#cell])), ..body)
  } else {
    body
  }
  block(breakable: false, above: 3mm, below: 4mm)[
    #set text(size: table-size)
    #set par(leading: 0.62em, spacing: 0pt, justify: false)
    #table(
      columns: cols,
      stroke: (x, y) => if y == 0 {
        (top: 1pt + ink, bottom: 0.5pt + ink)
      } else if y == rows.len() {
        (bottom: 1pt + ink)
      } else {
        (bottom: 0.35pt + linen)
      },
      fill: (x, y) => if y == 0 {
        headerfill
      } else if calc.even(y) {
        zebra
      } else {
        none
      },
      inset: (x: 1.5mm, y: 2mm),
      ..cells,
    )
  ]
}

#let facts_table(rows) = block(breakable: false, above: 4mm, below: 5mm)[
  #set text(size: table-size)
  #table(
    columns: (34mm, 1fr),
    stroke: (x, y) => if y < rows.len() { (top: 0.5pt + linen) } else { none },
    inset: (left: 0pt, right: 3mm, top: 2mm, bottom: 2mm),
    ..rows
      .map(row => (
        text(font: "Inter", size: label-size, tracking: 0.08em, fill: brick)[#ui_upper(row.at(0))],
        row.at(1),
      ))
      .flatten(),
  )
]

// ---- boxes --------------------------------------------------------------
#let labelled_box(
  label: none,
  fill: none,
  stroke: none,
  inset: (x: 3.2mm, y: 2.2mm),
  body,
) = block(
  fill: fill,
  stroke: stroke,
  inset: inset,
  width: 100%,
  above: 3mm,
  below: 3.2mm,
  breakable: false,
)[
  #set par(spacing: 1.4mm)
  #if label != none [
    #text(
      font: "Inter",
      size: label-size,
      weight: "semibold",
      tracking: 0.12em,
      fill: brick,
    )[#upper(label)]
    #v(1mm)
  ]
  #body
]

#let definition(label: none, body) = labelled_box(
  label: label,
  fill: cream,
  stroke: (left: 2pt + brick),
  inset: (left: 3.2mm, right: 3.2mm, top: 2.4mm, bottom: 2mm),
  body,
)

#let critical(label: none, body) = labelled_box(
  label: label,
  fill: rgb("#F1EEE4"),
  stroke: (left: 2pt + sage),
  inset: (left: 3.2mm, right: 3.2mm, top: 2.4mm, bottom: 2mm),
  body,
)

#let example(label: none, body) = labelled_box(
  label: label,
  fill: none,
  stroke: (top: 0.5pt + linen, bottom: 0.5pt + linen),
  inset: (x: 0pt, y: 2mm),
  body,
)

#let note(label: none, body) = block(
  above: 2.8mm,
  below: 3mm,
  stroke: (left: 1pt + gold),
  inset: (left: 3mm, right: 0pt, top: 1.2mm, bottom: 1.2mm),
  width: 100%,
  breakable: false,
)[
  #set text(fill: muted)
  #set par(spacing: 1.4mm)
  #if label != none [
    #text(
      font: "Inter",
      size: label-size,
      weight: "semibold",
      tracking: 0.12em,
      fill: brick,
    )[#upper(label)]
    #v(1mm)
  ]
  #body
]

#let recall(label: "HATIRLA", body) = definition(label: label, body)
#let explain(label: "AÇIKLA", body) = note(label: label, body)
#let review_plan(label: "TEKRAR PLANI", body) = definition(label: label, body)

// ---- revision components ------------------------------------------------
#let answer(label: "CEVAP", body) = block(
  above: 2mm,
  below: 3mm,
  stroke: (top: 0.5pt + linen),
  inset: (top: 1.4mm),
  width: 100%,
  breakable: false,
)[
  #text(font: "Inter", size: label-size, weight: "semibold", tracking: 0.1em, fill: sage)[#upper(label)]
  #v(0.8mm)
  #set text(size: table-size, fill: muted)
  #set par(spacing: 1mm)
  #body
]

#let must_know(body) = block(breakable: false, above: 2mm, below: 3mm)[
  #set text(size: table-size)
  #set par(leading: 0.62em, spacing: 0.8mm, justify: false)
  #show list.item: it => block(
    fill: cream,
    inset: (x: 2.4mm, y: 2mm),
    width: 100%,
    above: 1.2mm,
    below: 1.2mm,
  )[#it.body]
  #body
]

#let synthesis_page(title: "TEK SAYFA SENTEZ", body) = {
  set text(size: body-size, fill: ink)
  set par(justify: false, leading: leading, spacing: par-spacing)
  block(
    width: 100%,
    breakable: false,
    stroke: 1pt + brick,
    inset: (x: 6mm, y: 5mm),
  )[
    #text(font: "Inter", size: 9pt, tracking: 0.18em, fill: brick)[#ui_upper_str("FOLISIFT · SINAV TEKRARI")]
    #v(1.4mm)
    #text(font: "Inter", size: h1-size, weight: "semibold", tracking: -0.01em)[#title]
    #v(2mm)
    #line(length: 100%, stroke: 0.5pt + linen)
    #v(2mm)
    #columns(2, gutter: 7mm)[#body]
  ]
}

#let organiser_page(title: "", source: none, body) = page(header: none)[
  #set text(size: body-size, fill: ink)
  #set par(justify: false, leading: leading, spacing: par-spacing)
  #text(font: "Inter", size: 8.5pt, tracking: 0.18em, fill: brick)[#ui_upper_str("BİLGİ ORGANİZATÖRÜ")]
  #v(1mm)
  #text(font: "Inter", size: h1-size, weight: "semibold", tracking: -0.01em)[#title]
  #v(1.4mm)
  #if source != none [
    #text(font: "Inter", size: source-size, fill: faint)[#source]
    #v(1.4mm)
  ]
  #line(length: 100%, stroke: 1pt + brick)
  #v(2mm)
  #columns(2, gutter: 6mm)[#body]
]

// ---- front matter and TOC ----------------------------------------------
#let front(kicker: "", title: "", lead: none, body) = block(
  above: 0pt,
  below: 5mm,
)[
  #text(font: "Inter", size: 8.5pt, tracking: 0.2em, fill: brick)[#ui_upper_str(kicker)]
  #v(1.4mm)
  #text(font: "Inter", size: h1-size, weight: "semibold", tracking: -0.015em)[#title]
  #v(1.4mm)
  #line(length: 100%, stroke: 1pt + brick)
  #if lead != none [
    #v(2mm)
    #text(size: lead-size, fill: muted)[#lead]
    #v(2mm)
  ]
  #body
]

#let toc_row(body, page, dest, sub: false) = grid(
  columns: (1fr, 15mm),
  stroke: (bottom: 0.35pt + linen),
  inset: (y: __TOC_ROW_INSET__),
  align(left)[
    #link(dest)[
      #text(size: if sub { toc-sub-size } else { toc-title-size }, fill: if sub { muted } else { ink })[#body]
    ]
  ],
  align(right)[
    #link(dest)[
      #text(font: "Inter", size: toc-page-size, fill: if sub { faint } else { brick })[#page]
    ]
  ],
)

#let toc_part(body) = grid(
  columns: (1fr, 15mm),
  stroke: none,
  inset: (top: 3mm, bottom: 1.2mm),
  text(font: "Inter", size: label-size, tracking: 0.14em, fill: brick)[#ui_upper(body)],
  [],
)

#let toc_page(title: "", note: "", label: "İÇİNDEKİLER") = page(header: none)[
  #set text(size: body-size, fill: ink)
  #text(font: "Inter", size: 8.5pt, tracking: 0.2em, fill: brick)[#label]
  #v(1.4mm)
  #text(font: "Inter", size: h1-size, weight: "semibold")[#title]
  #v(1.2mm)
  #text(size: table-size, fill: muted)[#note]
  #v(5mm)
  #context {
    let entries = query(<toc-entry>)
    let state = entries.fold((none, false, ()), ((last, subs, out), entry) => {
      let value = entry.value
      let page-number = counter(page).at(entry.location()).first()
      if value.level == 1 {
        let part-rows = if value.part != none and value.part != last {
          out + (toc_part(value.part),)
        } else {
          out
        }
        (
          value.part,
          value.subs,
          part-rows + (toc_row(value.body, page-number, entry.location()),),
        )
      } else if value.level == 2 and subs {
        (last, subs, out + (toc_row(value.body, page-number, entry.location(), sub: true),))
      } else {
        (last, subs, out)
      }
    })
    state.at(2).join()
  }
]

// ---- sections -----------------------------------------------------------
#let chapter(
  part: "",
  side: "",
  kicker: "",
  title: "",
  toc: none,
  lead: none,
  range: none,
  running: "",
  flow: false,
  compact: false,
  subs: true,
) = {
  if not flow {
    pagebreak(weak: true)
  }
  running_entry(running)
  toc_entry(level: 1, body: if toc == none { title } else { toc }, part: part, subs: subs)
  block(above: if flow { 5mm } else { 0pt }, below: 5mm)[
    #line(length: 100%, stroke: 1.2pt + brick)
    #v(2.4mm)
    #grid(
      columns: (1fr, auto),
      text(font: "Inter", size: 7.5pt, tracking: 0.08em, fill: rgb("#6F6454"))[#ui_upper_str(part)],
      align(right)[#text(font: "Inter", size: 7.5pt, tracking: 0.08em, fill: rgb("#6F6454"))[#ui_upper_str(side)]],
    )
    #v(1.8mm)
    #text(font: "Inter", size: 8.5pt, tracking: 0.18em, fill: brick)[#ui_upper_str(kicker)]
    #v(1.2mm)
    #show heading.where(level: 1): it => text(
      font: "Inter",
      size: h1-size,
      weight: "semibold",
      tracking: -0.02em,
      fill: ink,
      hyphenate: false,
    )[#it.body]
    #heading(level: 1, outlined: true, bookmarked: true)[#title]
    #if lead != none [
      #v(1.8mm)
      #block(width: 100%, text(size: lead-size, fill: muted)[#lead])
    ]
    #if range != none [
      #v(1.8mm)
      #text(font: "Inter", size: source-size, tracking: 0.04em, fill: soft)[#range]
    ]
  ]
}

#let two_col(body) = block[
  #set text(size: table-size)
  #set par(justify: false, leading: 0.66em, spacing: 2mm)
  #columns(2, gutter: 8mm)[#body]
]

// ---- document -----------------------------------------------------------
#let revision_sheet(
  title: "",
  subtitle: "",
  course: "",
  language: "tr",
  sources: (),
  running: "",
  pilot: none,
  deck: none,
  cover_kicker: none,
  cover_title: none,
  cover_motif: none,
  cover_notes: (),
  body,
) = {
  set document(title: title, author: "Folisift")
  sheet_title.update(running)
  doc_lang.update(language)
  set page(
    paper: "a4",
    margin: (top: __MARGIN_TOP__, bottom: __MARGIN_BOTTOM__, left: __MARGIN_SIDE__, right: __MARGIN_SIDE__),
    fill: paper_bg,
    header: context {
      let current = here().page()
      let heads = query(<running-head>).filter(head => head.location().page() <= current)
      let left = if heads.len() > 0 { heads.last().value.text } else { "" }
      pad(top: 5mm, running_header(left, sheet_title.get()))
    },
    footer: context {
      set text(font: "Inter", size: 8pt, weight: "medium", fill: brick)
      align(center)[#counter(page).display("1")]
    },
  )
  set text(font: "Inter", size: body-size, fill: ink, lang: language, hyphenate: false)
  set par(justify: false, leading: leading, spacing: par-spacing, linebreaks: "optimized")
  set list(
    marker: ([#text(fill: brick)[—]], [•]),
    indent: 5mm,
    spacing: list-spacing,
  )
  set raw(tab-size: 2)
  show raw.where(block: true): it => block(
    fill: sand,
    stroke: (left: 1.4pt + gold),
    inset: (left: 2.6mm, right: 2.6mm, top: 1.8mm, bottom: 1.8mm),
    width: 100%,
    breakable: false,
  )[
    #set par(leading: 0.55em)
    #text(font: "Noto Sans Mono", size: code-size)[#it]
  ]
  show raw.where(block: false): it => box(
    fill: sand,
    inset: (x: 0.7mm),
    text(font: "Noto Sans Mono", size: code-size)[#it],
  )
  set figure(gap: 2mm)
  show figure: it => block(above: 3mm, below: 3mm, it)
  show figure.caption: it => block(
    above: 1.2mm,
    stroke: (top: 0.35pt + linen),
    inset: (top: 1.2mm),
  )[
    #set text(size: caption-size, style: "italic", fill: muted)
    #if it.numbering != none [
      #it.supplement
      #context it.counter.display(it.numbering)
      #h(1.5mm)
    ]
    #it.body
  ]
  show heading.where(level: 2): it => {
    toc_entry(level: 2, body: it.body)
    block(above: 4.5mm, below: 1.8mm, sticky: true)[
      #text(font: "Inter", size: h2-size, weight: "semibold", tracking: -0.005em, fill: ink, hyphenate: false)[#it.body]
      #v(1mm)
      #line(length: 100%, stroke: 0.5pt + linen)
    ]
  }
  show heading.where(level: 3): it => block(above: 3.4mm, below: 1.4mm, sticky: true)[
    #text(font: "Inter", size: h3-size, weight: "semibold", tracking: 0.06em, fill: sage)[#ui_upper(it.body)]
  ]
  body
}

#let organiser(
  title: "",
  subtitle: "",
  course: "",
  language: "tr",
  sources: (),
  running: "",
  pilot: none,
  deck: none,
  cover_kicker: none,
  cover_title: none,
  cover_motif: none,
  cover_notes: (),
  body,
) = {
  set document(title: title, author: "Folisift")
  sheet_title.update(running)
  doc_lang.update(language)
  set page(
    paper: "a4",
    margin: (top: __MARGIN_TOP__, bottom: __MARGIN_BOTTOM__, left: __MARGIN_SIDE__, right: __MARGIN_SIDE__),
    fill: paper_bg,
    header: none,
    footer: context {
      set text(font: "Inter", size: 8pt, weight: "medium", fill: brick)
      align(center)[#counter(page).display("1")]
    },
  )
  set text(font: "Inter", size: body-size, fill: ink, lang: language, hyphenate: false)
  set par(justify: false, leading: leading, spacing: par-spacing, linebreaks: "optimized")
  set list(
    marker: ([#text(fill: brick)[—]], [•]),
    indent: 4mm,
    spacing: list-spacing,
  )
  show heading.where(level: 2): it => block(above: 2.4mm, below: 1mm, sticky: true)[
    #text(font: "Inter", size: h3-size, weight: "semibold", tracking: 0.06em, fill: sage)[#ui_upper(it.body)]
  ]
  show heading.where(level: 3): it => block(above: 1.8mm, below: 0.8mm)[
    #text(font: "Inter", size: label-size, weight: "semibold", tracking: 0.06em, fill: brick)[#ui_upper(it.body)]
  ]
  body
}
