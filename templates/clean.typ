// Folisift paper template. Guide content is generated from Markdown and must
// not inject raw Typst. Keep visual decisions in this file.

// ---- tokens -------------------------------------------------------------
#let ink = rgb("#111827")
#let muted = rgb("#4B5563")
#let soft = rgb("#374151")
#let faint = rgb("#6B7280")
#let brick = rgb("#1D4ED8")
#let sage = rgb("#0F766E")
#let gold = rgb("#B45309")
#let paper_bg = rgb("#FFFFFF")
#let linen = rgb("#D1D5DB")
#let cream = rgb("#EFF6FF")
#let sand = rgb("#F3F4F6")
#let zebra = rgb("#F9FAFB")
#let headerfill = rgb("#E5E7EB")

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
  size: 6.9pt,
  style: "normal",
  fill: faint,
  tracking: 0.02em,
)[[#body]])

#let fig_label(number, word: "Şekil") = text(
  font: "Inter",
  size: 6.8pt,
  tracking: 0.12em,
  fill: brick,
)[#upper(word + " " + number)]

#let toc_entry(level: 1, body: none, part: none, subs: true) = [
  #metadata((
    level: level,
    body: body,
    part: part,
    subs: subs,
  )) <toc-entry>
]

#let sheet_title = state("sheet-title", "")

#let running_entry(text) = [
  #metadata((text: text)) <running-head>
]

#let running_header(left_text, right_text) = {
  set text(font: "Inter", size: 7pt, fill: rgb("#4B5563"), tracking: 0.14em)
  grid(
    columns: (1fr, auto),
    align(start)[#ui_upper_str(left_text)],
    align(end)[#text(tracking: 0.06em, fill: faint)[#right_text]],
  )
}

#let table_block(caption: none, source: none, label: "Tablo", body) = block(
  breakable: false,
  above: 3mm,
  below: 4mm,
)[
  #if caption != none [
    #context {
      counter("table").step()
      block(above: 0pt, below: 1.5mm)[
        #text(font: "Inter", size: 7.8pt, weight: "semibold", fill: ink)[#label #counter("table").display()]
        #text(font: "Inter", size: 7.8pt, fill: ink)[ #caption]
      ]
    }
  ]
  #body
  #if source != none [
    #v(1mm)
    #text(font: "Inter", size: 6.9pt, fill: faint)[#source]
  ]
]

// ---- tables -------------------------------------------------------------
#let data_table(columns: 1, header: (), rows: (), widths: none) = {
  let cols = if widths == none or widths.len() == 0 { columns } else { widths }
  let body = rows.flatten()
  let cells = if header.len() > 0 {
    (table.header(..header.map(cell => text(
      font: "Inter",
      size: 6.9pt,
      weight: "semibold",
      tracking: 0.09em,
    )[#ui_upper(cell)])), ..body)
  } else {
    body
  }
  block(breakable: false, above: 3mm, below: 4mm)[
    #set text(size: 7.9pt)
    #set par(leading: 0.58em, spacing: 0pt, justify: false)
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
      inset: (x: 1.5mm, y: 2.2mm),
      ..cells,
    )
  ]
}

#let facts_table(rows) = block(breakable: false, above: 4mm, below: 5mm)[
  #set text(size: 9.4pt)
  #table(
    columns: (34mm, 1fr),
    stroke: (x, y) => if y < rows.len() { (top: 0.5pt + linen) } else { none },
    inset: (left: 0pt, right: 3mm, top: 2.2mm, bottom: 2.2mm),
    ..rows
      .map(row => (
        text(
          font: "Inter",
          size: 7.4pt,
          tracking: 0.1em,
          fill: brick,
        )[#ui_upper(row.at(0))],
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
  #set par(spacing: 1.5mm)
  #if label != none [
    #text(
      font: "Inter",
      size: 6.9pt,
      weight: "semibold",
      tracking: 0.16em,
      fill: brick,
    )[#upper(label)]
    #v(1.2mm)
  ]
  #body
]

#let definition(label: none, body) = labelled_box(
  label: label,
  fill: cream,
  stroke: (left: 2pt + brick),
  inset: (left: 3.2mm, right: 3.2mm, top: 2.5mm, bottom: 2mm),
  body,
)

#let critical(label: none, body) = labelled_box(
  label: label,
  fill: rgb("#F0FDFA"),
  stroke: (left: 2pt + sage),
  inset: (left: 3.2mm, right: 3.2mm, top: 2.5mm, bottom: 2mm),
  body,
)

#let example(label: none, body) = labelled_box(
  label: label,
  fill: none,
  stroke: (top: 0.5pt + linen, bottom: 0.5pt + linen),
  inset: (x: 0pt, y: 2.2mm),
  body,
)

