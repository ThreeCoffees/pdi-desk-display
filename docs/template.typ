// =============================================================================
//  Szablon pracy dyplomowej – Politechnika Gdańska
//  Zgodny z Zarządzeniem Rektora PG nr 45/2024 z 15 listopada 2024 r.
//  (załączniki 1–4: wytyczne i przykładowy szablon pracy PL/EN)
//
//  Użycie w pliku z treścią pracy:
//    #import "template.typ": thesis, appendix, thesis-bibliography, chapter-author
//    #show: thesis.with(title: "...", authors: ("...",), ...)
// =============================================================================

// ---------------------------------------------------------------------------
//  Teksty zależne od języka pracy
// ---------------------------------------------------------------------------
#let strings = (
  pl: (
    university: "Politechnika Gdańska",
    abstract: "Streszczenie",
    abstract-en: "Abstract",
    keywords: "Słowa kluczowe",
    keywords-en: "Keywords",
    toc: "Spis treści",
    symbols: "Wykaz ważniejszych oznaczeń i skrótów",
    bibliography: "Wykaz literatury",
    appendix: "Załącznik nr",
    author: "Autor",
    authors: "Autorzy",
    supervisor: "Opiekun pracy",
    faculty: "Wydział",
    field: "Kierunek studiów",
    types: (
      bachelor: "Praca dyplomowa licencjacka",
      engineering: "Praca dyplomowa inżynierska",
      master: "Praca dyplomowa magisterska",
    ),
  ),
  en: (
    university: "Gdańsk University of Technology",
    abstract: "Abstract",
    abstract-en: "Abstract",
    keywords: "Keywords",
    keywords-en: "Keywords",
    toc: "Table of contents",
    symbols: "List of important symbols and abbreviations",
    bibliography: "Bibliography",
    appendix: "Appendix",
    author: "Author",
    authors: "Authors",
    supervisor: "Thesis supervisor",
    faculty: "Faculty",
    field: "Field of study",
    types: (
      bachelor: "Bachelor's diploma thesis",
      engineering: "Engineering diploma thesis",
      master: "Master's diploma thesis",
    ),
  ),
)

// ---------------------------------------------------------------------------
//  Funkcje pomocnicze dostępne dla autora pracy
// ---------------------------------------------------------------------------

/// Imię i nazwisko autora rozdziału / podrozdziału (praca zespołowa).
/// Użycie: `= Tytuł rozdziału #chapter-author("Jan Kowalski")`
#let chapter-author(name) = text(weight: "regular", style: "italic")[ (#name)]

#let appendix-counter = counter("thesis-appendix")

/// Załącznik/dodatek umieszczany na końcu pracy.
/// Użycie: `#appendix("Tytuł załącznika")[Treść]`
#let appendix(title, body) = {
  appendix-counter.step()
  context {
    let n = appendix-counter.get().first()
    let label = strings.at(text.lang).appendix
    heading(level: 1, numbering: none)[#label #n: #title]
  }
  body
}

/// Wykaz literatury. Argumentem jest element `bibliography(...)` utworzony
/// w pliku z treścią pracy (dzięki temu ścieżka do pliku .yml/.bib jest
/// liczona względem pliku pracy, a nie szablonu).
#let thesis-bibliography(bib) = {
  heading(level: 1, numbering: none)[#context strings.at(text.lang).bibliography]
  bib
}

