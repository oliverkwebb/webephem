# deskephem - CLI Astronomy Calculator

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
```

deskephem works via reverse polish notation (RPN) to be as simplistic as possible, under this model, it keeps a stack with
values on it, and functions can manipulate this stack.
