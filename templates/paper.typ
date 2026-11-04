// Folisift paper template. Guide content is generated from Markdown and must
// not inject raw Typst. Keep visual decisions in this file.

// ---- tokens -------------------------------------------------------------
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
  size: __LABEL_SIZE__,
  style: "normal",
  fill: faint,
  tracking: 0.02em,
)[[#body]])

#let fitted_image(path, alt: none, width: 90%, max_height: 14cm, fit: "contain") = layout(size => {
  let target = size.width * width
  let wide = image(path, width: target, alt: alt)
  // Scale the cap with the requested width so width=60% and width=90% are
  // visibly different even when both would hit the page-height guard.
  let cap = if fit == "width" { 1000cm } else { max_height * (width / 90%) }
  block(width: size.width)[
    #align(center, if fit != "height" and measure(wide).height <= cap {
      wide
    } else {
      image(path, height: cap, alt: alt)
    })
  ]
})

#let fig_label(number, word: "Şekil") = text(
  font: "Inter",
  size: __FIG_LABEL_SIZE__,
  tracking: 0.12em,
  fill: brick,
)[#upper(word + " " + number)]

#let toc_entry(level: 1, body: none, part: none, subs: true, key: "") = [
  #metadata((
    level: level,
    body: body,
    part: part,
    subs: subs,
    key: key,
  )) <toc-entry>
]

#let sheet_title = state("sheet-title", "")

#let running_entry(text) = [
  #metadata((text: text)) <running-head>
]

#let running_header(left_text, right_text) = {
  set text(font: "Inter", size: __RUNNING_SIZE__, fill: rgb("#7A6F5E"), tracking: 0.14em)
  grid(
    columns: (1fr, auto),
    align(start)[#ui_upper_str(left_text)],
    align(end)[#text(tracking: 0.06em, fill: faint)[#right_text]],
  )
}

#let table_block(caption: none, source: none, label: "Tablo", number: true, keep_with_next: false, allow_page_break: false, min_space_mm: none, body) = block(
  breakable: allow_page_break,
  sticky: keep_with_next,
  above: if min_space_mm == none { 3mm } else { min_space_mm * 1mm },
  below: 4mm,
)[
  #if caption != none [
    #context {
      counter("table").step()
      block(above: 0pt, below: 1.5mm)[
        #if number [
          #text(font: "Inter", size: __CAPTION_SIZE__, weight: "semibold", fill: ink)[#label #counter("table").display()]
        ]
        #text(font: "Inter", size: __CAPTION_SIZE__, fill: ink)[ #caption]
      ]
    }
  ]
  #body
  #if source != none [
    #v(1mm)
    #text(font: "Inter", size: __LABEL_SIZE__, fill: faint)[#source]
  ]
]

