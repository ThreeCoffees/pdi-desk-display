#set page(
  width: 13.333in,
  height: 7.5in,
  margin: (x: 0.62in, y: 0.42in),
  fill: rgb("#ffffff"),
)
#set text(font: "DejaVu Sans", size: 17pt, fill: rgb("#1f2937"))
#set par(leading: 0.72em)
#set list(indent: 18pt, body-indent: 6pt, spacing: 8pt)

#let ink = rgb("#172b4d")
#let muted = rgb("#526173")
#let accent = rgb("#245b8f")
#let panel-bg = rgb("#f3f6f9")
#let border = rgb("#cbd5e1")

#let panel(body) = block(
  width: 100%,
  fill: panel-bg,
  stroke: (paint: border, thickness: 0.8pt),
  inset: 14pt,
)[#body]

#let placeholder(label, detail, height: 2.1in) = block(
  width: 100%,
  height: height,
  fill: rgb("#f8fafc"),
  stroke: (paint: accent, thickness: 1.2pt),
  inset: 14pt,
)[
  #align(center + horizon)[
    #text(size: 13pt, weight: "bold", fill: accent)[#label]
    #v(7pt)
    #text(size: 10pt, fill: muted)[#detail]
  ]
]

#let slide(section, title, body) = {
  pagebreak()
  grid(
    columns: (1fr, auto),
    gutter: 12pt,
    [#text(size: 10pt, weight: "bold", fill: muted)[DESK-DISPLAY · SEMINARIUM]],
    [#text(size: 10pt, weight: "bold", fill: accent)[#section]],
  )
  v(40pt)
  body
  v(1fr)
  line(length: 100%, stroke: border)
  v(7pt)
  grid(
    columns: (1fr,),
    [#text(size: 9pt, fill: muted)[desk-display · firmware urządzenia]],
  )
}

#text(size: 10pt, weight: "bold", fill: accent)[PREZENTACJA II]
#v(25pt)
#text(size: 42pt, weight: "bold", fill: ink)[RSS w firmware desk-display]
#v(8pt)
#text(size: 25pt, fill: muted)[Przetwarzanie strumieniowe i ograniczone zasoby]
#v(23pt)
#grid(
  columns: (1fr,),
  [
    #panel[
      #text(size: 20pt, weight: "bold", fill: ink)[Przetwarzanie danych RSS]
      #v(9pt)
      - Odbierać kanał porcjami i przekazywać dane parserowi.
      - Przechowywać wynik w strukturach o ograniczonej pojemności.
      #v(8pt)
      #text(size: 12pt, fill: muted)[Jak projektować taki przepływ w firmware o ograniczonych zasobach?]
    ]
  ],
)

#slide("Wprowadzenie", "Projekt w skrócie", [
  #grid(
    columns: (1.15fr, 1fr),
    gutter: 18pt,
    [
      #text(size: 22pt, weight: "bold", fill: ink)[Jak przetwarzać dane w urządzeniu o ograniczonych zasobach?]
      #v(12pt)
      - To firmware działające na urządzeniu o ograniczonych zasobach.
      - Kanały RSS pobierane są przez sieć i analizowane przez parser.
      - Warto przyjrzeć się sposobowi odbioru danych i przechowywania wyniku.
    ],
    [
      #panel[
        #text(size: 13pt, weight: "bold", fill: accent)[KONTEKST]
        #v(7pt)
        #text(size: 16pt)[Projekt `desk-display` łączy kod urządzenia z osobnymi komponentami, które można testować na hoście.]
        #v(8pt)
        #text(size: 12pt, fill: muted)[Zaczynamy od drogi danych RSS i ograniczeń pamięci.]
      ]
    ],
  )
])

