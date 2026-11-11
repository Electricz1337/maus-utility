# Maus-Utility

Autoclicker, Dauerklick, Farb-Trigger, Klickfolgen und Cursor-Info für Windows 10/11. Das Programm verwendet reines Rust mit der Windows-API und GDI+ und keine externen Crates.

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
| Klickfolge | Eigene Abfolge aus Klicks und Wartezeiten, optional geloopt | F8 |
| Cursor-Info | Zeigt X/Y, RGB und Hex unter dem Cursor; Strg+C merkt den Wert | – |

- **Maustaste:** Mouse 1 und Mouse 2 sind eigene Buttons. Der aktive Button ist orange hinterlegt.
- **Einheiten:** Für Intervall, Zufall-Offset, Cooldown, Prüfintervall, Loslassen nach und Wartezeit wählst du per Dropdown zwischen Millisekunden, Sekunden, Minuten und Stunden. Kommazahlen wie `1,5` gehen auch.
- **Zufall-Offset:** Bei 100 ms Intervall und 20 ms Offset liegt jede Pause zufällig zwischen 80 und 120 ms.
- **Farb-Trigger:**
  - Position und Zielfarbe trägst du am einfachsten aus der Merkliste ein (siehe Cursor-Info).
  - *Toleranz* erlaubt Abweichungen je Farbkanal.
  - *Klick-Offset* klickt jedes Mal auf ein zufälliges Pixel im Bereich ±N Pixel um die Position.
  - *Nur 1× je Treffer* klickt einmal, sobald die Farbe erscheint, und erst wieder, nachdem sie verschwunden war. Ohne den Haken wird in jedem Prüfintervall geklickt.
  - *Cursor zurück* setzt den Mauszeiger nach dem Klick an seine alte Position.
  - *Cooldown* ist eine Pause nach jedem Klick, bevor wieder geprüft wird. 0 bedeutet keine Pause.
- **Dauerklick:** Bei *Loslassen nach* lässt das Programm die Taste nach dieser Zeit automatisch los. 0 bedeutet, sie bleibt gedrückt, bis du den Hotkey erneut drückst.
- **Cursor-Info:**
  1. Schalter „Anzeigen“ einschalten.
  2. Maus auf die gewünschte Stelle bewegen und **Strg+C** drücken. Der Wert landet in der Merkliste.
  3. Mit „→ Farb-Trigger“ oder einem Doppelklick überträgst du einen Eintrag in den Farb-Trigger.

  Strg+C kopiert in anderen Programmen weiterhin ganz normal.
- **Klickfolge:** Standardmäßig eingeklappt. Mit dem Button oben rechts blendest du sie ein und aus. Neue Schritte landen hinter dem markierten Eintrag, der laufende Schritt ist hervorgehoben.
  - *Speichern …* und *Laden …* sichern Folgen als `.mausfolge`-Datei, inklusive Loop-Einstellung und Durchläufen. Das Format ist Klartext und lässt sich auch von Hand bearbeiten (`click 1`, `click 2`, `wait 1500` in ms).
- **Hinweise:** Der Button unten links klappt eine Kurzhilfe auf. Sie ist standardmäßig eingeklappt. Ist ein Hotkey belegt, erscheint die Warnung trotzdem daneben.
- **Hotkeys ändern:** Ins Hotkey-Feld klicken und die neue Taste drücken, auch mit Strg, Alt oder Umschalt. Esc bricht ab, Entf entfernt den Hotkey.
- **Hell/Dunkel:** Der Mond- bzw. Sonnen-Button oben rechts wechselt das Design. Beim Start folgt das Programm der Windows-Einstellung.
- **Tray:** Minimieren schickt das Programm in den Infobereich. Ein Linksklick auf das Symbol öffnet das Fenster wieder, ein Rechtsklick öffnet das Menü. Das X beendet das Programm.

Simulierte Klicks lösen die Buttons des Programms selbst nicht aus. So stoppt sich zum Beispiel der Autoclicker nicht versehentlich selbst.

## Hinweis

Manche Spiele mit Anti-Cheat erkennen simulierte Eingaben oder Tastatur-Hooks (für Strg+C). Wo Autoclicker verboten sind, kann ihr Einsatz zu einem Bann führen.

## Lizenz

MIT – siehe [LICENSE](LICENSE).
