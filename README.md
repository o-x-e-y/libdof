# Libdof

The .dof file format is a json compatible format with specified fields describing any keyboard layout,
including some defaults.

It has a set amount of (sometimes `optional*`) fields:

* `name`: name of the layout
* `authors[]*`: authors of the layout
* `year*`: year the layout was created.
* `description*`: string containing a description of the layout.
* `link*`: url to a page with more information about the layout.

* `layers{}`: specifies all layers on the layout. They're of the form of `name: <layer>` where each
    layer is an array of rows specified by a string consisting of keys delimited by any amount of
    whitespace. They work like the following:
    * If the string length is 1, output:
        * An empty key when it's equal to `~`
        * A transparent key when it's equal to `*`, which refers to the value on the main layer.
            This is equivalent to `~` when on the main layer.
        * Enter when it's equal to `\n`,
        * Tab when it's equal to `\t`,
        * A character key otherwise.

    * If the string length is more than 1, output:
        * `~` and `*` characters if it contains `\\~` and `\\*` respectively,
        * A special key like shift or space when provided with specific identifiers which can be
            found at the bottom of the document,
        * A layer key if it leads with an `@`, for example `@altgr`
        * A word key with its first character removed if it leads with `#`, `\\#` or`\\@`, for
            example `\\@altgr` would output `@altgr` rather than become an altgr layer key,
        * A word key, which outputs multiple characters at the same time, otherwise.

    * If the string is any of these modifier values:
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
        * `del` => `Del`

    All layer names are allowed though two are reserved:
    * `main` (mandatory)
    * `shift`
  
    While `main` is mandatory to be filled, `shift` can be elided and will follow qwerty's
    capitalization scheme when unspecified. Any shape is allowed, but if you use a standard 3x10
    shape you may be able to forego specifying a custom finger map and keyboard as well.

* `board`: specifies the keyboard type the layout is made for. Three formats are accepted here:
    * `ansi`, `iso`, `ortho`, `colstag` specify preset boards, which by default place the provided
        as if they are a standard 3x10 matrix.

    * A relative board, which specifies a keyboard that has horizontal rows, but isn't ortholinear;
        ansi and iso boards fall into this category. The way they're specified is with an array of
        strings that contain something like `"k 2k 3"`, where `k` is a key of 1x1, `<number>k` is
        a key with a width of `number`, and simply a number specifies empty space, used for split
        ortho boards for example.

    * A full specification. They're specified as an array of arrays, containing each key as a
        string. Each string contains 2 to four numbers specified by whitespace: `"1 2.5"` specifies
        `(x, y) = (1, 2.5)`, where further numbers specify the width and height respectively.

* `anchor[]*`: Specifies where on the physical board the actual layers 'anchor' on. For example, if
    you wanted to place a 3x10 matrix on an `ansi` or `iso` board, the anchor is one from the top,
    and one from the left so `[1, 1]`. There are default anchors for the preset boards, but you are
    required to provide an anchor for custom boards.

* `fingering`: specifies which finger presses which key. For known boards (`iso`, `ansi` etc)
    you can specify a known name:
    * `board: "ansi"` where the main layer shape starts at qwerty `q`, allowed fingerings are
        `traditional`, `standard`, `angle`
    * `board: "iso"` where the main layer shape starts at qwerty `q` and the bottom row includes
        the iso key, allowed fingerings are `traditional`, `standard`, `angle`
    * `board: "ortho"`, `board: "colstag"`, allowed fingerings are `traditional`, `standard`

    If you use a custom keyboard, you can specify fingering the same way you would a layer, but
    with fingers. Layer and fingering shapes should match though, as it will error otherwise.
    * `LP` or `0`: left pinky
    * `LR` or `1`: left ring
    * `LM` or `2`: left middle
    * `LI` or `3`: left index
    * `LT` or `4`: left thumb
    * `RT` or `5`: right thumb
    * `RI` or `6`: right index
    * `RM` or `7`: right middle
    * `RR` or `8`: right ring
    * `RP` or `9`: right pinky

* `combos{}`: Allows you to specify combos on a layer by layer basis. This works the same as with
    `layers`, except you now specify an array of combos. Say you specified `"k k-2": "rpt"` for
    the `main` layer, this means that if you pressed the first (starting top-left) and second `k`
    together, they output the repeat key. Index `0` and `1` are equivalent here, and do not have to
    be specified.

For all of these, it might be worth it to check out the
[example dofs](https://github.com/O-X-E-Y/libdof/tree/main/example_dofs).
