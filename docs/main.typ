// Plik z treścią pracy. Szablon znajduje się w osobnym pliku template.typ.
#import "template.typ": thesis, appendix, thesis-bibliography, chapter-author

#let placeholder(body) = {
  context {
    let x = here().position().x
    box(width: 0pt, height: 0pt, place(
      top + left,
      dx: 6mm - x, // 6 mm from the left page edge
      dy: -0.85em,
      rect(width: 1mm, height: 2em, fill: red, radius: 1pt),
    ))
  }
  // Inline text
  text(fill: red, weight: "medium")[#body]
}

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
    #placeholder([
    Streszczenie powinno zawierać określenie problemu naukowego lub praktycznego,
    cel i zakres pracy, zastosowane metody badań, wyniki i najważniejsze wnioski.
    ])
  ],
  // keywords-pl: ("słowo 1", "słowo 2"),
  abstract-en: [
    #placeholder([
    The abstract should define the problem, the aim and scope of the thesis,
    the research methods, the findings and the most important conclusions.
    ])
  ],
  // keywords-en: ("keyword 1", "keyword 2"),

  symbols: (
    ([ESP32], [rodzina mikrokontrolerów firmy Espressif]),
    ([EPD], [Electrophoretic display - wyświetlacz elektroforetyczny]),

  ),
)


// TODO: Rozdział 1. Wstęp i cel pracy nakreśla problematykę opisaną lub rozwiązywaną w pracy dyplomowej wraz z uzasadnieniem celowości jej realizacji. Podaje cel i ewentualnie tezę (hipotezę). Syntetycznie opisuje dotychczasowe dokonania w danej tematyce, założenia techniczne oraz może zwięźle przedstawić zawartość poszczególnych rozdziałów.
// W przypadku pracy realizowanej przez kilku studentów, przy omawianiu zawartości rozdziałów należy podać ich autorów. Punkty stanowiące element składowy podrozdziału powinny być opracowane przez jednego autora.

= Wstęp i cel pracy

== Wstęp

Nie jest trudno znaleźć w sklepach internetowych niewielkich wyświetlaczy takich jak ten, który został wytworzony w ramach tego projektu. 
Urządzenia takie są zazwyczaj tworzone z myślą o wąskim przekroju konsumentów w celu spełnienia specyficznych potrzeb, których nie spełniają powszechne urządzenia ogólnego przeznaczenia wyposażone w wyświetlacz i połączenie z siecią Internet, takie jak laptopy, smartfony, komputery stacjonarne, zegarki typu smartwatch, i wiele innych. 
Dla bardziej niszowych zastosowań tworzone są takie urządzenia jak elektroniczne akcesoria ubioru (_wearables_), oraz coraz więcej przeróżnych przedmiotów codziennego użytku wchodzące w skład IoT (_Internet of Things_, Internet Rzeczy). Takim właśnie urządzeniem jest to stworzone w ramach projektu.

== Cel pracy

Przedmiot projektu nie jest rozwiązaniem z gruntu unikalnym ani innowacyjnym. Urządzenia zdolne do udostępniania prognozy pogody i wiadomości są w bardzo powszechnym użytku, a przeciętny użytkownik zapewne posiada kilka różnych komputerów do tego zdolnych. Celem projektu jest stworzenie produktu specjalnego przeznaczenia do czytania prognozy i wiadomości _bez konieczności obsługi_ po dokonaniu wstępnej konfiguracji.

Planowane zadania do wykonania:
- Przegląd rozwiązań technologicznych ze szczególnym uwzględnieniem wyświetlaczy e-papierowych i mikrokontrolera ESP-32.
- Projekt urządzenia i oprogramowania.
- Budowa urządzenia na podstawie wcześniejszego projektu.
- Wytworzenie oprogramowania pobierającego wybrane informacje z użyciem kanałów RSS na mikrokontrolerze.
- Wytworzenie oprogramowania wyświetlającego informacje na wyświetlaczu e-papierowym.
- Wytworzenie aplikacji pozwalającej na konfigurowanie kanałów RSS z których pobierane będą informacje.
- Testy opracowanego rozwiązania.

Osiągnięte zostały #placeholder([x `out of` 7]) z zamierzonych celów.

// Kolejne rozdziały rozpoczynają się od krótkiego wprowadzenia, po czym szczegółowo charakteryzuje się poszczególne problemy, ich rozwiązanie i obliczenia. Należy zwrócić uwagę, by nie występowały powtórzenia tych samych treści w różnych fragmentach pracy.
= Przegląd technologii

== Istniejące rozwiązania

- https://trmnl.com/
- https://wiki.seeedstudio.com/xiao_075inch_epaper_panel/

== Mikrokontrolery
=== ESP32



== Wyświetlacze

LCD, LED

=== Papier elektroniczny

Wyświetlacz e-papierowy (e-papier) znacząco różni się od wyświetlaczy LCD i LED. Jego przeznaczeniem jest imitować swoim wyglądem papier z naniesionym tuszem. Pożądane cechy takiego wyświetlacza to m.in. możliwość wyświetlania obrazu bez emitowania własnego światła, możliwość czytania w świetle słonecznym oraz, opcjonalnie, możliwość wyświetlania kolorów innych niż czerń i biel. 

