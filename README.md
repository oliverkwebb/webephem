# deskephem - Modular CLI Astronomy

```
M""""""'YMM                   dP       MM""""""""`M          dP
M  mmmm. `M                   88       MM  mmmmmmmM          88
M  MMMMM  M .d8888b. .d8888b. 88  .dP  M`      MMMM 88d888b. 88d888b. .d8888b. 88d8b.d8b.
M  MMMMM  M 88ooood8 Y8ooooo. 88888"   MM  MMMMMMMM 88'  `88 88'  `88 88ooood8 88'`88'`88
M  MMMM' .M 88.  ...       88 88  `8b. MM  MMMMMMMM 88.  .88 88    88 88.  ... 88  88  88
M       .MM `88888P' `88888P' dP   `YP MM        .M 88Y888P' dP    dP `88888P' dP  dP  dP
MMMMMMMMMMM                            MMMMMMMMMMMM 88
                                                    dP
```

```
$ deskephem moon phase # Werewolf early warning system
🌒 Waxing Crescent (5.0%)
$ deskephem Venus equ # Coordinates of Venus
23h50m21.9s +7°55'37.4"
$ deskephem -d 1781-03-13 -l 53n -L 1.8w Uranus horiz,magnitude # Location and brightness of Uranus at William Herschel's first observation
278°45'42.91" 23°22'1.52" 5.60
```

deskephem is a CLI astronomy calculator for the properties of celestial objects such as the moon, planets, stars, and sun:

* Coordinates in the sky (equatorial, horizontal, ecliptic)
* Rise and set times
* Phase (Emoji, Illuminated Fraction, Name)
* Brightness (Magnitude)
* Distance
* Angular Size

It supports outputs in json, csv, and plain text. It can also generate tables of output:

```
$ time deskephem -E 1600-01-01,1mon,9999-06-01 mars ecliptic # Query is ran ~100k times
         date                   ecliptic
===================================================
 1600-01-01T00:00:00  143°04′10.6″ +3°48′36.4″
 1600-02-01T00:00:00  133°03′30.5″ +4°31′51.0″
 1600-03-01T00:00:00  125°00′55.1″ +3°53′57.8″
[...]
1.89 user 0.20 system
```

The basic usage for deskephem is:
```
deskephem [-l Latitude] [-L longitude] [-d Date] [-T Output-Format] [-E [Start Date],[Interval],[End Date]] object property
```
