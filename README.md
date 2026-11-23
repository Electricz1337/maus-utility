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
  - *Prüfstelle* ist das Pixel, dessen Farbe überwacht wird. *Klickziel* ist die Stelle, an der dann geklickt wird. Bleibt das Klickziel leer, wird an der Prüfstelle geklickt.
  - *Andere Klicker pausieren* hält Autoclicker und Klickfolge an, sobald die Farbe erkannt wird. Sie laufen erst nach dem Klick und dem Cooldown weiter.
- **Dauerklick:** Bei *Loslassen nach* lässt das Programm die Taste nach dieser Zeit automatisch los. 0 bedeutet, sie bleibt gedrückt, bis du den Hotkey erneut drückst.
- **Positionen:** Autoclicker, Dauerklick und Klickfolge haben ein X/Y-Feld. Sind beide leer, wird an der aktuellen Cursorposition geklickt. Ist eine Position eingetragen, springt der Cursor zum Klicken dorthin und danach wieder zurück. Beim Dauerklick bleibt er dort, bis losgelassen wird.
- **Cursor-Info:**
  1. Schalter „Anzeigen“ einschalten.
  2. Maus auf die gewünschte Stelle bewegen und **Strg+C** drücken. Der Wert landet in der Merkliste.
  3. Mit „Übernehmen“ überträgst du einen Eintrag als Position an Autoclicker, Dauerklick, Klickfolge oder als Prüfstelle bzw. Klickziel an den Farb-Trigger. Ein Doppelklick setzt direkt die Prüfstelle und Farbe des Farb-Triggers.

  Strg+C kopiert in anderen Programmen weiterhin ganz normal.
- **Klickfolge:** Standardmäßig eingeklappt. Mit dem Button oben rechts blendest du sie ein und aus. Neue Schritte landen hinter dem markierten Eintrag, der laufende Schritt ist hervorgehoben. Steht bei *Position* etwas, bekommen neue Klick-Schritte diese Position (`click 1 640 900` in der Datei).
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

## Releases

Fertige EXE-Dateien findest du unter **Releases**. Ein neues Release entsteht automatisch, sobald die Versionsnummer in `Cargo.toml` erhöht und auf `main` gepusht wird. GitHub baut die EXE dann selbst (Workflow `.github/workflows/release.yml`).

## Beispiel: Klicken, bis eine Farbe erscheint, dann woanders klicken

1. **Cursor-Info** einschalten und mit Strg+C drei Stellen merken: die Stelle für die Dauerklicks, das Pixel, das sich färbt, und den Button, der dann gedrückt werden soll.
2. **Autoclicker:** Den ersten Eintrag über „Übernehmen → Autoclicker: Position“ eintragen und ein Intervall wählen.
3. **Farb-Trigger:** Das Pixel über „Übernehmen → Farb-Trigger: Prüfstelle + Farbe“ setzen und den Button über „Übernehmen → Farb-Trigger: Klickziel“. „Andere Klicker pausieren“ einschalten und einen Cooldown einstellen, z. B. 2 Sekunden.
4. Zuerst den Farb-Trigger starten, dann den Autoclicker.
