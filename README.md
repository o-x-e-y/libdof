The .dof file format is a json compatible format with specified fields describing any keyboard layout, including some defaults.

# Fields

Some fields are optional.

- `name`: name of the layout
- `author`: (optional) author of the layout   
- `board`: keyboard type the layout is made for. Any value is allowed, but a few values have special properties (explained further below):

  | value     | main layer shape |
  | --------- | ---------------- |
  | `ansi`    | 3x10             |
  | `iso`     | 2x10 + 1x11      |
  | `colstag` | 3x10             |
  
* `date`: (optional) date the layout was created    
* `[tags]`: (optional) array of strings containing relevant tags the layout could have    
* `note`: (optional) string containing some of the author's thoughts    
* `[alt_fingerings]`: (optional) array of 2-key combinations that are recommended to (where possible) be alted    
* `{combos}`: (optional) object of n-key input: n-key output combinations such that when pressed together, the first produces the second
* `{layers}`: specifies all layers on the layout. Each layer is of the form `name: <layer>`, and each layer has rows specified by a string consisting of keys delimited by any amount of whitespace. 
  - All characters are allowed, but some modifier names are reserved, being:
    * `sft`, `shft`
    * `ctl`, `ctrl`
    * `alt`
    * `mta`, `meta`
    * `cps`, `caps`
    * `tb`, `tab`
    * `ret`, `ent`
    * `bsp`, `bcsp`, `bksp`
    * `spc`, `spce`
    * `rpt`, `rept`  
    - Any other 2+ letter combination is interpreted as the layer-switch key to another layer.
  - All layer names are allowed, but there are a few reserved names, being:
    * `main` (mandatory)
    * `shift`
    * `ctrl`
    - While `main` is mandatory to be filled, `shift` and `ctrl` can be elided. 
  - Any shape is allowed, but if you use a standard 3x10 shape, you may be able to elide a fingermap (more on this below). 
  - `~` is used to (on main) specify an empty char, (on shift) to use the default for what's on lowercase, and on any other layer to refer to the key on the main layer. You can escape it using a backslash `\~` to get an actual `~` key. Likewise, backslash must be escaped `\\`.

* `fingering`: specifies which finger presses which key. It's formatted the same as the layers object, and it should have the exact same shape (it will error otherwise). 
  - Allowed fingers include:
    | value | meaning      |
    | ----- | ------------ |
    | `LP`  | left pinky   |
    | `LR`  | left ring    |
    | `LM`  | left middle  |
    | `LI`  | left index   |
    | `LT`  | left thumb   |
    | `RT`  | right thumb  |
    | `RI`  | right index  |
    | `RM`  | right middle |
    | `RR`  | right ring   |
    | `RP`  | right pinky  |
  
  - As alluded to above, you can alternatively specify a string in the `fingering` field to invoke a standard fingering:

    | `board`   | allowed `fingering` strings        |
    | --------- | ---------------------------------- |
    | `ansi`    | `traditional`, `standard`, `angle` |
    | `iso`     | `traditional`, `standard`, `angle` |
    | `ortho`   | `traditional`, `standard`          |
    | `colstag` | `traditional`, `standard`          |

    - If any other value is provided, it should error.

# Examples


```json
{
    "name": "Qwerty",
    "author": "Christopher Latham Sholes",
    "board": "ansi",
    "date": "1889",
    "tags": [
        "bad",
        "fast"
    ],
    "alt-fingerings": [
        "a b",
        "c d"
    ],
    "combos": {
        "a b": "c d",
    },
    "tags": ["bad", "fast"],
    "note": "the OG. Without Qwerty, none of this would be necessary.",
    "layers": {
        "main": [
            "q w e r t  y u i o p [ ] \\",
            "a s d f g  h j k l ; '",
            "z x c v b  n m , . /"
        ],
        "shift": [
            "Q W E R T  Y U I O P { } |",
            "A S D F G  H J K L : \"",
            "Z X C V B  N M < > ?"
        ]
    },
    "fingering": [
        "LP LR LM LI LI  RI RI RM RR RP RP RP RP",
        "LP LR LM LI LI  RI RI RM RR RP RP",
        "LP LR LM LI LI  RI RI RM RR RP",
    ]
}

```

Here is an example of a more minimal specification:

```json
{
    "name": "Qwerty",
    "board": "ansi",
    "layers": {      
        "main": [
            "q w e r t  y u i o p [ ] \\",
            "a s d f g  h j k l ; '",
            "z x c v b  n m , . /"
        ]
    },
    "fingering": "angle"
}
```
