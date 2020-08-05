# Conway's Bacteria

Modified rules for Conway's game of life with colony forming behavior.

## Rules

  1. If a Cell and all of it's neighbors are either 0 or 1, then follow standard [CGoL rules](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life).
  2. If a Cell has exactly 4 neighbors, one on each side, or one on each corner, it starts a Colony with its neighbors and grows to 2. Now it and all of it's neighbors follow Colony rules.
  3. If a cell is in a Colony, but with a height of 0, then if the sum of its neighbors' heights is greater or equal to 4 times its tallest neighbor then it goes to 1.
  4. If a cell is in a Colony, but with a height of 1 or more, then if the sum of its neighbors' heights is greater or equal to 8 times its own height then it increases by 1, up to 4.

## Note

  The main program is looking to save it's output in a directory named `scratch/` in the cwd, so it will panic! without that.
