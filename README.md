# Maus-Utility

Autoclicker, Dauerklick, Farb-Trigger, Makros und Cursor-Info für Windows 10/11. Das Programm verwendet reines Rust mit der Windows-API und GDI+ und keine externen Crates.

## Kompilieren

```
cargo build --release
```

Die fertige Datei liegt danach unter `target\release\maus-utility.exe`.

## Bedienung

| Bereich | Was es macht | Standard-Hotkey |
|---|---|---|
| Autoclicker | Klickt mit Mouse 1 oder 2 im eingestellten Intervall, mit optionalem Zufall-Offset | F6 |
| Dauerklick | Hält Mouse 1 oder 2 gedrückt, bis du erneut drückst oder die eingestellte Zeit abläuft | F7 |
| Farb-Trigger | Prüft ein Pixel und klickt, sobald dort die Zielfarbe erscheint | F9 |
| Makro | Eigene Abläufe aus Klicks, Wartezeiten, Farbprüfungen und Sprüngen, optional geloopt | F8 |
| Cursor-Info | Zeigt X/Y, RGB und Hex unter dem Cursor; Strg+C merkt den Wert | – |

- **Maustaste:** Mouse 1 und Mouse 2 sind eigene Buttons. Der aktive Button ist orange hinterlegt.
- **Einheiten:** Für Intervall, Zufall-Offset, Cooldown, Prüfintervall, Loslassen nach und die Zeiten im Makro wählst du per Dropdown zwischen Millisekunden, Sekunden, Minuten und Stunden. Kommazahlen wie `1,5` gehen auch.
- **Zufall-Offset:** Bei 100 ms Intervall und 20 ms Offset liegt jede Pause zufällig zwischen 80 und 120 ms.
- **Farb-Trigger:**
  - Position und Zielfarbe trägst du am einfachsten aus der Merkliste ein (siehe Cursor-Info).
  - *Toleranz* erlaubt Abweichungen je Farbkanal.
  - *Klick-Offset* klickt jedes Mal auf ein zufälliges Pixel im Bereich ±N Pixel um die Position.
  - *Nur 1× je Treffer* klickt einmal, sobald die Farbe erscheint, und erst wieder, nachdem sie verschwunden war. Ohne den Haken wird in jedem Prüfintervall geklickt.
  - *Cursor zurück* setzt den Mauszeiger nach dem Klick an seine alte Position.
  - *Cooldown* ist eine Pause nach jedem Klick, bevor wieder geprüft wird. 0 bedeutet keine Pause.
  - *Prüfstelle* ist das Pixel, dessen Farbe überwacht wird. *Klickziel* ist die Stelle, an der dann geklickt wird. Bleibt das Klickziel leer, wird an der Prüfstelle geklickt.
  - *Andere Klicker pausieren* hält Autoclicker und Makro an, sobald die Farbe erkannt wird. Sie laufen erst nach dem Klick und dem Cooldown weiter.
- **Dauerklick:** Bei *Loslassen nach* lässt das Programm die Taste nach dieser Zeit automatisch los. 0 bedeutet, sie bleibt gedrückt, bis du den Hotkey erneut drückst.
- **Positionen:** Autoclicker, Dauerklick und Makro-Klicks haben ein X/Y-Feld. Sind beide leer, wird an der aktuellen Cursorposition geklickt. Ist eine Position eingetragen, springt der Cursor zum Klicken dorthin und danach wieder zurück. Beim Dauerklick bleibt er dort, bis losgelassen wird.
- **Cursor-Info:**
  1. Schalter „Anzeigen“ einschalten.
  2. Maus auf die gewünschte Stelle bewegen und **Strg+C** drücken. Der Wert landet in der Merkliste.
  3. Mit „Übernehmen“ überträgst du einen Eintrag als Position an Autoclicker, Dauerklick, das Makro oder als Prüfstelle bzw. Klickziel an den Farb-Trigger. Ein Doppelklick setzt direkt die Prüfstelle und Farbe des Farb-Triggers.

  Strg+C kopiert in anderen Programmen weiterhin ganz normal.