#let note(label: none, body) = block(
  above: 3mm,
  below: 3.2mm,
  stroke: (left: 1pt + gold),
  inset: (left: 3.2mm, right: 0pt, top: 1.2mm, bottom: 1.2mm),
  width: 100%,
  breakable: false,
)[
  #set text(size: 8.9pt, fill: muted)
  #set par(spacing: 1.5mm)
  #if label != none [
    #text(
      font: "Inter",
      size: 6.9pt,
      weight: "semibold",
      tracking: 0.16em,
      fill: brick,
    )[#upper(label)]
    #v(1.2mm)
  ]
  #body
]

// ---- front matter and TOC ----------------------------------------------
#let front(kicker: "", title: "", lead: none, body) = page(header: none)[
  #set text(font: "Inter", size: 9.5pt, fill: ink)
  #set par(justify: true, leading: 0.67em, spacing: 1.8mm, linebreaks: "optimized")
  #text(
    font: "Inter",
    size: 7.2pt,
    tracking: 0.2em,
    fill: brick,
  )[#ui_upper_str(kicker)]
  #v(2mm)
  #text(font: "Inter", size: 19pt, weight: "semibold", tracking: -0.01em)[#title]
  #v(3mm)
  #if lead != none [
    #text(size: 11.2pt, fill: rgb("#1F2937"))[#lead]
    #v(5mm)
  ]
  #body
]

#let toc_row(body, page, sub: false) = grid(
  columns: (1fr, 15mm),
  stroke: (bottom: 0.35pt + linen),
  inset: (y: 1.9mm),
  align(left)[#text(size: if sub { 8.9pt } else { 9.6pt }, fill: if sub { muted } else { ink })[#body]],
  align(right)[#text(font: "Inter", size: 8.2pt, fill: if sub { faint } else { brick })[#page]],
)

#let toc_part(body) = grid(
  columns: (1fr, 15mm),
  stroke: none,
  inset: (top: 4mm, bottom: 1.9mm),
  text(font: "Inter", size: 7.2pt, tracking: 0.16em, fill: brick)[#ui_upper(body)],
  [],
)

#let toc_page(title: "", note: "", label: "İÇİNDEKİLER") = page(header: none)[
  #set text(font: "Inter", size: 9.5pt, fill: ink)
  #text(font: "Inter", size: 7.2pt, tracking: 0.2em, fill: brick)[#label]
  #v(2mm)
  #text(font: "Inter", size: 19pt, weight: "semibold")[#title]
  #v(1.8mm)
  #text(size: 8.6pt, fill: muted)[#note]
  #v(7mm)
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
          part-rows + (toc_row(value.body, page-number),),
        )
      } else if value.level == 2 and subs {
        (last, subs, out + (toc_row(value.body, page-number, sub: true),))
      } else {
        (last, subs, out)
      }
    })
    state.at(2).join()
  }
]

// ---- chapters -----------------------------------------------------------
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
  block(
    above: if flow { 9mm } else if compact { 3mm } else { 0pt },
    below: 8mm,
  )[
    #line(length: 100%, stroke: 1.4pt + brick)
    #v(4mm)
    #grid(
      columns: (1fr, auto),
      text(font: "Inter", size: 7pt, tracking: 0.08em, fill: rgb("#4B5563"))[#ui_upper_str(part)],
      align(right)[#text(font: "Inter", size: 7pt, tracking: 0.08em, fill: rgb("#4B5563"))[#ui_upper_str(side)]],
    )
    #v(3mm)
    #text(font: "Inter", size: 8pt, tracking: 0.22em, fill: brick)[#ui_upper_str(kicker)]
    #v(2mm)
    #show heading.where(level: 1): it => text(
      font: "Inter",
      size: if flow or compact { 19pt } else { 21pt },
      weight: "semibold",
      tracking: -0.02em,
      fill: ink,
      hyphenate: false,
    )[#it.body]
    #heading(level: 1, outlined: true, bookmarked: true)[#title]
    #if lead != none [
      #v(2.5mm)
      #block(width: 150mm, text(size: 10.8pt, fill: rgb("#1F2937"))[#lead])
    ]
    #if range != none [
      #v(3mm)
      #text(font: "Inter", size: 7.4pt, tracking: 0.04em, fill: soft)[#range]
    ]
  ]
}

#let two_col(body) = block[
  #set text(size: 8.25pt)
  #set par(justify: true, leading: 0.60em, spacing: 2.6mm, linebreaks: "optimized")
  #show heading.where(level: 3): it => block(above: 2.4mm, below: 0.9mm)[
    #text(
      font: "Inter",
      size: 7.4pt,
      weight: "semibold",
      tracking: 0.12em,
      fill: sage,
    )[#ui_upper(it.body)]
  ]
  #place(line(
    start: (50%, 0pt),
    end: (50%, 100%),
    stroke: 0.35pt + linen,
  ))
  #columns(2, gutter: 10mm)[#body]
]

