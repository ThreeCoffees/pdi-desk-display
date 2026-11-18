// Seminarium dyplomowe, część 1: kontekst projektu.
// Typst + touying (motyw "simple", bez stylizacji).
// Kompilacja: typst compile prezentacja_czesc1.typ
// Notatki prelegenta: ustaw show-notes na true, aby zobaczyć je obok slajdów w PDF.

#import "@preview/touying:0.6.1": *
#import themes.simple: *
#import themes.dewdrop: *

#let show-notes = false

// --- Pomocnicze: czerwone znaczniki braków (do uzupełnienia później) ---
#let ph(body) = text(fill: red, weight: "medium")[[TODO: #body]]
#let ph-fig(desc, h: 6cm) = rect(
  width: 100%,
  height: h,
  stroke: (paint: red, dash: "dashed"),
  inset: 1em,
)[#align(center + horizon)[#text(fill: red)[Miejsce na rysunek: #desc]]]

#show: dewdrop-theme.with(
  aspect-ratio: "16-9",
  config-info(
    title: [Projekt i implementacja wyświetlacza biurkowego i jego infrastruktury sieciowej],
    subtitle: [Seminarium dyplomowe, część 1: kontekst projektu],
    author: [Adam Matczak, Leonard Mądzielewski],
    date: datetime.today(),
    institution: [
      Politechnika Gdańska, Wydział Elektroniki, Telekomunikacji i Informatyki \
      Promotor: dr inż. Krzysztof Bikonis
    ],
  ),
  config-common(
    show-notes-on-second-screen: if show-notes { right } else { none },
  ),
)

#set text(lang: "pl")

#title-slide([
    = Projekt i implementacja wyświetlacza biurkowego i jego infrastruktury sieciowej
  === Seminarium dyplomowe, część 1: kontekst projektu

  Adam Matczak, Leonard Mądzielewski \
  Promotor: dr inż. Krzysztof Bikonis \
  // Politechnika Gdańska, Wydział Elektroniki, Telekomunikacji i Informatyki
])

#speaker-note[
  (ok. 1 min) Przedstawiamy się, podajemy tytuł i promotora. Zaznaczyć na starcie: to nie obrona, tylko opowieść o tym, *w jakim świecie* powstaje nasz projekt: jakie technologie, jakie podobne produkty i jakie pułapki. Szczegóły implementacji będą w części 2.
]

== Plan

+ Projekt: co robimy i po co
+ Przegląd technologii: e-ink, ESP32, alternatywy
+ Przegląd rynku: kto już to zrobił
+ Wyzwania: trzy rzeczy, które łatwo zepsuć

#speaker-note[
  (ok. 0,5 min) Cztery bloki. Najwięcej czasu poświęcimy technologiom i wyzwaniom, bo to tam jest najwięcej rzeczy do wyniesienia dla kogoś, kto nie zajmuje się embedami na co dzień.
]

= Projekt

== Co robimy

- Urządzenie biurkowe z wyświetlacza e-papierowego i mikrokontrolera ESP32
- Pokazuje *prognozę pogody* i *wiadomości* z kanałów RSS
- Po wstępnej konfiguracji działa *bez obsługi*
- Konfiguracja z dowolnego urządzenia z przeglądarką i Wi-Fi

#ph-fig([zdjęcie / render urządzenia], h: 5cm)

#speaker-note[
  (ok. 1,5 min) Najpierw sam produkt: mała tabliczka na biurku. Ważne zdanie: "bez konieczności obsługi". Nie ma klawiatury, nie ma ekranu dotykowego. Włączasz, konfigurujesz raz, zapominasz. Zdjęcie lub render wstawimy, gdy będzie gotowa obudowa.
]

== Po co, skoro mamy telefony

- Każdy ma już kilka urządzeń, które pokażą pogodę i wiadomości
- Ale: smartfon i laptop to urządzenia ogólnego przeznaczenia. Powiadomienia, aplikacje, rozpraszacze
- Urządzenie specjalnego przeznaczenia: *zawsze widoczne, ciche, bezobsługowe*
- Projekt nie jest innowacją produktową. Jest *ćwiczeniem inżynierskim* na styku sprzętu, sieci i oprogramowania wbudowanego