Istnieje wiele technologii wyświetlaczy imitujących papier, jednak projekt skupia się wokół wyświetlacza elektroforetycznego (EPD, Electrophoretic Display). @electrophoretic-display Komórki elektroforetyczne cechują się bistabilnością (w przypadku wyświetlaczy czarno-białych), tj. po odcięciu prądu od wyświetlacza piksele czarne pozostają czarne, a białe pozostają białe.

==== Budowa i sposób działania komórki EPD

W medium komórki EPD zawieszone są cząsteczki tuszu - białego oraz czarnego w przypadku wyświetlaczy dwukolorowych. Z dwóch stron komórki znajdują się elektrody. Jedna umieszczona jest od strony urządzenia. Druga jest przezroczysta i znajduje się od strony odbiorcy. To, która elektroda jest katodą, a która anodą, zależy od zastosowanego tuszu i tego, w jaki sposób ten przejawia zjawisko elektroforezy. Przykładowy schemat budowy komórki elektroforetycznej o dwóch kolorach przedstawia @electrophoretic-bw-schema.

#figure([#placeholder([Schemat piksela elektroforetycznego])],
  caption: [Schemat piksela elektroforetycznego]
)<electrophoretic-bw-schema>

Elektroforeza polega na tym, że pod wpływem różnych napięć pomiędzy elektrodami tusz biały i czarny przemieszczają się wewnątrz medium w różnych kierunkach - z dala od, lub w kierunku odbiorcy - przez co przesłaniają się nawzajem. Po usunięciu napięcia w komórce tusz zatrzymuje się w miejscu.

==== Korzystanie z EPD

#placeholder([Ta sekcja bardziej pasowałaby w rozdziale o implementacji])

= Osiągnięte rezultaty
== Konfiguracja
== Użycie

= Wykorzystane technologie
== Rust na ESP32

@implrust

== WiFi, HTTP, DHCP

Urządzenie łączy się bezprzewodowo na dwa sposoby. Pierwszy łączy płytkę z urządzeniem do konfiguracji ustawień użytkownika. Drugi sposób płytka wykorzystuje autonomicznie, po zakończonej konfiguracji, do pobierania zasobów z sieci Internet. Oba połączenia są bezprzewodowe z użyciem WiFi.

Przed konfiguracją płytka nie jest jeszcze połączona z żadną zewnętrzną siecią. Interfejs fizyczny urządzenia nie jest wyposażony w klawiaturę, więc wpisanie doń ustawień bezpośrednio nie jest wykonalne w prosty do użytkowania sposób. Rozsądnym wymaganiem jest to, aby konfiguracja powinna odbywać się za pomocą klawiatury. Rozważone zostały następujące rozwiązania:

- Protokół *TELNET* #placeholder([jest do niczego i do bani ale za to jest prosty i do bani])
- Bluetooth. #placeholder([])
- Do konfiguracji został wybrany *Access Point (AP) WiFi*. Wybór mikrokontrolera był podyktowany m.in. właśnie możliwością eksponowania własnej sieci WiFi. Ponadto upraszcza architekturę projektu, ponieważ połączenie jako klient WiFi jest również wymaganiem już po zakończonej konfiguracji. 

Płytka serwuje AP o znanym SSID (`desk_display`) oraz haśle. Użytkownik konfiguruje płytkę używając dowolnego urządzenia wyposażonego w przeglądarkę internetową i możliwość połączenia z WiFi. Po połączeniu urządzenie konfigurujące jest w sieci lokalnej mikrokontrolera. Mikrokontroler ma w niej przypisany na stałe znany adres IP `192.168.2.1`. Na standardowym porcie protokołu HTTP (HyperText Transfer Protocol) o numerze `80` serwuje stronę konfiguracyjną. Zrzut ekranu przeglądarki na stronie konfiguracyjnej znajduje się poniżej (@config-screenshot).

#figure([#placeholder([Zrzut ekranu strony konfiguracyjnej])],
  caption: [Zrzut ekranu strony konfiguracyjnej]
)<config-screenshot>

W formularzach HTML znajdujących się na stronie można skonfigurować następujące parametry:
#placeholder([
- dupa 
- cyce
- wadowice
])

Wypełnione formularze są wysyłane metodą POST protokołu HTTP na mikrokontroler, który w rezultacie aktualizuje swoją konfigurację. Użytkownik może zweryfikować poprawność konfiguracji. Połączenie z WiFi można sprawdzić przyciskiem "Test board's Internet connection". Pozyskiwanie syndykatów RSS można sprawdzić #placeholder([w inny sposób]).

Strona konfiguracyjna jest zbudowana w technologii Webowej z użyciem języków HTML, CSS oraz JavaScript.

== Wyświetlacz e-papierowy
== RSS

= Architektura rozwiązania
== Komponent1
== Komponent2

// Podsumowanie opisuje uzyskane efekty i osiągnięcia pracy i/lub wyniki badań w sposób zwięzły i uporządkowany, odnosząc się do celu i założeń pracy oraz osiągnięć opisanych w poszczególnych rozdziałach. Interpretuje się i formułuje wnioski z przeprowadzonych działań. Ponadto w rozdziale tym należy opisać, jakie czynności należy zrealizować, jeśli prace nad tematem byłyby kontynuowane lub co należy poprawić, jeżeli wyniki były niezadowalające.
= Podsumowanie

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

#set bibliography(title: "Literatura")
#bibliography("sources.bib")

#appendix("Tytuł załącznika")[
  Treść załącznika.
]
