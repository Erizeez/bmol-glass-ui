# UI fonts

Put the interface font files here to have them embedded into the binary at
build time (see `build.rs`). Font files in this directory are git-ignored:
Apple's license only allows SF fonts in apps for Apple platforms, so they are
never committed or distributed.

When this directory is empty, the font module falls back at runtime to the
system UI font on macOS (`SFNS.ttf`) or to Iced's default font elsewhere.

## Which fonts

Best: SF Pro Text static weights from <https://developer.apple.com/fonts/> —
for example `SF-Pro-Text-Regular.otf` and `SF-Pro-Text-Semibold.otf`. The
family name is read from each file's name table automatically, and the weight
comes from its OS/2 table.

## Alternative: instance the macOS system font

`/System/Library/Fonts/SFNS.ttf` is a variable font whose default optical
size is 28 (the Display cut) — rendered at settings sizes it looks tight and
spindly, and the renderers we use cannot address the `opsz` axis. Generate
static Text-cut instances instead (needs `pip install fontTools`):

```python
from fontTools.varLib.instancer import instantiateVariableFont
from fontTools.ttLib import TTFont

for wght, sub, wclass in [(400, 'Regular', 400), (590, 'Semibold', 600)]:
    f = TTFont('/System/Library/Fonts/SFNS.ttf')
    instantiateVariableFont(
        f, {'wdth': 100, 'opsz': 17, 'GRAD': 400, 'wght': wght},
        updateFontNames=False)
    for platform in [(3, 1, 0x409), (1, 0, 0)]:
        f['name'].setName('SFNS Text', 1, *platform)
        f['name'].setName(sub, 2, *platform)
        f['name'].setName(f'SFNS Text {sub}', 4, *platform)
        f['name'].setName(f'SFNSText-{sub}', 6, *platform)
    f['OS/2'].usWeightClass = wclass
    f.save(f'SFNS-Text-{sub}.ttf')  # copy into this directory
```
