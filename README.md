Super Rario Bros
================

A Super Mario Bros. clone, written in Rust, using SDL2.

**PR's are very much welcome! :)**

**NOTE:** This code is still in development. It is borne out of an interest in
learning Rust, not necessarily because we're game developers or Mario
enthusiasts.

The 'Rario' part is because it's written in Rust. Bask in our glorious wittiness.

###Currently Working
- Basic rendering of the map with scrolling
- A movable Mario!
- Trivial collision handling

###TODO
- Improve collision handling
    - It would be nice to have some threaded code, but we'd have to work around
      not being able to copy a number of resources to the other thread(s), and
      it might just not be worth it.
    - <del>Hopefully trait objects and heterogeneous containers can be used to
      extend this to work on multiple objects of the same trait (look at
      'trait-refactor' branch)</del>
- <del>Track texture dimensions in the appropriate sprite struct so as to setup
  for future soft stretching and stop hardcoding values</del>
    - Texture.query() provides access to dimensions via the TextureQuery struct
- Add some kind of map generation
    - <del>Trivially, this is the process of creating collision Rects that Mario may
      not enter</del>
    - Less trivially, this is generating all of the sprites in the map.
        - Enemies must be initialized as they are encountered by Mario, not at
          the program initializaiton
        - Figure out how we want to implement the basic AI for each enemy
            - <del>Enemies should inherit the Sprite struct, maybe
              encapsulated</del>
            - It is more favorable, for now, to override the necessary
              trait-implemented methods for each different enemy.
- Movement:
    - Change maximum jump height based on the length of time for which the jump
      button is held and the speed of Mario
    - Try to use actual physics equations, if they are, indeed, what the
      original game uses
- Perform soft stretching (using `blit_scaled()` so that we may make the game
  window larger)
    - This requires refactoring all code that includes `TILE_SIZE`, which is
      assumed to be 16 pixels at the standard NES resolution.

### Map Parsing - Text to Tiles (Custom Parsing)
World 1-1:
```
See 'res/world1-1.txt'
```
Key:
```
$ = Mario (start)
# = Breakable block
? = Block with single
C = Block with multiple coins
M = Powerup (dependent upon Mario's current state)
S = Star
P = Pipe
R = Stairs
G = Goomba
K = Koopa
E = End (middle of castle)
u = 1-UP (hidden because lowercase)
```

###MISC
To batch convert images to 24-bit bitmaps, use: `mogrify -format bmp -background magenta -alpha remove -type truecolor *.bmp`