#speaker-note[
  (ok. 1,5 min) Uczciwie: to nie jest wynalazek, takie urządzenia istnieją (pokażemy je w przeglądzie rynku). Wartość projektu to nauka: sprzęt, sieć, Rust na mikrokontrolerze. Nisza: IoT, urządzenia peryferyjne, które robią jedną rzecz. #ph[doprecyzować motywację osobistą / zespołową]
]

== Zakres w jednym obrazku

#ph-fig([schemat blokowy: Internet (RSS, pogoda) - Wi-Fi - ESP32 - e-ink; telefon - AP ESP32 - strona konfiguracyjna], h: 6.5cm)

#speaker-note[
  (ok. 1 min) Dwa połączenia Wi-Fi: jedno do konfiguracji (płytka jako punkt dostępowy), drugie do pobierania danych z Internetu (płytka jako klient). Do tego wyświetlacz. Szczegóły architektury zostawiamy na część 2. Tu tylko orientacja w terenie.
]

= Przegląd technologii

== E-ink: co to jest

- Wyświetlacz *elektroforetyczny*: w mikrokapsułkach są naładowane cząstki (np. czarne i białe)
- Pole elektryczne przesuwa cząstki ku powierzchni, powstaje obraz
- *Nie świeci*. Odbija światło otoczenia, tak jak papier
- *Bistabilny*: obraz zostaje bez zasilania. Prąd potrzebny tylko do zmiany obrazu

#ph-fig([schemat mikrokapsułek, rysunek z dokumentacji E Ink lub własny], h: 5cm)

#speaker-note[
  (ok. 2 min) Intuicja: jak magnesik na lodówce. Raz ustawisz, zostaje. Dlatego e-czytniki trzymają miesiące na baterii. Podkreślić "bistabilność": to najważniejsza własność, która później tłumaczy zarówno zalety (energia), jak i wyzwania (nie wolno trzymać obrazu w nieskończoność, o tym w części Wyzwania). Rysunek: wziąć z dokumentacji producenta lub narysować własny.
]

== E-ink: zalety i wady

#grid(
  columns: (1fr, 1fr),
  gutter: 1.5em,
  [
    *Zalety*
    - Minimalny pobór energii w stanie spoczynku
    - Czytelność w pełnym słońcu
    - Brak podświetlenia, wygodny dla oczu
    - Wygląd "papierowy", nie rozprasza
  ],
  [
    *Wady*
    - Wolne odświeżanie: od ułamka sekundy do kilku sekund
    - Efekt "ducha" (_ghosting_)
    - Ograniczona skala szarości i kolorów
    - Wrażliwość na temperaturę
    - Niemożliwe animacje i wideo
  ],
)

#speaker-note[
  (ok. 1,5 min) Dla nas wady nie są wadami: prognoza i wiadomości zmieniają się raz na kilkanaście minut. To klasyczny przykład dopasowania technologii do zastosowania. Dla gry czy wideo e-ink byłby fatalny.
]

== Porównanie z innymi wyświetlaczami

#text(size: 0.65em)[
  #table(
    columns: (auto, 1fr, 1fr, 1fr, 1fr),
    align: left,
    [*Technologia*], [*Energia*], [*W słońcu*], [*Szybkość*], [*Typowe użycie*],
    [E-ink], [prąd tylko przy zmianie obrazu], [bardzo dobra], [wolna], [czytniki, etykiety cenowe, dashboardy],
    [LCD / TFT], [stały, podświetlenie], [słaba lub średnia], [szybka], [interfejsy, panele HMI],
    [OLED], [zależy od treści, bez podświetlenia], [średnia], [bardzo szybka], [wearables, małe wyświetlacze],
    [LCD znakowy / segmentowy], [niska], [zależy od wersji], [szybka], [proste wskaźniki, liczniki],
    [Memory LCD], [bardzo niska], [dobra], [szybka], [zegarki, urządzenia przenośne],
    [Matryca LED], [wysoka], [bardzo dobra], [szybka], [tablice informacyjne],
  )
]