#slide("Tematy prezentacji", "Od danych RSS do diagnostyki obrazu", [
  #grid(
    columns: (1fr, 1fr),
    gutter: 10pt,
    [#panel[Kontekst projektu]],
    [#panel[Droga danych RSS]],
    [#panel[Parser i pamięć]],
    [#panel[Testowanie wybranych elementów]],
    [#panel[Framebuffer i diagnostyka ekranu]],
    [#panel[Zewnętrzny podgląd obrazu]],
    [#panel[Możliwości i ograniczenia]],
    [#panel[Podsumowanie i pytania]],
  )
])

#slide("Droga danych RSS", "Od źródła do wyniku", [
  #grid(
    columns: (1.2fr, 1fr),
    gutter: 18pt,
    [
      #placeholder(
        "MIEJSCE NA DIAGRAM",
        "Skonfigurowany kanał → pobranie HTTP → stopniowy parser → wynik RSS",
        height: 2.6in,
      )
    ],
    [
      #text(size: 19pt, weight: "bold")[Co robi firmware?]
      #v(10pt)
      - Pobiera adresy z konfiguracji kanałów.
      - Odbiera treść odpowiedzi i przekazuje ją parserowi.
      - Po zakończeniu odczytu finalizuje parsowanie.
    ],
  )
])

#slide("Przetwarzanie strumieniowe", "Nie czekaj na cały dokument", [
  #grid(
    columns: (1fr, 1fr),
    gutter: 18pt,
    [
      #text(size: 20pt, weight: "bold", fill: ink)[Odbiór porcjami]
      #v(10pt)
      #text(size: 15pt)[Odebrane fragmenty są przekazywane parserowi na bieżąco, bez wcześniejszego gromadzenia całego dokumentu.]
      #v(12pt)
      #panel[#text(size: 12pt, fill: muted)[Idea: parser może pracować w trakcie odbioru, bez gromadzenia całego XML w jednym buforze wejściowym.]]
    ],
    [
      #text(size: 20pt, weight: "bold", fill: ink)[Co to daje?]
      #v(10pt)
      - Przepływ danych ma wyraźne etapy.
      - Parser utrzymuje stan pomiędzy kolejnymi porcjami.
      - To podejście pasuje do środowiska o ograniczonych zasobach.
    ],
  )
])

#slide("Parser i pamięć", "Stan parsera, ograniczony wynik", [
  #grid(
    columns: (1fr, 1fr),
    gutter: 18pt,
    [
      #panel[
        #text(size: 14pt, weight: "bold", fill: accent)[PARSER]
        #v(8pt)
        #text(size: 15pt)[Przyjmuje kolejne fragmenty, rozpoznaje elementy RSS i buduje wynik.]
        #v(8pt)
        #text(size: 11pt, fill: muted)[Rozpoznawanie elementów kanału i wpisów.]
      ]
    ],
    [
      #panel[
        #text(size: 14pt, weight: "bold", fill: accent)[WYNIK]
        #v(8pt)
        #text(size: 15pt)[Wynik ma określony rozmiar i ograniczoną liczbę wpisów.]
        #v(8pt)
        #text(size: 11pt, fill: muted)[Łatwiej przewidzieć, ile miejsca zajmą dane.]
      ]
    ],
  )
  #v(18pt)
  #align(center)[#text(size: 21pt, weight: "bold", fill: ink)[Przewidywalniejsze zasoby ↔ ograniczona ilość przechowywanych danych]]
])

#slide("Testowanie", "Wybrane elementy bez płytki", [
  #grid(
    columns: (1fr, 1fr),
    gutter: 18pt,
    [
      #text(size: 20pt, weight: "bold")[Co można sprawdzić na hoście?]
      #v(10pt)
      - Poprawność odczytu i analizy kanałów RSS.
      - Zachowanie pamięci przechowującej obraz.
      - Wybrane przypadki bez uruchamiania całego urządzenia.
    ],
    [
      #panel[
        #text(size: 15pt, weight: "bold", fill: accent)[ZAKRES TESTÓW]
        #v(9pt)
        #text(size: 15pt)[Testy pomagają sprawdzić wybrane elementy bez uruchamiania całego urządzenia.]
      ]
    ],
  )
])

#slide("Motyw podglądu", "Gdy panel nie pokazuje obrazu", [
  #grid(
    columns: (1fr, 1fr),
    gutter: 18pt,
    [
      #text(size: 21pt, weight: "bold")[Pytanie]
      #v(8pt)
      #text(size: 17pt)[Czy system przygotowuje właściwą klatkę, mimo że uszkodzony ekran jej nie wyświetla?]
      #v(14pt)
      #text(size: 21pt, weight: "bold")[Pomysł]
      #v(8pt)
      #text(size: 16pt)[Odczytać zawartość framebufferu z zewnątrz i zobaczyć ją na komputerze.]
    ],
    [
      #panel[
        #text(size: 14pt, weight: "bold", fill: accent)[CO MOŻE POMÓC USTALIĆ PODGLĄD?]
        #v(10pt)
        - Czy framebuffer zawiera oczekiwaną klatkę.
        - Czy problem może leżeć po stronie panelu lub jego połączenia.
        #v(9pt)
        #text(size: 11pt, fill: muted)[To pomoc diagnostyczna, nie samodzielna diagnoza sprzętowa.]
      ]
    ],
  )
])

#slide("Framebuffer", "Dwa bufory i aktualna klatka", [
  #grid(
    columns: (1.15fr, 1fr),
    gutter: 18pt,
    [
      #placeholder(
        "MIEJSCE NA SCHEMAT",
        "Bufor roboczy: przygotowanie kolejnej klatki → zamiana ról → bufor aktualny: klatka udostępniana do odczytu",
        height: 2.7in,
      )
    ],
    [
      #text(size: 19pt, weight: "bold")[Podwójne buforowanie]
      #v(9pt)
      - Bufor aktualny przechowuje obraz dostępny do odczytu.
      - W drugim buforze przygotowywana jest kolejna klatka.
      - Po przygotowaniu bufory zamieniają się rolami.
    ],
  )
])

