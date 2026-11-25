# Plan prezentacji seminaryjnej: RSS i wizualizacja ekranu w `desk-display`

## Cel i główna myśl

30-minutowe wystąpienie o idei przetwarzania danych RSS przy ograniczonych zasobach oraz zdalnego podglądu monochromatycznego framebufferu. Główną motywacją dla podglądu jest sytuacja, w której ekran jest uszkodzony: jeśli samo urządzenie nadal działa, zewnętrzny podgląd może pomóc sprawdzić, jaki obraz przygotowuje system. Dane RSS można przetwarzać stopniowo, a zawartość framebufferu udostępnić zewnętrznemu narzędziu przez HTTP. To dwa osobne przepływy w projekcie — RSS nie jest obecnie przekazywany do framebufferu.

**Odbiorcy:** techniczna publiczność seminaryjna, bez wymogu wcześniejszej znajomości repozytorium.

**Materiały:** diagram przepływu RSS, schemat dwóch buforów oraz przygotowane screenshoty rzeczywistego okna `tools/display-viewer/main.py`. Prezentacja nie zakłada demonstracji na żywo ani dostępu do urządzenia/AP podczas seminarium.

## Agenda — 30 minut

| Czas | Temat | Treść i teza |
|---|---|---|
| 0–3 min | Kontekst projektu | `desk-display`, firmware `no_std` na ESP32-C3 oraz motywacja: możliwość sprawdzenia obrazu, gdy ekran jest uszkodzony. |
| 3–8 min | Droga danych RSS | `src/wifi/rss.rs`: pobranie skonfigurowanych kanałów i stopniowe przekazywanie odpowiedzi do parsera. |
| 8–13 min | Idea parsera | `rss-parser`: przetwarzanie danych w miarę ich napływania oraz przechowywanie wyniku w strukturach o ograniczonej pojemności. |
| 13–16 min | Sprawdzanie logiki | Osobne crate’y umożliwiają testowanie wybranych elementów na komputerze, bez płytki. |
| 16–20 min | Problem uszkodzonego ekranu | Dlaczego zewnętrzny podgląd może być przydatny, gdy panel nie pokazuje obrazu, a urządzenie nadal działa; framebuffer i `GET /display`. |
| 20–25 min | Idea zewnętrznego podglądu | `tools/display-viewer/main.py`: pobranie obrazu przez HTTP, dekodowanie i wyświetlenie w oknie; screenshoty zamiast pokazu na żywo. |
| 25–27 min | Zakres rozwiązania | Podgląd pomaga ocenić zawartość framebufferu, ale nie naprawia ani nie zastępuje ekranu; RSS i framebuffer to osobne przepływy. |
| 27–30 min | Podsumowanie i pytania | Główna idea, kompromis między przewidywalnym użyciem pamięci a rozmiarem przechowywanych danych, pytania. |

**Kontrola czasu:** 3 + 5 + 5 + 3 + 4 + 5 + 2 + 3 = **30 minut**.

## Rozwinięcie wystąpienia

### 0–3 min — kontekst projektu

- Krótko przedstawić rolę `desk-display` i osadzić ją w realiach firmware `no_std` dla ESP32-C3.
- Zapowiedzieć dwa wątki: przetwarzanie RSS przy ograniczonej pamięci oraz podgląd framebufferu, którego główną motywacją jest diagnostyka przy uszkodzonym ekranie.
- Zasygnalizować, że są to osobne części projektu, a nie gotowy przepływ od artykułu RSS do obrazu na ekranie.

### 3–8 min — droga danych RSS

- Pokazać `src/wifi/rss.rs`: urządzenie pobiera skonfigurowane kanały i przekazuje odebrane dane parserowi.
- Wyjaśnić ideę przetwarzania strumieniowego: parser może pracować w trakcie odbioru, bez wcześniejszego gromadzenia całego dokumentu XML w jednym dużym buforze.
- Na diagramie pokazać prosty ciąg: źródło RSS → pobranie → parser → wynik.

### 8–13 min — idea parsera i pamięć