#speaker-note[
  (ok. 2 min) Tabela jest jakościowa celowo. Liczby zależą od konkretnego modelu. Przejść po wierszach, zatrzymać się na OLED (uwaga na wypalanie przy statycznym obrazie) i na Memory LCD jako ciekawej alternatywie: niskie zużycie energii, szybkie odświeżanie, ale mała skala i wyższa cena. Dla nas liczy się energia i czytelność przy stałym obrazie.
]

== E-ink w praktyce: kiedy tak, kiedy nie

#grid(
  columns: (1fr, 1fr),
  gutter: 1.5em,
  [
    *Dobre dopasowanie*
    - Czytniki e-booków
    - Elektroniczne etykiety cenowe
    - Dashboardy, kalendarze, rozkłady
    - Wszystko, co zmienia się rzadko
  ],
  [
    *Złe dopasowanie*
    - Interfejsy z płynną interakcją
    - Gry, wideo
    - Sterowanie w czasie rzeczywistym
    - Aplikacje wymagające pełnej palety kolorów
  ],
)

#speaker-note[
  (ok. 1 min) Krótki przystanek: reguła kciuka. Pytanie do publiczności: czy ktoś ma czytnik e-booków? Zwykle tak, wtedy łatwo o odniesienie.
]

== ESP32: co to jest

- Rodzina mikrokontrolerów firmy *Espressif*, zintegrowane Wi-Fi i Bluetooth
- Wiele wariantów: ESP32, S2, S3, *C3*, C6 i inne (różne rdzenie, radia, pamięci)
- Tani, szeroko dostępny, ogromna społeczność
- Wspiera kilka ekosystemów: ESP-IDF, Arduino, MicroPython, Rust

#speaker-note[
  (ok. 1,5 min) Dla kogoś, kto miał jeden przedmiot o embedach: mikrokontroler to mały komputer w jednym układzie: procesor, pamięć, peryferia. ESP32 wyróżnia się tym, że radio Wi-Fi jest w środku, więc nie trzeba dokładać modułu. Dlatego stał się standardem w IoT dla hobbystów i coraz częściej w produktach.
]

== Dlaczego ESP32-C3U

- Rdzeń *RISC-V* 32-bit, do 160 MHz, 400 kB SRAM
- Wi-Fi 2,4 GHz (802.11 b/g/n) i Bluetooth LE
- Wi-Fi jako *punkt dostępu* (AP) *i* klient, czyli dwa tryby pracy potrzebne w naszym projekcie
- Niski pobór energii, tryby uśpienia
- Wsparcie dla *Rusta* (`no_std`, `esp-hal`) rozwijane przez samego producenta
- Oznaczenie *U*: #ph[co oznacza w naszym module, np. złącze anteny zewnętrznej?]

#speaker-note[
  (ok. 2 min) Kluczowy argument: obsługa trybu AP. Wybór układu był podyktowany m.in. możliwością wystawienia własnej sieci Wi-Fi, o czym więcej w części Wyzwania. C3 jest jednordzeniowy, co ma konsekwencje dla współbieżności (też tam). RISC-V to otwarta architektura zestawu instrukcji, Espressif przeszedł na nią w nowszych układach. #ph[dodać własną motywację zespołu, np. cena, dostępność, wcześniejsze doświadczenia]. Dane liczbowe warto jeszcze raz sprawdzić w datasheecie.
]

== Porównanie z innymi platformami

