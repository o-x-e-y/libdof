The .dof file format is a json compatible format with specified fields describing any keyboard layout, including some defaults.

It has a set amount of (sometimes optional) fields:

name: name of the layout    
(optional) author: author of the layout   
board: keyboard type the layout is made for. Any value is allowed, but a few values have special properties (explained further below):
  * ansi
  * iso
  * ortho
  * colstag
  
(optional) `date`: date the layout was created. 
(optional) `[tags]`: array of strings containing relevant tags the layout could have.
(optional) `note`: string containing some of the author's thoughts.
(optional) `[alt_fingerings]`: array of 2-character combinations that are intended to be alted.
(optional) `{combos}`: object of n-character: n-character combinations that convert into each other when pressed together.
{layers}: specifies all layers on the layout. They're of the form of name: <layer>, and each layer has rows specified by a string consisting of keys delimited by any amount of whitespace. All characters are allowed, but some modifier names are reserved, being:
  * sft, shft,
  * ctl, ctrl,
  * alt, 
  * mta, meta,
  * cps, caps,
  * tb, tab,
  * ret, ent,
  * bsp, bcsp, bksp,
  * spc, spce,
  * rpt, rept.
  
Any other 2+ letter combination refers to another layer.
All layer names are allowed though two are reserved, being:
  * main (mandatory)
  * shift
  
While main is mandatory to be filled, shift can be elided and will follow qwerty's capitalization scheme. Any shape is allowed, but if you use a standard 3x10 shape, you may be able to elide a fingermap (more on this below). '~' is used to (on main) specify an empty character and on any other layer (including shift) to refer to the key on the base layer. You can escape it like so \~, as you do with \\ to get an actual ~ key.

fingering: specifies which finger presses which key. It's formatted the same as the layers object, and it should have the exact same shape (it will error otherwise):
  * LP or 0: left pinky
  * LR or 1: left ring
  * LM or 2: left middle
  * LI or 3: left index
  * LT or 4: left thumb
  * RT or 5: right thumb
  * RI or 6: right index
  * RM or 7: right middle
  * RR or 8: right ring
  * RP or 9: right pinky
  
As alluded to above you can forego defining this completely and instead provide just a string instead in the following scenarios:
  * board = ansi, main layer shape = 3x10, allowed fingerings: traditional, standard, angle
  * board = iso, main layer shape = 2x10 + 1x11, allowed fingerings: traditional, standard, angle
  * board = ortho, main layer shape = 3x10, allod fingerings: traditional, standard
  * board = colstag, main layer shap = 3x10, allowed fingerings: traditional, standard
  
If any other value is provided, it should error.