- Omówić ideę parsera w `rss-parser/src/parser.rs`: dane napływają stopniowo, a parser zachowuje stan potrzebny do zbudowania wyniku.
- W `rss-parser/src/types.rs` wynik i jego pola mają ograniczone pojemności. Przedstawić to jako sposób na bardziej przewidywalne wykorzystanie pamięci, bez wyliczania wszystkich limitów.
- Podkreślić kompromis: ograniczone struktury pomagają kontrolować zasoby, ale ograniczają też ilość przechowywanych danych.
- Wspomnieć, że testy sprawdzają logikę parsera; nie są pomiarem zużycia pamięci całego firmware na urządzeniu.

### 13–16 min — sprawdzanie logiki

- Pokazać podział na crate’y: logikę `rss-parser` i `framebuffer` można testować na hoście, bez podłączonej płytki.
- Zaznaczyć, że takie testy pomagają sprawdzić wybrane komponenty, ale nie zastępują testów całego urządzenia.

### 16–20 min — problem uszkodzonego ekranu i framebuffer

- Przedstawić motywację: jeśli ekran jest uszkodzony, ale urządzenie nadal działa, zewnętrzny podgląd może pokazać, jaki obraz system przygotowuje.
- Wyjaśnić, że framebuffer przechowuje obraz monochromatyczny, a podwójne buforowanie pozwala przygotować kolejną klatkę oddzielnie od tej aktualnie udostępnianej.
- `GET /display` odczytuje front buffer i zwraca jego surowe bajty jako `application/octet-stream`.
- Zaznaczyć, że endpoint udostępnia obraz z framebufferu; nie pobiera ani nie przetwarza kanałów RSS.

### 20–25 min — zewnętrzny podgląd

- Przedstawić `tools/display-viewer/main.py`: narzędzie pobiera surowe dane przez HTTP, dekoduje piksele monochromatyczne i wyświetla obraz w Tkinterze.
- Wyjaśnić, jak taki podgląd mógłby pomóc odróżnić problem z panelem od problemu z obrazem przygotowywanym przez urządzenie.
- Pokazać przygotowane screenshoty zamiast uzależniać wystąpienie od dostępu do urządzenia lub sieci.
- Opisać pochodzenie screenshotów; jeśli pokazują obraz testowy, wyraźnie to zaznaczyć.

### 25–27 min — zakres rozwiązania

- Wyjaśnić, że podgląd framebufferu odpowiada na potrzebę diagnozowania sytuacji z uszkodzonym ekranem, ale sam ekranu nie naprawia ani nie zastępuje.
- Wyraźnie rozdzielić dwa aktualne elementy: RSS jest pobierany i parsowany, a framebuffer można udostępnić viewerowi.
- Zaznaczyć, że obecny kod nie łączy wyniku RSS z generowaniem klatki; połączenie tych części byłoby kolejnym krokiem rozwoju projektu.
- Nie przedstawiać screenshotu viewera jako dowodu, że urządzenie aktualizuje ekran danymi z RSS ani jako demonstracji naprawy uszkodzonego panelu.

### 27–30 min — podsumowanie i pytania

- Podsumować trzy tezy: RSS można przetwarzać stopniowo, ograniczone bufory wspierają przewidywalne wykorzystanie pamięci, a zdalny podgląd framebufferu może pomóc przy diagnozowaniu uszkodzonego ekranu.
- Wskazać jako dalszy kierunek połączenie przetwarzania RSS z generowaniem obrazu.
- Zostawić około dwóch minut na pytania.

## Przepływy do pokazania

**RSS:** skonfigurowany kanał → pobranie → stopniowe parsowanie → wynik.

**Podgląd:** front buffer → `GET /display` → bajty HTTP → `tools/display-viewer/main.py` → dekodowanie obrazu → okno Tkinter.

## Lista kontrolna przed wystąpieniem

- Zmierzyć czas przejścia przez agendę; suma segmentów musi pozostać równa 30 minutom.
- Przygotować czytelne screenshoty viewera i opisać, czy pokazują rzeczywisty obraz z urządzenia, czy przykład/test.
- Nie planować demonstracji na żywo ani nie uzależniać wystąpienia od dostępności urządzenia lub sieci.
- Utrzymać rozróżnienie między przepływem RSS a podglądem framebufferu; nie sugerować, że są już zintegrowane.
- Podkreślić, że podgląd pokazuje zawartość framebufferu i jest pomysłem diagnostycznym, nie naprawą ekranu.
- Podkreślić, że testy crate’ów sprawdzają wybrane komponenty, a nie całe firmware na sprzęcie.