#text(size: 0.62em)[
  #table(
    columns: (auto, 1fr, 1fr, 1fr),
    align: left,
    [*Platforma*], [*Co to jest*], [*Wi-Fi*], [*Uwagi*],
    [Arduino (klasyczne AVR)], [8-bitowy MCU, prosty ekosystem], [brak, trzeba dołożyć moduł], [mało pamięci, świetne do nauki],
    [Raspberry Pi (SBC)], [komputer z Linuksem], [tak], [wyższy pobór energii, długi start, nadmiarowy do tego zadania],
    [Raspberry Pi Pico W], [MCU RP2040 z modułem Wi-Fi], [tak], [dobre wsparcie MicroPython, podobna kategoria co ESP32],
    [STM32], [rodzina ARM Cortex-M], [zwykle przez zewnętrzny moduł], [popularne w przemyśle, bogate peryferia],
    [ESP32-C3], [MCU RISC-V z Wi-Fi i BLE], [wbudowane, AP i klient], [jednordzeniowy, niewielka pamięć, uśpienie],
  )
]

#speaker-note[
  (ok. 2 min) Rozróżnić trzy kategorie: (1) mikrokontrolery, (2) płytki rozwojowe i ekosystemy (Arduino to głównie środowisko, nie jeden układ), (3) komputery jednopłytkowe (Raspberry Pi). Raspberry Pi to w pewnym sensie "za duże" do zadania "pokaż tekst raz na 15 minut": cały system operacyjny, więcej prądu, mniej deterministyczne. Pico W to najbliższy konkurent. Decyzja po stronie ESP32-C3 to kompromis, nie "jedyna słuszna droga".
]

= Przegląd rynku

== TRMNL

- Wyświetlacz e-ink 7,5", 800×480, 1-bit (czarno-biały), bateria
- Cena urządzenia: około 139 USD (OG), większy model X (10,3") około 219 USD
- Model działania: urządzenie *cyklicznie budzi się*, pobiera gotowy obraz z serwera i wraca do uśpienia
- Biblioteka kilkuset wtyczek (kalendarz, pogoda, transport...), otwarty firmware
- Zalecane przez twórców podejście: komunikacja jednokierunkowa, urządzenie samo pyta serwer

#ph-fig([zdjęcie TRMNL z oficjalnej strony / własne zdjęcie, uwaga na prawa autorskie], h: 4cm)

#speaker-note[
  (ok. 2 min) Dane o specyfikacji i cenach: recenzje i sklepy z 2025-2026 (m.in. The Gadgeteer, SparkFun, sklep TRMNL), do ponownego zweryfikowania przed prezentacją, bo ceny się zmieniają. Najciekawsze: *architektura*. Urządzenie jest głupie, całe renderowanie robi serwer (chmura TRMNL lub własny). Zaleta: prosta, tania płytka i łatwa rozbudowa. Wada: zależność od serwera. My robimy inaczej: pobieranie i przetwarzanie dzieje się na mikrokontrolerze. #ph[potwierdzić nasze założenia]
]

== Seeed XIAO 7.5" ePaper Panel

- Panel monochromatyczny 7,5", 800×480
- Sterowany przez *XIAO ESP32-C3*, czyli ta sama rodzina układów co u nas
- Bateria 2000 mAh, producent podaje około 3 miesięcy pracy w trybie głębokiego uśpienia
- Obudowa drukowana 3D, USB-C
- Współpracuje z Home Assistant (ESPHome), Arduino i *z TRMNL*
- Cena około 80 USD (wg zestawienia recenzenta)

#speaker-note[
  (ok. 1,5 min) Z wiki Seeed. To jest najbliższy sprzętowo kuzyn naszego projektu: ten sam układ, podobny wyświetlacz. Różnica: oni dostarczają sprzęt jako platformę do innych aplikacji, my budujemy pełne rozwiązanie od zera. Cenę podać z zastrzeżeniem. Pochodzi z porównania recenzenta i może się różnić w sklepie producenta.
]

== Inne rozwiązania

