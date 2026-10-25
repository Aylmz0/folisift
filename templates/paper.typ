// Folisift paper template. Guide content is generated from Markdown and must
// not inject raw Typst. Keep visual decisions in this file.

#let paper(
  title: "",
  subtitle: "",
  course: "",
  language: "tr",
  sources: (),
  body,
) = {
  set document(title: title, author: "Folisift")
  set page(
    paper: "a4",
    margin: (x: 22mm, y: 20mm),
    fill: rgb("#FAF6EC"),
    header: context {
      set text(font: "Inter", size: 7.5pt, fill: rgb("#5E574B"))
      [Folisift]
      h(1fr)
      [#title]
    },
    footer: context {
      set text(font: "Inter", size: 8pt, fill: rgb("#5E574B"))
      h(1fr)
      counter(page).display("1")
    },
  )
  set text(
    font: "Noto Serif",
    size: 10.2pt,
    fill: rgb("#211E19"),
    lang: language,
    hyphenate: true,
  )
  set par(justify: true, leading: 0.62em, spacing: 0.95em)
  set raw(tab-size: 2)
  show raw: set text(font: "Noto Sans Mono", size: 8.5pt)
  set table(
    stroke: (x: none, y: 0.5pt + rgb("#D8CEBD")),
    inset: (x: 5pt, y: 4pt),
  )
  show table: set text(size: 9pt)
  show raw.where(block: true): it => block(
    fill: rgb("#F1EADB"),
    inset: 8pt,
    width: 100%,
    it,
  )
  show figure.caption: set text(
    size: 8.5pt,
    fill: rgb("#5E574B"),
    style: "italic",
  )
  set figure(gap: 8pt)

  show heading.where(level: 1): it => {
    pagebreak(weak: true)
    block(above: 0pt, below: 1.1em)[
      #text(
        font: "Inter",
        size: 18pt,
        weight: "bold",
        fill: rgb("#8A3E2F"),
        hyphenate: false,
      )[#it.body]
    ]
  }
  show heading.where(level: 2): it => block(above: 1.2em, below: 0.5em)[
    #text(
      font: "Inter",
      size: 13pt,
      weight: "semibold",
      fill: rgb("#8A3E2F"),
      hyphenate: false,
    )[#it.body]
  ]
  show heading.where(level: 3): it => block(above: 0.9em, below: 0.35em)[
    #text(
      font: "Inter",
      size: 10.5pt,
      weight: "semibold",
      fill: rgb("#211E19"),
      hyphenate: false,
    )[#it.body]
  ]

  // Cover -----------------------------------------------------------------
  page(header: none, footer: none, margin: (x: 26mm, y: 24mm))[
    #set par(justify: false)
    #rect(
      width: 100%,
      height: 100%,
      stroke: 0.6pt + rgb("#D8CEBD"),
      inset: 14mm,
    )[
      #align(center)[
        #v(12mm)
        #text(font: "Inter", size: 8.5pt, tracking: 0.08em, fill: rgb("#5E574B"))[
          #course
        ]
        #v(7mm)
        #line(length: 32%, stroke: 0.6pt + rgb("#B08A4F"))
        #v(8mm)
        #text(
          font: "Inter",
          size: 26pt,
          weight: "bold",
          fill: rgb("#8A3E2F"),
          hyphenate: false,
        )[#title]
        #v(5mm)
        #text(font: "Noto Serif", size: 12.5pt, fill: rgb("#5E574B"))[
          #subtitle
        ]
        #v(18mm)
        #if sources.len() > 0 [
          #text(font: "Inter", size: 8.5pt, fill: rgb("#5E574B"))[
            Kaynaklar: #sources.join(" · ")
          ]
        ]
        #v(1fr)
        #text(font: "Inter", size: 8pt, fill: rgb("#5E574B"))[
          Folisift · paper edition
        ]
      ]
    ]
  ]

  // Front matter ----------------------------------------------------------
  page(header: none)[
    #text(font: "Inter", size: 8.5pt, tracking: 0.08em, fill: rgb("#5E574B"))[
      İÇİNDEKİLER
    ]
    #v(5mm)
    #outline(title: none, depth: 2, indent: auto)
  ]

  body
}

#let definition(body) = block(
  fill: rgb("#F1EADB"),
  stroke: (left: 2pt + rgb("#42594F")),
  inset: 9pt,
  width: 100%,
  body,
)

#let critical(body) = block(
  fill: rgb("#F4EEE3"),
  stroke: (left: 2pt + rgb("#8A3E2F")),
  inset: 9pt,
  width: 100%,
  body,
)

#let example(body) = block(
  fill: none,
  stroke: (left: 1.5pt + rgb("#B08A4F")),
  inset: (left: 9pt, y: 4pt),
  width: 100%,
  body,
)

#let note(body) = block(
  fill: rgb("#F4EEE3"),
  stroke: (left: 1.5pt + rgb("#5E574B")),
  inset: 9pt,
  width: 100%,
  body,
)

#let source_ref(body) = text(
  font: "Inter",
  size: 8pt,
  fill: rgb("#5E574B"),
  [#text("[")#body#text("]")],
)