// ---- tables -------------------------------------------------------------
#let data_table(columns: 1, header: (), rows: (), widths: none, compact: false) = {
  let cols = if widths == none or widths.len() == 0 { columns } else { widths }
  let body = rows.flatten()
  let cells = if header.len() > 0 {
    (table.header(..header.map(cell => text(
      font: "Inter",
      size: __LABEL_SIZE__,
      weight: "semibold",
      tracking: 0.09em,
    )[#ui_upper(cell)])), ..body)
  } else {
    body
  }
  block(breakable: compact, above: if compact { 2mm } else { 3mm }, below: if compact { 2.5mm } else { 4mm })[
    #set text(size: if compact { __TABLE_SIZE__ * 0.84 } else { __TABLE_SIZE__ })
    #set par(leading: if compact { 0.40em } else { 0.55em }, spacing: 0pt, justify: false)
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
      inset: if compact { (x: 1.5mm, y: 0.75mm) } else { (x: 1.5mm, y: 2.2mm) },
      ..cells,
    )
  ]
}

#let glossary_group(body) = block(above: 2.5mm, below: 1.2mm, sticky: true)[
  #text(
    font: "Inter",
    size: __SMALL_SIZE__,
    weight: "semibold",
    tracking: 0.12em,
    fill: sage,
  )[#ui_upper(body)]
]

#let facts_table(rows) = block(breakable: false, above: 4mm, below: 5mm)[
  #set text(size: __BOX_SIZE__)
  #table(
    columns: (34mm, 1fr),
    stroke: (x, y) => if y < rows.len() { (top: 0.5pt + linen) } else { none },
    inset: (left: 0pt, right: 3mm, top: 2.2mm, bottom: 2.2mm),
    ..rows
      .map(row => (
        text(
          font: "Inter",
          size: __SMALL_SIZE__,
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
      size: __LABEL_SIZE__,
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
  fill: rgb("#F1EEE4"),
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
  #set text(size: __NOTE_SIZE__, fill: muted)
  #set par(spacing: 1.5mm)
  #if label != none [
    #text(
      font: "Inter",
      size: __LABEL_SIZE__,
      weight: "semibold",
      tracking: 0.16em,
      fill: brick,
    )[#upper(label)]
    #v(1.2mm)
  ]
  #body
]

// ---- evidence-based learning components --------------------------------
#let recall(label: "HATIRLA", body) = definition(label: label, body)
#let explain(label: "AÇIKLA", body) = note(label: label, body)
#let answer(label: "CEVAP", body) = labelled_box(
  label: label,
  fill: rgb("#F1EEE4"),
  stroke: (left: 1pt + sage),
  inset: (left: 3.2mm, right: 3.2mm, top: 2.5mm, bottom: 2mm),
  body,
)
#let review_plan(label: "TEKRAR PLANI", body) = definition(label: label, body)

// ---- front matter and TOC ----------------------------------------------
#let front(kicker: "", title: "", lead: none, body) = page(header: none)[
  #set text(font: "Noto Serif", size: __BODY_SIZE__, fill: ink)
  #set par(justify: true, leading: 0.62em, spacing: 1.8mm, linebreaks: "optimized")
  #text(
    font: "Inter",
    size: __MICRO_SIZE__,
    tracking: 0.2em,
    fill: brick,
  )[#ui_upper_str(kicker)]
  #v(2mm)
  #text(font: "Inter", size: __H1_SIZE__, weight: "semibold", tracking: -0.01em)[#title]
  #v(3mm)
  #if lead != none [
    #text(size: __LEAD_LARGE_SIZE__, fill: rgb("#3B352D"))[#lead]
    #v(5mm)
  ]
  #body
]

#let toc_row(body, page, dest, sub: false) = grid(
  columns: (1fr, 15mm),
  stroke: (bottom: 0.35pt + linen),
  inset: (y: 1.9mm),
  align(left)[
    #link(dest)[
      #text(size: if sub { 8.9pt } else { 9.6pt }, fill: if sub { muted } else { ink })[#body]
    ]
  ],
  align(right)[
    #link(dest)[
      #text(font: "Inter", size: 8.2pt, fill: if sub { faint } else { brick })[#page]
    ]
  ],
)

#let toc_part(body) = grid(
  columns: (1fr, 15mm),
  stroke: none,
  inset: (top: 4mm, bottom: 1.9mm),
  text(font: "Inter", size: __MICRO_SIZE__, tracking: 0.16em, fill: brick)[#ui_upper(body)],
  [],
)