- *Inkplate* (Soldered, dawniej e-radionica): ESP32, 6" 800×600, panel odzyskiwany z czytników, odświeżanie pełne około 1,3 s, częściowe około 0,26 s, cena około 110 EUR
- *Pimoroni Inky Frame 7.3"*: Raspberry Pi Pico 2 W, 7 kolorów, 800×480, MicroPython, cena około 95 USD
- *Waveshare i inni*: surowe panele z płytkami sterującymi, dla tych, którzy chcą zbudować wszystko sami
- Rozwiązania społeczności: dashboardy na odzyskanych czytnikach Kindle, projekty na Raspberry Pi

#speaker-note[
  (ok. 2 min) Pokazać różnorodność: różne kompromisy (kolor vs szybkość odświeżania, MicroPython vs Arduino/C++). Inkplate ciekawy ekologicznie (recykling paneli). Pimoroni: kolor, ale kolorowy e-ink odświeża się wyraźnie wolniej. Ceny orientacyjne, zweryfikować. Warto pokazać 1-2 zdjęcia. #ph[dodać zdjęcia, zachować atrybucję]
]

== Co wyróżnia nasz projekt

#text(size: 0.65em)[
  #table(
    columns: (auto, 1fr, 1fr, 1fr),
    align: left,
    [*Cecha*], [*TRMNL*], [*Seeed XIAO 7.5"*], [*Nasz projekt*],
    [Układ], [ESP32], [ESP32-C3], [ESP32-C3U],
    [Źródło treści], [serwer pośredniczący], [zależy od oprogramowania (ESPHome, TRMNL, Arduino)], [bezpośrednio z kanałów RSS na urządzeniu #ph[potwierdzić]],
    [Konfiguracja], [przez aplikację / chmurę], [zależna od oprogramowania], [własny punkt dostępowy i strona WWW],
    [Język], [C/C++ (firmware)], [Arduino / ESPHome], [*Rust* (`no_std`)],
  )
]

#speaker-note[
  (ok. 1,5 min) Nie twierdzimy, że robimy lepiej. Pokazujemy, gdzie leży różnica: (1) architektura bez serwera pośredniczącego, (2) Rust na mikrokontrolerze, (3) własna ścieżka konfiguracji. Wiersze "Język" dla TRMNL i Seeed należy zweryfikować. Informacje oparte na ogólnym przeglądzie, nie na analizie kodu. #ph[uzupełnić po sprawdzeniu źródeł]
]

= Wyzwania

== Wyzwanie 1: jak nie zepsuć wyświetlacza e-ink

- Panel e-ink *trwale zapamiętuje* obraz, bo jest bistabilny
- Stałe pole elektryczne i brak odświeżania mogą prowadzić do *wypalenia / degradacji*
- Przesadne odświeżanie też szkodzi: panel ma *limit częstotliwości zmian*
- Producenci podają konkretne zalecenia dla każdego panelu #ph[sprawdzić w datasheecie naszego modelu]

#speaker-note[
  (ok. 2 min) Brzmi paradoksalnie: wyświetlacz, który nie zużywa prądu, może się zepsuć od zbyt rzadkiego lub zbyt częstego użycia. Typowe zalecenia producentów (np. Waveshare, Good Display) to m.in.: odstęp między odświeżeniami rzędu minut, przynajmniej jedno odświeżenie na dobę, uśpienie panelu po odświeżeniu i przechowywanie z wyczyszczonym ekranem. Konkretne liczby zależą od modelu, więc w prezentacji podajemy zasadę, nie liczbę. #ph[wstawić liczby z naszego datasheetu]
]

== Zasady higieny

- *Pełne vs częściowe odświeżanie*: częściowe szybsze, ale zostawia ducha, co jakiś czas pełne
- *Uśpić panel* po każdym odświeżeniu, nie zostawiać go pod napięciem
- *Odstęp między odświeżeniami*: wymuszać programowo, a nie ufać pętli głównej
- *Temperatura* i zasilanie: sekwencja włączania i wyłączania zgodna z kartą katalogową
- Obraz nie powinien leżeć tygodniami bez zmian

