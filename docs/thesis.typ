// #import "template.typ": praca

// #show: praca()

#set text(lang: "pl")
// #set heading(numbering:"1")
#set page(numbering: "1")
#set par(justify: true)

// Nie ma formalnych wymagań określających objętość pracy dyplomowej. Typowa praca dyplomowa licencjacka i inżynierska (projekt inżynierski) zawiera około 40–50 stron tekstu oraz załączniki/dodatki. Natomiast typowa praca dyplomowa magisterska liczy około 60–80 stron tekstu i obliczeń oraz załączniki/dodatki.

// TODO: Rozporządzenia rektora

// TODO: Strona tytułowa (właściwy wzór do pobrania z portalu Moja PG).

= Strona tytułowa
USUŃ TĘ STRONĘ I PODMIEŃ ZA WZÓR
#pagebreak()

// Tytuł: Projekt i implementacja wyświetlacza biurkowego i jego infrastruktury sieciowej
// Title: Design and implementation of a physical desktop screen and its network infrastructure

// Cel: Projekt i wytworzenie urządzenia biurkowego z wyświetlaczem i połączeniem sieciowym. Na ekranie wyświetlane będą m.in. informacje dotyczące prognozy pogody, nagłówki gazet, czytnik artykułów, itp. Urządzenie powinno być zrealizowane z wykorzystaniem wybranego wyświetlacza e-papierowego oraz mikrokontrolera ESP-32. Informacje do wyświetlania na wyświetlaczu będą pobierane z wykorzystaniem kanałów RSS.

// TODO: Streszczenie w języku polskim i Abstract (streszczenie w języku angielskim) Streszczenie powinno zawierać określenie problemu naukowego lub praktycznego do rozwiązania, cel i zakres pracy, zastosowane metody badań, wyniki i najważniejsze wnioski.

== Streszczenie

== Abstract

#pagebreak()

// TODO: Spis treści zawiera zestawienie tytułów wszystkich rozdziałów, podrozdziałów i punktów podrozdziałów pracy wraz z odpowiadającymi im numerami stron. Jeżeli praca jest realizowana przez kilku autorów, to autorzy w każdym rozdziale, ewentualnie podrozdziale podają, kto jest autorem tej części pracy.

#outline()
#pagebreak()

// TODO: Wykaz ważniejszych oznaczeń i skrótów.

= Wykaz oznaczeń i skrótów
#lorem(10)

// TODO: Rozdział 1. Wstęp i cel pracy nakreśla problematykę opisaną lub rozwiązywaną w pracy dyplomowej wraz z uzasadnieniem celowości jej realizacji. Podaje cel i ewentualnie tezę (hipotezę). Syntetycznie opisuje dotychczasowe dokonania w danej tematyce, założenia techniczne oraz może zwięźle przedstawić zawartość poszczególnych rozdziałów.
// W przypadku pracy realizowanej przez kilku studentów, przy omawianiu zawartości rozdziałów należy podać ich autorów. Punkty stanowiące element składowy podrozdziału powinny być opracowane przez jednego autora.

= Wstęp i cel pracy

== Wstęp

Nie jest trudno znaleźć w sklepach internetowych niewielkich wyświetlaczy takich jak ten, który został wytworzony w ramach tego projektu. 
Urządzenia takie są zazwyczaj tworzone z myślą o wąskim przekroju konsumentów w celu spełnienia specyficznych potrzeb, których nie spełniają powszechne urządzenia ogólnego przeznaczenia wyposażone w wyświetlacz i połączenie z siecią Internet, takie jak laptopy, smartfony, komputery stacjonarne, a nawet zegarki typu smartwatch. 
Dla bardziej niszowych zastosowań tworzone są takie urządzenia jak elektroniczne akcesoria ubioru (_wearables_), oraz coraz więcej przeróżnych przedmiotów codziennego użytku wchodzące w skład IoT (_Internet of Things_, Internet Rzeczy).

== Cel pracy
Przedmiot projektu nie jest rozwiązaniem z gruntu unikalnym ani innowacyjnym. Urządzenia zdolne do udostępniania prognozy pogody i wiadomości są w bardzo powszechnym użytku, a przeciętny użytkownik zapewne posiada kilka różnych komputerów do tego zdolnych. Celem projektu jest stworzenie produktu o wąskim zastosowaniu.

Planowane zadania do wykonania:
- Przegląd rozwiązań technologicznych ze szczególnym uwzględnieniem wyświetlaczy e-papierowych i mikrokontrolera ESP-32.
- Projekt urządzenia i oprogramowania.
- Budowa urządzenia na podstawie wcześniejszego projektu.
- Wytworzenie oprogramowania pobierającego wybrane informacje z użyciem kanałów RSS na mikrokontrolerze.
- Wytworzenie oprogramowania wyświetlającego informacje na wyświetlaczu e-papierowym.
- Wytworzenie aplikacji pozwalającej na konfigurowanie kanałów RSS z których pobierane będą informacje.
- Testy opracowanego rozwiązania.

// Kolejne rozdziały rozpoczynają się od krótkiego wprowadzenia, po czym szczegółowo charakteryzuje się poszczególne problemy, ich rozwiązanie i obliczenia. Należy zwrócić uwagę, by nie występowały powtórzenia tych samych treści w różnych fragmentach pracy.
= Przegląd technologii

== Istniejące rozwiązania

- https://trmnl.com/
- https://wiki.seeedstudio.com/xiao_075inch_epaper_panel/

== Mikrokontrolery
=== ESP32

== Wyświetlacze
=== e-ink

= Osiągnięte rezultaty
== Konfiguracja
== Użycie

= Wykorzystane technologie
== Rust

== ESP32
== RSS
== WiFi
== HTTP
== DHCP
== e-ink

= Architektura rozwiązania
== Komponent1
== Komponent2

// Podsumowanie opisuje uzyskane efekty i osiągnięcia pracy i/lub wyniki badań w sposób zwięzły i uporządkowany, odnosząc się do celu i założeń pracy oraz osiągnięć opisanych w poszczególnych rozdziałach. Interpretuje się i formułuje wnioski z przeprowadzonych działań. Ponadto w rozdziale tym należy opisać, jakie czynności należy zrealizować, jeśli prace nad tematem byłyby kontynuowane lub co należy poprawić, jeżeli wyniki były niezadowalające.
= Podsumowanie

@docs-espressif

#set bibliography(title: "Literatura")
#bibliography("sources.bib")

// Załączniki/dodatki zawierają elementy uzupełniające pracę dyplomową, np. w celu prezentacji wykonanych obliczeń, schematy ideowe. Umieszcza się je na końcu pracy.