#let toc_page(title: "", note: "", label: "İÇİNDEKİLER", subs: true, depth: 2, toc_include: (), toc_exclude: ()) = page(header: none)[
  #set text(font: "Noto Serif", size: __BODY_SIZE__, fill: ink)
  #text(font: "Inter", size: __MICRO_SIZE__, tracking: 0.2em, fill: brick)[#label]
  #v(2mm)
  #text(font: "Inter", size: __H1_SIZE__, weight: "semibold")[#title]
  #v(1.8mm)
  #text(size: __NOTE_SMALL_SIZE__, fill: muted)[#note]
  #v(7mm)
  #context {
    let entries = query(<toc-entry>)
    let state = entries.fold((none, false, ()), ((last, subs, out), entry) => {
      let value = entry.value
      let page-number = counter(page).at(entry.location()).first()
      let text = value.key
      let included = if toc_include.len() == 0 { true } else { toc_include.any(term => text.contains(term)) }
      let excluded = if toc_exclude.len() == 0 { false } else { toc_exclude.any(term => text.contains(term)) }
      if not included {
        if value.level == 1 { (value.part, value.subs, out) } else { (last, subs, out) }
      } else if excluded {
        if value.level == 1 { (value.part, value.subs, out) } else { (last, subs, out) }
      } else if value.level == 1 {
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
      } else if value.level == 2 and subs and depth >= 2 {
        (last, subs, out + (toc_row(value.body, page-number, entry.location(), sub: true),))
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
  keep_with_next: true,
  break_before: false,
  break_after: false,
  allow_split: false,
  full_page_opener: false,
) = {
  if not flow or break_before or full_page_opener {
    pagebreak(weak: not (break_before or full_page_opener))
  }
  running_entry(running)
  toc_entry(level: 1, body: if toc == none { title } else { toc }, part: part, subs: subs, key: repr(if toc == none { title } else { toc }))
  block(
    above: if flow { 9mm } else if compact { 3mm } else { 0pt },
    below: 8mm,
    breakable: allow_split,
    sticky: keep_with_next and not full_page_opener,
  )[
    #line(length: 100%, stroke: 1.4pt + brick)
    #v(4mm)
    #grid(
      columns: (1fr, auto),
      text(font: "Inter", size: __RUNNING_SIZE__, tracking: 0.08em, fill: rgb("#7A6F5E"))[#ui_upper_str(part)],
      align(right)[#text(font: "Inter", size: __RUNNING_SIZE__, tracking: 0.08em, fill: rgb("#7A6F5E"))[#ui_upper_str(side)]],
    )
    #v(3mm)
    #text(font: "Inter", size: __UI_SIZE__, tracking: 0.22em, fill: brick)[#ui_upper_str(kicker)]
    #v(2mm)
    #show heading.where(level: 1): it => text(
      font: "Inter",
      size: if flow or compact { __H1_SIZE__ } else { __H1_LARGE_SIZE__ },
      weight: "semibold",
      tracking: -0.02em,
      fill: ink,
      hyphenate: false,
    )[#it.body]
    #heading(level: 1, outlined: true, bookmarked: true)[#title]
    #if lead != none [
      #v(2.5mm)
      #block(width: 150mm, text(size: __LEAD_SIZE__, fill: rgb("#3B352D"))[#lead])
    ]
    #if range != none [
      #v(3mm)
      #text(font: "Inter", size: __SMALL_SIZE__, tracking: 0.04em, fill: soft)[#range]
    ]
  ]
  if break_after {
    pagebreak(weak: false)
  }
}

#let two_col(flow: false, keep_with_next: false, min_rows_before_break: 0, body) = block(
  breakable: flow,
  sticky: keep_with_next,
)[
  #set text(size: __TWO_COL_SIZE__)
  #set par(justify: true, leading: 0.56em, spacing: 3.6mm, linebreaks: "optimized")
  #show block: set block(above: 2.8mm, below: 0pt)
  #show heading.where(level: 3): it => block(above: 4.5mm, below: 5mm)[
    #text(
      font: "Inter",
      size: __SMALL_SIZE__,
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
      set text(font: "Inter", size: __UI_SIZE__, weight: "medium", fill: faint)
      align(center)[
        #text[Folisift] #h(3mm) #counter(page).display("1")
      ]
    },
  )
  set text(
    font: "Noto Serif",
    size: __BODY_SIZE__,
    fill: ink,
    lang: language,
    hyphenate: false,
  )
  set par(justify: true, leading: 0.62em, spacing: 1.8mm, linebreaks: "optimized")
  set list(
    marker: ([#text(fill: brick)[—]], [•]),
    indent: 5mm,
    spacing: 3.2mm,
  )
  set raw(tab-size: 2, theme: none)
  show raw.where(block: true): it => block(
    fill: sand,
    stroke: (left: 1.4pt + gold),
    inset: (left: 2.6mm, right: 2.6mm, top: 2mm, bottom: 2mm),
    width: 100%,
    breakable: false,
  )[
    #set par(leading: 0.5em)
    #text(font: "Noto Sans Mono", size: __CODE_BLOCK_SIZE__)[#it]
  ]
  show raw.where(block: false): it => box(
    fill: sand,
    inset: (x: 0.7mm),
    text(font: "Noto Sans Mono", size: __CODE_INLINE_SIZE__)[#it],
  )
  set figure(gap: 2mm)
  show figure: it => block(above: 3.6mm, below: 3.8mm, it)
  show figure.caption: it => block(
    above: 1.4mm,
    stroke: (top: 0.35pt + linen),
    inset: (top: 1.4mm),
  )[
    #set text(size: __TABLE_SIZE__, style: "italic", fill: muted)
    #if it.numbering != none [
      #it.supplement
      #context it.counter.display(it.numbering)
      #h(1.5mm)
    ]
    #it.body
  ]

  show heading.where(level: 2): it => {
    let raw = it.body.text
    toc_entry(level: 2, body: it.body, key: repr(it.body))
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
        size: __H2_SIZE__,
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
      size: __H3_SIZE__,
      weight: "semibold",
      tracking: 0.12em,
      fill: sage,
    )[#ui_upper(it.body)]
  ]

  // Cover ---------------------------------------------------------------
  page(margin: 0pt, header: none, footer: none, fill: rgb("#F3EBDD"))[
    #set text(font: "Noto Serif", size: __BODY_SIZE__, fill: ink, lang: language)
    #pad(left: 20mm, right: 20mm, top: 18mm, bottom: 15mm)[
      #rect(width: 100%, height: 264mm, stroke: 0.8pt + gold)[
        #place(center, rect(
          width: 100% - 8mm,
          height: 100% - 8mm,
          stroke: 0.4pt + rgb("#C9B99E"),
        ))
        #place(center + top, dy: 16mm)[
          #block(width: 138mm)[
            #align(center)[
              #text(
                font: "Inter",
                size: __UI_SIZE__,
                tracking: 0.22em,
                fill: brick,
              )[#ui_upper_str(if cover_kicker == none { course } else { cover_kicker })]
              #v(5mm)
              #line(length: 28mm, stroke: 0.8pt + brick)
              #v(28mm)
              #set par(justify: false)
              #text(
                font: "Noto Serif",
                size: __COVER_TITLE_SIZE__,
                weight: "regular",
                tracking: -0.015em,
                hyphenate: false,
              )[#if cover_title == none { title } else { cover_title }]
              #v(5mm)
              #text(font: "Noto Serif", size: __COVER_SUBTITLE_SIZE__, style: "italic", fill: muted)[#subtitle]
              #if deck != none [
                #v(9mm)
                #text(font: "Inter", size: __BODY_SIZE__, tracking: 0.04em, fill: muted)[#deck]
              ]
              #if pilot != none [
                #v(7mm)
                #box(
                  stroke: 0.6pt + brick,
                  inset: (x: 4mm, y: 1.6mm),
                  text(
                    font: "Inter",
                    size: __RUNNING_SIZE__,
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
                size: if index == 0 { __SMALL_SIZE__ } else { __TINY_SIZE__ },
                tracking: 0.05em,
                fill: if index == 0 { soft } else { faint },
              )[#item]
              linebreak()
            }
            #v(2mm)
            #link("https://github.com/Aylmz0/folisift")[
              #text(
                font: "Inter",
                size: __SMALL_SIZE__,
                tracking: 0.05em,
                fill: faint,
              )[Folisift · github.com/Aylmz0/folisift]
            ]
          ]
        ]
      ]
    ]
  ]

  body
}