// ---------------------------------------------------------------------------
//  Główna funkcja szablonu
// ---------------------------------------------------------------------------
#let thesis(
  // --- język: "pl" (praca po polsku) lub "en" (praca po angielsku) ---
  lang: "pl",
  // --- dane na stronie tytułowej ---
  title: "Tytuł pracy",
  authors: (),
  supervisor: "",
  faculty: "",
  field-of-study: "",
  thesis-type: "engineering", // "bachelor" | "engineering" | "master"
  city: "Gdańsk",
  year: none,
  // Gotowa strona tytułowa pobrana z portalu Moja PG (np. image("tytulowa.png")).
  // Jeśli none – generowana jest strona zastępcza.
  title-page: none,
  // --- streszczenia (maks. 1 strona każde) ---
  abstract-pl: none,          // używane tylko dla lang: "pl"
  keywords-pl: (),
  abstract-en: none,
  keywords-en: (),
  // --- wykaz oznaczeń i skrótów: tablica par (symbol, opis) ---
  symbols: (),
  // --- formatowanie ---
  font: ("Arial", "Liberation Sans"), // krój uczelni (Księga Identyfikacji Wizualnej PG)
  font-size: 10pt,
  line-spacing: 1.5,                  // interlinia 1,5 wiersza
  double-sided: true,                 // druk dwustronny: abstrakt zaczyna się na stronie nr 3
  chapter-pagebreak: false,           // true = każdy rozdział od nowej strony
  body,
) = {
  let s = strings.at(lang)
  let year = if year == none { datetime.today().year() } else { year }

  // Interlinia: 1,5 wiersza jak w edytorach tekstu (pojedynczy wiersz Arial ≈ 1,15 em;
  // Typst liczy odstęp od krawędzi wysokości wersalików ≈ 0,716 em).
  let leading = line-spacing * 1.15em - 0.716em

  set document(title: title, author: authors)

  // Format A4, pionowo, marginesy w odbiciu lustrzanym (inside/outside),
  // ciągła numeracja w stopce, bez numeru na stronie tytułowej.
  set page(
    paper: "a4",
    flipped: false,
    margin: (inside: 3cm, outside: 2cm, top: 2.5cm, bottom: 2.5cm),
    numbering: "1",
    footer: context {
      let n = counter(page).get().first()
      if n > 1 { align(center, counter(page).display("1")) }
    },
  )

  set text(font: font, size: font-size, lang: lang, hyphenate: true)
  set par(justify: true, leading: leading, spacing: 1.2em)

  // --- nagłówki: 1. / 1.1. / 1.1.1. ---
  set heading(numbering: (..n) => n.pos().map(str).join(".") + ".")
  show heading: set text(font: font, weight: "bold")
  show heading.where(level: 1): set text(size: 14pt)
  show heading.where(level: 2): set text(size: 12pt)
  show heading.where(level: 3): set text(size: 10.5pt)
  show heading.where(level: 1): set block(above: 2em, below: 1.2em)
  show heading.where(level: 2): set block(above: 1.6em, below: 1em)
  show heading.where(level: 3): set block(above: 1.4em, below: 0.9em)
  // Tytuły rozdziałów (poziom 1) wersalikami.
  show heading.where(level: 1): it => {
    show text: upper
    if chapter-pagebreak { pagebreak(weak: true) }
    it
  }

  // --- spis treści: też wersaliki na poziomie 1 ---
  show outline.entry.where(level: 1): it => {
    show text: upper
    it
  }

  // --- rysunki i tabele ---
  // Tytuł tabeli – nad tabelą, tytuł rysunku – pod rysunkiem.
  show figure.where(kind: table): set figure.caption(position: top)
  show figure.where(kind: image): set figure.caption(position: bottom)
  set figure.caption(separator: [. ])
  show figure.where(kind: image): set block(breakable: false)
  show figure.caption: set text(size: font-size)
  show figure: set block(above: 1.2em, below: 1.2em)
  // Każda tabela i rysunek muszą być przywołane w tekście: @etykieta.

  // --- wzory ---
  set math.equation(numbering: "(1)")

  // --- listy ---
  set list(indent: 1em)
  set enum(indent: 1em)

  // =========================================================================
  //  1. Strona tytułowa (strona nr 1, numer niewidoczny)
  // =========================================================================
  if title-page != none {
    title-page
  } else {
    // Strona zastępcza – właściwy wzór należy pobrać z portalu Moja PG.
    align(center)[
      #text(size: 14pt, weight: "bold")[#upper(s.university)]
      #v(0.4em)
      #if faculty != "" [#s.faculty #faculty]
      #v(1fr)
      #text(size: 12pt)[#s.types.at(thesis-type)]
      #v(1.5em)
      #text(size: 18pt, weight: "bold")[#title]
      #v(1fr)
      #set align(left)
      #if authors.len() > 0 [
        *#if authors.len() > 1 { s.authors } else { s.author }:*
        #authors.join(", ") \
      ]
      #if field-of-study != "" [*#s.field:* #field-of-study \ ]
      #if supervisor != "" [*#s.supervisor:* #supervisor]
      #v(1fr)
      #city #year
    ]
  }

  // Dla druku dwustronnego streszczenie wypada na stronie nr 3 (odd).
  pagebreak(to: if double-sided { "odd" } else { none })

  // =========================================================================
  //  2. Streszczenie (tylko wersja PL) i Abstract – maks. 1 strona każde
  // =========================================================================
  let abstract-page(head, body, kw-label, kws) = {
    heading(level: 1, numbering: none, outlined: false)[#head]
    body
    if kws.len() > 0 [
      #v(0.8em)
      *#kw-label:* #kws.join(", ")
    ]
  }

  if lang == "pl" {
    if abstract-pl != none {
      abstract-page(s.abstract, abstract-pl, s.keywords, keywords-pl)
      pagebreak(weak: true)
    }
    if abstract-en != none {
      abstract-page(s.abstract-en, abstract-en, s.keywords-en, keywords-en)
      pagebreak(weak: true)
    }
  } else {
    if abstract-en != none {
      abstract-page(s.abstract-en, abstract-en, s.keywords-en, keywords-en)
      pagebreak(weak: true)
    }
  }

  // =========================================================================
  //  3. Spis treści (rozdziały, podrozdziały, punkty podrozdziałów)
  // =========================================================================
  heading(level: 1, numbering: none, outlined: false)[#s.toc]
  outline(title: none, depth: 3, indent: 1.5em)
  pagebreak(weak: true)

  // =========================================================================
  //  4. Wykaz ważniejszych oznaczeń i skrótów
  // =========================================================================
  if symbols.len() > 0 {
    heading(level: 1, numbering: none)[#s.symbols]
    set par(justify: false)
    for (sym, desc) in symbols [
      #sym – #desc \
    ]
  }

  // =========================================================================
  //  5. Treść pracy (rozdziały, podsumowanie, literatura, załączniki)
  // =========================================================================
  body
}