#slide("Udostępnianie obrazu", "Obraz przesyłany przez sieć", [
  #grid(
    columns: (1fr, 1fr),
    gutter: 18pt,
    [
      #panel[
        #text(size: 14pt, weight: "bold", fill: accent)[ŻĄDANIE]
        #v(9pt)
        #text(size: 18pt, weight: "bold")[Żądanie HTTP]
        #v(9pt)
        #text(size: 13pt, fill: muted)[Urządzenie odczytuje aktualny obraz.]
      ]
    ],
    [
      #panel[
        #text(size: 14pt, weight: "bold", fill: accent)[ODPOWIEDŹ]
        #v(9pt)
        #text(size: 18pt, weight: "bold")[Dane obrazu]
        #v(9pt)
        #text(size: 13pt, fill: muted)[Surowe bajty obrazu monochromatycznego.]
      ]
    ],
  )
])

#slide("Podgląd obrazu", "Od danych do okna", [
  #grid(
    columns: (1.1fr, 1fr),
    gutter: 18pt,
    [
      #placeholder(
        "MIEJSCE NA DIAGRAM",
        "Odczyt obrazu przez sieć → odtworzenie pikseli → podgląd w oknie na komputerze",
        height: 2.6in,
      )
    ],
    [
      #text(size: 18pt, weight: "bold")[Narzędzie podglądu na komputerze]
      #v(8pt)
      - Pobiera dane obrazu z urządzenia przez sieć.
      - Odtwarza piksele z bitów framebufferu.
      - Wyświetla obraz w osobnym oknie.
    ],
  )
])

#slide("Przykład podglądu", "Zrzut zamiast pokazu na żywo", [
  #grid(
    columns: (1.25fr, 1fr),
    gutter: 18pt,
    [
      #placeholder(
        "WSTAW ZRZUT EKRANU PODGLĄDU",
        "Zrzut okna narzędzia podglądu; zaznacz, czy obraz pochodzi z urządzenia, czy z przykładu.",
        height: 3.2in,
      )
    ],
    [
      #text(size: 19pt, weight: "bold")[Podgląd na komputerze]
      #v(8pt)
      #text(size: 16pt)[Zrzut pokazuje obraz z urządzenia w osobnym oknie.]
    ],
  )
])

#slide("Granice podglądu", "Co mówi obraz — i czego nie mówi", [
  #grid(
    columns: (1fr, 1fr),
    gutter: 18pt,
    [
      #panel[
        #text(size: 15pt, weight: "bold", fill: accent)[MOŻE POMÓC]
        #v(9pt)
        #text(size: 16pt)[Sprawdzić, jaki obraz znajduje się w udostępnionym framebufferze.]
        #v(9pt)
        #text(size: 12pt, fill: muted)[To dodatkowa obserwacja pracy systemu.]
      ]
    ],
    [
      #panel[
        #text(size: 15pt, weight: "bold", fill: accent)[GRANICE PODGLĄDU]
        #v(9pt)
        #text(size: 16pt)[Nie potwierdza, czy panel fizycznie wyświetla klatkę ani czy połączenie z ekranem działa prawidłowo.]
        #v(9pt)
        #text(size: 12pt, fill: muted)[Podgląd dotyczy obrazu zapisanego w framebufferze.]
      ]
    ],
  )
])

#slide("Dwa zagadnienia", "Przetwarzanie danych i diagnostyka obrazu", [
  #grid(
    columns: (1fr, 1fr),
    gutter: 18pt,
    [
      #panel[
        #text(size: 15pt, weight: "bold", fill: accent)[PRZETWARZANIE RSS]
        #v(9pt)
        #text(size: 16pt)[Pobieranie kanałów, parsowanie porcjami i wynik o ograniczonej pojemności.]
      ]
    ],
    [
      #panel[
        #text(size: 15pt, weight: "bold", fill: accent)[DIAGNOSTYKA OBRAZU]
        #v(9pt)
        #text(size: 16pt)[Odczyt framebufferu przez HTTP i zewnętrzny podgląd klatki.]
      ]
    ],
  )
  #v(20pt)
  #align(center)[#text(size: 15pt, fill: muted)[Podgląd pozwala ocenić zawartość framebufferu, ale nie zastępuje ani nie naprawia ekranu.]]
])

#slide("Pytania", "Pytania?", [
  #v(30pt)
  #align(center)[#text(size: 38pt, weight: "bold", fill: accent)[Dziękuję]]
  #v(14pt)
  #align(center)[#text(size: 20pt, fill: ink)[Pytania?]]
])