#speaker-note[
  (ok. 2 min) Kluczowa myśl inżynierska: *ochrona sprzętu powinna siedzieć w kodzie, który jej nie da się obejść*. Jedna funkcja, która wszystko odświeża i pilnuje limitów, zamiast rozproszonych wywołań. W Rust można to wyrazić w systemie typów (np. stan panelu jako typ). To zostawiamy na część 2. Tu tylko idea. #ph[dodać własne doświadczenia, jeśli były problemy ze sprzętem]
]

== Wyzwanie 2: Wi-Fi i bezpieczeństwo

#ph[do konsultacji z promotorem: zakres i głębokość tego fragmentu]

- Nasz ESP32 jest jednocześnie *klientem* (łączy się z Internetem) i *punktem dostępu* (konfiguracja)
- AP to *wystawiona powierzchnia ataku*: każdy w zasięgu może spróbować się połączyć
- Czego się obawiać: słabe lub domyślne hasło, konfiguracja po HTTP bez szyfrowania, brak uwierzytelnienia strony, błędy w parsowaniu danych z sieci
- Dodatkowe ograniczenia: mało pamięci i mocy, więc TLS na MCU kosztuje

#speaker-note[
  (ok. 2 min) Przy braku klawiatury wystawienie AP jest najprostszą ścieżką konfiguracji, ale płaci się za to ryzykiem. Mikrokontroler sam w sobie jest wrażliwy: nie ma izolacji procesów, system operacyjny nie istnieje albo jest minimalny, a jeden błąd w obsłudze pakietu to potencjalnie pełna kontrola. Cały ten slajd jest do ustalenia z promotorem: ile mówić o konkretnych rozwiązaniach, a ile o samych zagrożeniach.
]

== Jak ograniczać ryzyko

- AP z *WPA2/WPA3*, unikalnym hasłem urządzenia (nie jedno hasło dla wszystkich)
- AP *tylko w trybie konfiguracji*, z limitem czasu, potem wyłączony
- *Walidacja wejścia*: limity rozmiarów, brak zaufania do danych z formularza i z RSS
- Połączenia wychodzące po *HTTPS* z weryfikacją certyfikatów, jeżeli zasoby na to pozwalają
- Sekrety (hasło do Wi-Fi) w pamięci flash: rozważyć szyfrowanie flash i secure boot
- Język bezpieczny pamięciowo (Rust) usuwa całą klasę błędów, ale nie wszystkie

#speaker-note[
  (ok. 2 min) To lista wzorców, nie opis naszej implementacji. Hasło z naklejki zamiast hasła fabrycznego "admin" jest kluczowe: historia IoT zna masowe ataki na urządzenia z domyślnymi hasłami. Rust: przepełnienia bufora i use-after-free nie przejdą bez `unsafe`, ale logika biznesowa i protokoły dalej mogą mieć luki. #ph[które z tych wzorców faktycznie stosujemy: uzgodnić przed częścią 2]
]

== Wyzwanie 3: zrównoleglenie w systemach wbudowanych

- Urządzenie robi kilka rzeczy *naraz*:
  - pobiera dane z sieci (*fetching*), czyli długie czekanie na I/O
  - utrzymuje *pętlę sterującą* (odświeżanie ekranu, harmonogram, tryby zasilania)
  - obsługuje stronę konfiguracyjną
- Pobieranie z sieci nie może zablokować reszty
- ESP32-C3 ma *jeden rdzeń*, więc współbieżność, nie równoległość

#speaker-note[
  (ok. 2 min) Rozróżnić *współbieżność* (concurrency: wiele zadań w toku) i *równoległość* (parallelism: dosłownie jednocześnie). Na jednym rdzeniu mamy tylko pierwsze. Przykład: wysyłamy zapytanie HTTP i przez sekundę czekamy na odpowiedź. W tym czasie procesor nie powinien stać, ale też nie powinniśmy przegapić zdarzenia z innego źródła (np. przychodzące połączenie z telefonu).
]

== Modele współbieżności na mikrokontrolerze

