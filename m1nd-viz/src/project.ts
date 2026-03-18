import {makeProject} from '@motion-canvas/core';

import brain from './scenes/brain?scene';
import verdict from './scenes/verdict?scene';

export default makeProject({
  scenes: [brain, verdict],
});
