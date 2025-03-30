# deskephem - CLI Astronomy Calculator

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

deskephem is a modular, idiomatic CLI astronomy calculator, which can get the properties of celestial objects like the moon, planets, stars, and sun.
These properties include:

* Coordinates in the sky (equatorial, horizontal, and azimuthal)
* Rise and set times
* Phase
* Brightness
* Distance
* Angular Size

```shell
$ deskephem Venus location # Gets the current coordinates of Venus
23h50m21.9s +7°55'37.4"
$ deskephem 1781-03-13 isdate Uranus location # The location of Uranus at William Hershel's first observation
05h48m34.6s +23°38'6.4"
```

deskephem works via reverse polish notation (RPN) to be as simplistic as possible, under this model, it keeps a stack with
values on it, and functions can manipulate this stack.