- **Makro:** Standardmäßig eingeklappt. Mit dem Button oben rechts blendest du es ein und aus. Oben steht die Schrittliste, darunter der Schritt-Editor.
  - **Schritt-Typen:**
    | Typ | Was passiert |
    |---|---|
    | Klick | Klick mit Mouse 1 oder 2, an der Position oder (leer) an der Cursorposition. *Klick-Offset* ±N klickt jedes Mal zufällig bis zu N Pixel daneben |
    | Warten | Pause für die eingestellte Zeit. *Zufall-Offset* ±Zeit verlängert oder verkürzt jede Pause zufällig, z. B. 1 s ± 200 ms = 0,8 bis 1,2 s |
    | Wenn Farbe, sonst springen | Prüft einmal die Farbe an X/Y. Passt sie, geht es mit dem nächsten Schritt weiter, sonst mit Schritt *Sonst zu* |
    | Auf Farbe warten | Wartet, bis die Farbe erscheint. *Max. warten* 0 = unbegrenzt; bei Zeitablauf geht es zu Schritt *Bei Ablauf* |
    | Zu Schritt springen | Springt zu der angegebenen Schrittnummer |
  - Eine Schrittnummer von 0 bedeutet „einfach weiter“. Eine Nummer hinter dem letzten Schritt beendet den Durchlauf.
  - **Bearbeiten:** Ein Klick auf einen Schritt lädt ihn in den Editor. *+ Als neuer Schritt* fügt den Editor-Inhalt hinter dem markierten Schritt ein, *Auswahl ändern* überschreibt den markierten Schritt. Beim Verschieben, Einfügen und Entfernen werden Sprungziele automatisch angepasst.
  - Über *Übernehmen* in der Merkliste kommen Position bzw. Position und Farbe direkt in den Editor.
  - *Speichern …* und *Laden …* sichern Makros als `.mausmakro`-Datei, inklusive Loop-Einstellung und Durchläufen. Alte `.mausfolge`-Dateien lassen sich weiterhin laden. Das Format ist Klartext:
    ```
    click 1 500 600                # Mouse 1 an X/Y (ohne X/Y: an der Cursorposition)
    click 1 500 600 off=5          # dasselbe mit Klick-Offset ±5 Pixel
    wait 1000                      # Millisekunden
    wait 1000 rnd=200              # 1000 ms mit Zufall-Offset ±200 ms
    ifcolor 700 300 336699 10 1    # X Y Farbe Toleranz Sonst-zu-Schritt
    waitcolor 650 650 FFFFFF 10 5000 0   # X Y Farbe Toleranz Max-ms Bei-Ablauf-Schritt
    goto 1
    ```
- **Ein/Aus-Schalter:** Jede Karte (Autoclicker, Dauerklick, Farb-Trigger, Makro) hat oben rechts einen Schalter. Ausgeschaltet wird die Funktion gestoppt, ihr Hotkey freigegeben (die Taste funktioniert dann wieder normal in anderen Programmen) und die Karte gesperrt.
- **Hinweise:** Der Button unten links klappt eine Kurzhilfe auf. Sie ist standardmäßig eingeklappt. Ist ein Hotkey belegt, erscheint die Warnung trotzdem daneben.
- **Hotkeys ändern:** Ins Hotkey-Feld klicken und die neue Taste drücken, auch mit Strg, Alt oder Umschalt. Esc bricht ab, Entf entfernt den Hotkey.
- **Hell/Dunkel:** Der Mond- bzw. Sonnen-Button oben rechts wechselt das Design. Beim Start folgt das Programm der Windows-Einstellung.
- **Tray:** Minimieren schickt das Programm in den Infobereich. Ein Linksklick auf das Symbol öffnet das Fenster wieder, ein Rechtsklick öffnet das Menü. Das X beendet das Programm.

Simulierte Klicks lösen die Buttons des Programms selbst nicht aus. So stoppt sich zum Beispiel der Autoclicker nicht versehentlich selbst.

## Hinweis

Manche Spiele mit Anti-Cheat erkennen simulierte Eingaben oder Tastatur-Hooks (für Strg+C). Wo Autoclicker verboten sind, kann ihr Einsatz zu einem Bann führen.

## Lizenz

MIT – siehe [LICENSE](LICENSE).

## Releases

Fertige EXE-Dateien findest du unter **Releases**. Ein neues Release entsteht automatisch, sobald die Versionsnummer in `Cargo.toml` erhöht und auf `main` gepusht wird. GitHub baut die EXE dann selbst (Workflow `.github/workflows/release.yml`).

## Beispiel: Klicken, bis eine Farbe erscheint, dann bestätigen

| Nr. | Schritt |
|---|---|
| 1 | Klick Mouse 1 auf die Arbeitsstelle |
| 2 | 1 s warten |
| 3 | Wenn die Zielfarbe an der Prüfstelle erscheint, weiter, sonst zurück zu Schritt 1 |
| 4 | 0,5 s warten |
| 5 | Klick auf den ersten Button |
| 6 | 1 s warten |
| 7 | Klick auf den Bestätigen-Button |
| 8 | 1 s warten, dann beginnt das Makro von vorn (Loopen an) |

Die Wartezeiten sind die Pausen zwischen den Klicks. Statt fester Wartezeit vor der Bestätigung kann Schritt 6 auch ein „Auf Farbe warten“ sein.