#text(size: 0.65em)[
  #table(
    columns: (auto, 1fr, 1fr),
    align: left,
    [*Podejście*], [*Idea*], [*Wady i zalety*],
    [Pętla główna (_super loop_)], [jedna pętla, odpytywanie zdarzeń], [proste i przewidywalne, ale trudne przy wielu zadaniach i blokujące operacje],
    [Przerwania], [obsługa zdarzeń sprzętowych w odrębnym kontekście], [szybka reakcja, ale trudne w debugowaniu, ryzyko wyścigów],
    [RTOS (np. FreeRTOS)], [wątki z priorytetami i wywłaszczaniem], [znany model, ale każdy wątek ma swój stos, więc kosztuje pamięć],
    [Async / await (np. Embassy)], [zadania kooperacyjne, bez wywłaszczania], [lekkie, bez osobnych stosów, ale zadanie musi samo oddać sterowanie],
    [Rdzenie i procesory], [dosłowna równoległość na wielu rdzeniach], [niedostępne w C3, w innych układach ESP32 tak],
  )
]

#speaker-note[
  (ok. 2,5 min) Przejść po tabeli od najprostszego do najbardziej wyrafinowanego. Zamiast wybierać zwycięzcę, pokazać kompromis: prostota vs skalowalność vs koszt pamięci. Async w Rust jest szczególnie ciekawy, bo kompilator zamienia funkcje `async` w maszyny stanów, bez osobnych stosów. #ph[powiedzieć, co wybraliśmy i dlaczego, jeśli uznamy za stosowne na tym etapie, lub zostawić na część 2]
]

== Typowe pułapki

- *Zasoby współdzielone*: magistrala SPI wyświetlacza, radio Wi-Fi, pamięć
- *Wyścigi i zakleszczenia*: dwa zadania zmieniają tę samą konfigurację
- *Głodzenie*: jedno zadanie nigdy nie oddaje sterowania, a _watchdog_ resetuje urządzenie
- *Pamięć*: sterta (_heap_) ma ograniczony rozmiar, fragmentacja i nieudane alokacje
- *Energia*: czy uśpić płytkę w trakcie pobierania? Jak pogodzić Wi-Fi z trybami oszczędzania?

#speaker-note[
  (ok. 2 min) Zamknięcie sekcji. Wszystkie trzy wyzwania łączy jedna myśl: na mikrokontrolerze *nie ma siatki bezpieczeństwa*. Zły dostęp do pamięci, zła kolejność operacji czy za częste odświeżanie przekładają się na realny, fizyczny skutek. Stąd nacisk na język i konstrukcje, które pomagają nie robić głupich błędów.
]

= Podsumowanie

== Co z tego wynika

- E-ink + ESP32 to dobre dopasowanie do zadania "pokaż rzadko zmieniające się informacje"
- Podobne produkty istnieją, a nasz projekt jest *ćwiczeniem inżynierskim*, nie próbą podboju rynku
- Trzy trudne tematy: *ochrona wyświetlacza*, *bezpieczeństwo Wi-Fi*, *współbieżność*
- W części 2: architektura, implementacja i wyniki

#speaker-note[
  (ok. 1 min) Powtórzyć najważniejsze trzy zdania i zaprosić na część 2.
]

== Źródła

- TRMNL: trmnl.com; recenzje i sklepy (The Gadgeteer, SparkFun, 2025-2026)
- Seeed Studio: wiki.seeedstudio.com/xiao_075inch_epaper_panel
- Inkplate 6: docs.soldered.com/inkplate/6/overview
- Pimoroni Inky Frame 7.3": opis produktu i artykuł electronics-lab.com
- Dokumentacja Espressif: ESP32-C3 datasheet, esp-hal
- #ph[dopisać źródła dla e-ink (E Ink Corp., Waveshare, Good Display) i dla bezpieczeństwa IoT]

#speaker-note[
  Ceny i parametry produktów sprawdzić ponownie tuż przed seminarium.
]

== Pytania?

#align(center + horizon)[
  *Dziękujemy za uwagę*
]
