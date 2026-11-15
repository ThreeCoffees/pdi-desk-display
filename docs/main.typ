// Plik z treścią pracy. Szablon znajduje się w osobnym pliku template.typ.
#import "template.typ": thesis, appendix, thesis-bibliography, chapter-author

#show: thesis.with(
  lang: "pl",                      // "pl" lub "en"
  // title: "Tytuł pracy dyplomowej",
  // authors: ("Jan Kowalski",),
  // supervisor: "dr inż. Anna Nowak",
  // faculty: "Wydział Elektroniki, Telekomunikacji i Informatyki",
  // field-of-study: "Informatyka",
  // thesis-type: "engineering",      // "bachelor" | "engineering" | "master"
  // year: 2025,
  title-page: image("StronaTytulowa_197637.pdf", width: 100%),  // strona z portalu Moja PG

  abstract-pl: [
    Streszczenie powinno zawierać określenie problemu naukowego lub praktycznego,
    cel i zakres pracy, zastosowane metody badań, wyniki i najważniejsze wnioski.
  ],
  keywords-pl: ("słowo 1", "słowo 2"),
  abstract-en: [
    The abstract should define the problem, the aim and scope of the thesis,
    the research methods, the findings and the most important conclusions.
  ],
  keywords-en: ("keyword 1", "keyword 2"),

  symbols: (
    ($e$, [niepewność pomiaru]),
    ($f$, [częstotliwość [Hz]]),
    ($k$, [stała Boltzmanna $1,38 dot 10^(-23)$ Ws/K]),
  ),
)

= Wstęp i cel pracy

Tekst wstępu. Przy pierwszym użyciu terminu podaj jego definicję, a skróty
rozwijaj przy pierwszym użyciu, np. Local Area Network (LAN)

= Tytuł drugiego rozdziału #chapter-author("Jan Kowalski")

Krótkie wprowadzenie do rozdziału. Zob. @tab-wyniki oraz @rys-schemat.

== Tytuł podrozdziału

Treść podrozdziału. Wzór (@rownanie) opisuje zależność szumu cieplnego:

$ U_n = sqrt(4 k T R f) $ <rownanie>

=== Tytuł punktu podrozdziału

Treść punktu.

#figure(
  rect(width: 60%, height: 3cm, stroke: 0.5pt)[#align(center + horizon)[Rysunek]],
  caption: [Przykładowy rysunek (podpis pod rysunkiem)],
) <rys-schemat>

== Tytuł podrozdziału

#figure(
  table(
    columns: 3,
    [*Parametr*], [*Jednostka*], [*Wartość*],
    [Częstotliwość], [Hz], [50],
    [Napięcie], [V], [230],
  ),
  caption: [Przykładowa tabela (podpis nad tabelą)],
) <tab-wyniki>

= Podsumowanie

Zwięzły opis uzyskanych efektów odniesiony do celu pracy, wnioski oraz
kierunki dalszych prac.

#appendix("Tytuł załącznika")[
  Treść załącznika.
]