// ---- document -----------------------------------------------------------
#let paper(
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
    margin: (top: 16mm, bottom: 14mm, left: 16.5mm, right: 16.5mm),
    fill: paper_bg,
    header: context {
      let current = here().page()
      let heads = query(<running-head>).filter(head => head.location().page() <= current)
      let left = if heads.len() > 0 { heads.last().value.text } else { "" }
      pad(top: 6mm, running_header(left, sheet_title.get()))
    },
    footer: context {
      set text(font: "Inter", size: 8pt, weight: "medium", fill: brick)
      align(center)[#counter(page).display("1")]
    },
  )
  set text(
    font: "Inter",
    size: 9.5pt,
    fill: ink,
    lang: language,
    hyphenate: false,
  )
  set par(justify: true, leading: 0.67em, spacing: 1.8mm, linebreaks: "optimized")
  set list(
    marker: ([#text(fill: brick)[—]], [•]),
    indent: 5mm,
    spacing: 3.2mm,
  )
  set raw(tab-size: 2)
  show raw.where(block: true): it => block(
    fill: sand,
    stroke: (left: 1.4pt + gold),
    inset: (left: 2.6mm, right: 2.6mm, top: 2mm, bottom: 2mm),
    width: 100%,
    breakable: false,
  )[
    #set par(leading: 0.5em)
    #text(font: "Noto Sans Mono", size: 7.4pt)[#it]
  ]
  show raw.where(block: false): it => box(
    fill: sand,
    inset: (x: 0.7mm),
    text(font: "Noto Sans Mono", size: 8.4pt)[#it],
  )
  set figure(gap: 2mm)
  show figure: it => block(above: 3.6mm, below: 3.8mm, it)
  show figure.caption: it => block(
    above: 1.4mm,
    stroke: (top: 0.35pt + linen),
    inset: (top: 1.4mm),
  )[
    #set text(size: 7.9pt, style: "italic", fill: muted)
    #it.body
  ]

  show heading.where(level: 2): it => {
    toc_entry(level: 2, body: it.body)
    let raw = it.body.text
    let head = if raw == none {
      it.body
    } else {
      let parts = raw.split(" ")
      let number = parts.at(0)
      let rest = parts.slice(1).join(" ")
      if number.contains(".") and rest.len() > 0 {
        [
          #text(fill: brick, weight: "medium")[#number]
          #h(2.2mm)
          #text[#rest]
        ]
      } else {
        it.body
      }
    }
    block(above: 7.7mm, below: 2.2mm, sticky: true)[
      #text(
        font: "Inter",
        size: 12pt,
        weight: "semibold",
        tracking: -0.005em,
        fill: ink,
        hyphenate: false,
      )[#head]
      #v(1.4mm)
      #line(length: 100%, stroke: 0.5pt + linen)
    ]
  }
  show heading.where(level: 3): it => block(above: 5.4mm, below: 4.2mm, sticky: true)[
    #text(
      font: "Inter",
      size: 8.2pt,
      weight: "semibold",
      tracking: 0.12em,
      fill: sage,
    )[#ui_upper(it.body)]
  ]

  // Cover ---------------------------------------------------------------
  page(margin: 0pt, header: none, footer: none, fill: rgb("#FFFFFF"))[
    #set text(font: "Inter", size: 9.5pt, fill: ink, lang: language)
    #pad(left: 20mm, right: 20mm, top: 18mm, bottom: 15mm)[
      #rect(width: 100%, height: 264mm, stroke: 0.6pt + linen)[
        #place(center + top, dy: 16mm)[
          #block(width: 138mm)[
            #align(center)[
              #text(
                font: "Inter",
                size: 8pt,
                tracking: 0.22em,
                fill: brick,
              )[#ui_upper_str(if cover_kicker == none { course } else { cover_kicker })]
              #v(5mm)
              #line(length: 28mm, stroke: 0.8pt + brick)
              #v(28mm)
              #text(
                font: "Inter",
                size: 33pt,
                weight: "regular",
                tracking: -0.015em,
                hyphenate: false,
              )[#if cover_title == none { title } else { cover_title }]
              #v(5mm)
              #text(font: "Inter", size: 15pt, style: "italic", fill: muted)[#subtitle]
              #if deck != none [
                #v(9mm)
                #text(font: "Inter", size: 9.5pt, tracking: 0.04em, fill: muted)[#deck]
              ]
              #if pilot != none [
                #v(7mm)
                #box(
                  stroke: 0.6pt + brick,
                  inset: (x: 4mm, y: 1.6mm),
                  text(
                    font: "Inter",
                    size: 7pt,
                    tracking: 0.18em,
                    fill: brick,
                  )[#ui_upper_str(pilot)],
                )
              ]
            ]
          ]
        ]
        #if cover_motif != none [
          #place(center + bottom, dy: -42mm, image(cover_motif, width: 95mm))
        ]
        #place(center + bottom, dy: -14mm)[
          #align(center)[
            #set par(leading: 0.7em)
            #for (index, item) in cover_notes.enumerate() {
              text(
                font: "Inter",
                size: if index == 0 { 7.4pt } else { 6.6pt },
                tracking: 0.05em,
                fill: if index == 0 { soft } else { faint },
              )[#item]
              linebreak()
            }
          ]
        ]
      ]
    ]
  ]

  body
}
