The .dof file format is a json compatible format with specified fields describing any keyboard layout,
including some defaults.

It has a set amount of (sometimes optional) fields:

* `name`: name of the layout    
* `[author]`: author of the layout   
* `board`: keyboard type the layout is made for. Any value is allowed, but a few values have special
properties (explained further below):
    - `ansi`
    - `iso`
    - `ortho`
    - `colstag`
* `[date]`: date the layout was created. 
* `[tags]`: array of strings containing relevant tags the layout could have.
* `[description]`: string containing some of the author's thoughts.
* `[link]`: url to a page with more information about the layout.
* `layers`: specifies all layers on the layout. They're of the form of `name: <layer>`, and
each layer has rows specified by a string consisting of keys delimited by any amount of
whitespace (but typically a space). They work like the following:
    - if the string length is 1, output:
        - An empty key when it's equal to `~`
        - A transparent key when it's equal to `*`, which refers to the value on the main layer. This
is equivalent to `~` when on the main layer.
        - Enter when it's equal to `\n`,
        - Tab when it's equal to `\t`,
        - A character key otherwise.
    - if the string length is more than 1, output:
        - `~` and `*` characters if it contains `\\~` and `\\*` respectively,
        - A special key like shift or space when provided with specific identifiers which can be
found at the bottom of the document,
        - A layer key if it leads with an `@`, for example `@altgr`
        - A word key with its first character removed if it leads with `#`, `\\#` or`\\@`, for example
`\\@altgr` would output `@altgr` rather than become an altgr layer key,
        - A word key, which outputs multiple characters at the same time, otherwise.


    All layer names are allowed though two are reserved, being:
    - `main` (mandatory)
    - `shift`
  
    While main is mandatory to be filled, shift can be elided and will follow qwerty's
capitalization scheme. Any shape is allowed, but if you use a standard 3x10 shape, you may be
able to elide a fingermap (more on this below).

* `fingering`: specifies which finger presses which key. It's formatted the same as the
layers object, and it should have the exact same shape (it will error otherwise):
    - `LP` or `0`: left pinky
    - `LR` or `1`: left ring
    - `LM` or `2`: left middle
    - `LI` or `3`: left index
    - `LT` or `4`: left thumb
    - `RT` or `5`: right thumb
    - `RI` or `6`: right index
    - `RM` or `7`: right middle
    - `RR` or `8`: right ring
    - `RP` or `9`: right pinky
  
    As alluded to above you can forego defining this completely and instead provide just a string
instead in the following scenarios:
    - board = ansi, main layer shape starts at qwerty `q`, allowed fingerings: traditional,
standard, angle
    - board = iso, main layer shape starts at qwerty `q` with 11 keys on the bottom row, allowed
fingerings: traditional, standard, angle
    - board = ortho, main layer shape = 3x10, allod fingerings: traditional, standard
    - board = colstag, main layer shap = 3x10, allowed fingerings: traditional, standard
  
    If any other value is provided, it should error.

## Special modifier values:
* `esc` => `Esc`,
* `repeat`, `rpt` => `Repeat`,
* `space`, `spc` => `Space`,
* `tab`, `tb` => `Tab`,
* `enter`, `return`, `ret`, `ent`, `rt` => `Enter`,
* `shift`, `shft`, `sft`, `st` => `Shift`,
* `caps`, `cps`, `cp` => `Caps`,
* `ctrl`, `ctl`, `ct` => `Ctrl`,
* `alt`, `lalt`, `ralt`, `lt` => `Alt`,
* `meta`, `mta`, `met`, `mt`, `super`, `sup`, `sp` => `Meta`,
* `fn` => `Fn`,
* `backspace`, `bksp`, `bcsp`, `bsp` => `Backspace`,
* `del` => `Del`,