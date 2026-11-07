*Part of [testtools/](README.md), added with #397.*

* **The gallery of unused pictures** (`play_testgallery.rs`, #397; the product owner's request of
  2026-11-07: the game's unused assets on the test map so they can be seen; the demo's own tool, no
  footage). The **[** key (the one right of P: every letter is another control's, so it's a
  punctuation key; `BracketLeft`, in the tool registry `TOOLS` like the menus', so it opens in edit
  mode too and the edit mode's help lists it) opens a panel of the 40 textures the cut-content
  catalogue (`todo/cut-content/catalogue.md`, sections 1 and 6, `textures-unreferenced.txt`) found
  that **nothing in the game uses**: no material, object type, menu, script, effect or the XBE's
  strings and constants names them (the sweep's method is the catalogue's, hashing every name and
  every 32-bit value of the data and the XBE). They are drawn from the game's own texture
  libraries, not the catalogue's PNGs.
  * **The pictures**, seven pages in the ticket's five groups (`PAGES`, `PICTURES`):

    | Page | Group | Holds |
    |---|---|---|
    | 1 | cut characters | the cut deathmatch characters' 128 x 128 select pictures (splash_screen): `kingmen.tga` h_fb53f8eb, `terrorist.tga` h_e0e368d2, `subversive.tga` h_e1a19b31, `mutant1.tga` h_e3730562, `mutant2.tga` h_ef8b9f92, `supermutant.tga` h_f890957c, `feral_meancast.tga` h_10b1e538, `thinker.tga` h_10b50b57, `lesserpriest.tga` h_e1555c19, `slave.tga` h_e848500e |
    | 2 | cut squad | the cut Kingman Posse's squad picture h_19f9d1f0 (name not recovered) |
    | 3 | faces | the cut HUD faces, 32 x 32: `face_kingman.tga` h_1cc0787c, `face_terrorist.tga` h_0be444b1, `face_subversive.tga` h_f7f1acff, `face_mutant.tga` h_fc5f519f, `face_supermutant.tga` h_f05a2afc, `face_vsquad.tga` h_f0fb436d (the Confed V-squad marine's), and two splash_screen faces without a name, h_0fb255c3 and h_fa8d087b |
    | 4 | map | `mp8.tga` h_12a5159a, the picture of a cut eighth deathmatch map |
    | 5 | other | the "BRUTE FORCE" title logo h_10198036, a "temp / art" placeholder h_f60de1cd, a smoke puff h_f7231aeb, a menu bracket h_f1559904, and the older 32 x 32 select thumbnails `follower.tga`, `feral outcast.tga`, `outcast shaman.tga`, `feral shaman.tga`, `feral colonist.tga` and the Subversive's `sub.tga` |
    | 6 | other | menu and HUD pieces: a blue bar h_0d9ff511 (512 x 32), two frame pieces h_e1c53fdd and h_f055730c, a gradient h_fb08fceb, `abutton.tga` h_158c7726 and two identical crescents h_1ef958f5 and h_eead4eff (the tutorial's) |
    | 7 | other | `holoscanbrattonlayer1.tga` h_16d9931f (64 x 4) and `rgbaGradient4x256.tga` h_e53a164b (4 x 256), the two textures `default.xbe` names for the hologram material no shipped material uses (catalogue 1.4), and a grey ramp h_1a1ec37f (8 x 16) |

    Each picture has its **name** (`<name>.tga`, recovered by hashing: the catalogue's section 6,
    the Kingman Posse draft's section 5, the XBE's strings; `names_hash_to_their_pictures` checks
    each against its hash, 26 of the 40; the others read "(unnamed)"), its **hash**, its library,
    its size, format and mips, and the catalogue's **one line**. The page's header says "unused in
    the game". The textures that sit only in unused world materials (the catalogue's section 1.5: ten
    more, masks and bump maps) aren't in the gallery: they aren't pictures.
  * **Where they come from**: the textures' own libraries, read from the archives on a thread the
    first time the gallery opens (`read_pictures`; all 40 in 0.1 s): splash_screen's (the front
    end's, 27), common's (10: the faces and the code-named textures; the five faces are also in
    other levels' libraries, the same bytes) and the tutorial's (3), by `texture::index` and
    `texture::decode` (DXT1, DXT3 and the uncompressed formats). Nothing is added to the game's
    loaded data: no texture a lookup could later find in another library's place.
  * **How they're drawn**: at their **own size**, one texel to a pixel, nearest-sampled, on a grey
    (so the alpha of a face's edge shows), in cells of a fixed height (`CAPTION_HEIGHT`; the first
    layout let each row's height follow its captions' wrapping and left gaps of 100 to 170 px
    between the rows, which fixed heights don't), five a row at 1280 px. A picture of 64 pixels or
    less a side is also shown **magnified** beside itself (`zoom_of`: the largest whole factor up
    to 4 that keeps its longer side within 128 px: a 32 x 32 face x4, a 64 x 4 scan line x2, an
    8 x 16 ramp x4; the 4 x 256 gradient and the 128 x 128 pictures aren't). The textures are
    stored bottom row first, as the HUD's and the menus' are, so the nodes flip them
    (`ImageNode::flip_y`).
  * **Keys**: Up / Down (or the wheel) the previous / next **page**, Page Up / Down the first page
    of the previous / next **group** (cut characters, cut squad, faces, map, other); `[` or Esc
    closes. The tools' own panel (on the right) shows the group, the page and the keys.
  * **Hooks**: `BF_TEST_GALLERY=<s>[,<page>][;...]` opens it at that page (its number from 1, or a
    group's name: `faces`); `BF_TEST_KEYS="1,bracketleft;2,down;3,pagedown;4,esc"` presses the keys
    (`bracketleft`, `pageup`, `pagedown` are named for it). `BF_TOOLS_LOG=1` prints "gallery:
    reading 40 pictures from splash_screen, common and tutorial", "gallery: read 40 of 40 pictures
    in 0.13 s" (naming any that weren't found) and "gallery: page 3 / 7 (faces): face_kingman.tga,
    ..." as each page is built.
  * **Not done**: the ten textures only in unused materials (above); the cut characters aren't
    playable (#395) and the cut squads' classes are #394's.
