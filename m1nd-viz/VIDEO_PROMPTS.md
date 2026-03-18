# m1nd Cinema -- AI Video Generation Prompts

**Target**: 33-second video assembled from 9 clips (3-5s each)
**Resolution**: 1280x720 (16:9)
**Model**: Wan-AI/Wan2.2-T2V-A14B via SiliconFlow API
**Aesthetic**: Dark cinematic (#060B14 void), neon green (#00E5A0) primary, minimal, precise
**Assembly**: Generate clips individually, concatenate with ffmpeg

---

## CLIP 1: TERMINAL COLD OPEN
**Timestamp**: 0:00 - 0:03 (3s)
**Storyboard Scene**: Scene 1 -- "The Familiar"

**Prompt**:
A dark terminal screen in near-black void. A single green cursor blinks on the left side of the screen, sharp monospace font. Characters begin typing one by one from left to right, forming a command line: a grep search command in bright green text on pitch-black background. The typing is steady and deliberate. The rest of the screen is completely empty, just darkness. Extreme minimalism. Only the terminal text glows faintly green against the abyss. Cinematic 4K, ultra-sharp text rendering, no blur, scanline aesthetic, cyberpunk terminal feel. The atmosphere is cold, precise, professional.

**Negative Prompt**: text overlay, watermark, blurry, colorful, bright background, UI elements, buttons, multiple windows, cartoon, anime, human, face, hands

**Duration**: 3s
**Transition to next**: The typed command holds on screen, creating anticipation before results appear.

---

## CLIP 2: THE COST
**Timestamp**: 0:03 - 0:08 (5s)
**Storyboard Scene**: Scene 2 -- "What You Don't See"

**Prompt**:
A dark terminal screen filling with scrolling lines of code search results, gray monospace text appearing rapidly line by line. In the upper portion of the screen, three data counters fade into view: a red token counter spinning upward through thousands, an amber clock ticking seconds, and a red dollar cost incrementing. The counters glow with their respective colors against the near-black background. The scrolling text creates a feeling of information overload. Below the results, a dim gray question appears. The mood shifts from neutral to uneasy as the red numbers grow larger. Dark cinematic atmosphere, data visualization aesthetic, financial dashboard feel, anxiety-inducing pace, dark navy-black background.

**Negative Prompt**: text overlay, watermark, bright colors, white background, cartoon, human, face, cheerful, colorful charts, 3D pie charts

**Duration**: 5s
**Transition to next**: All elements fade to pure black over the last half-second, creating a clean void.

---

## CLIP 3: THE COMMAND
**Timestamp**: 0:08 - 0:10 (2s -- combine with Clip 4 for minimum 3s generation)
**Storyboard Scene**: Scene 3 -- "The Alternative"

**Prompt**:
Pure black screen. A blinking green cursor appears on the left. Characters type out a new command in a warmer, more vibrant green than before. The text glows softly, the word at the start is bold and luminous. The typing finishes, the cursor vanishes instantly as if Enter was pressed. A subtle green energy begins building at the center of the screen, barely perceptible, like something waking up. The atmosphere shifts from cold terminal to something alive. Dark void background, single line of glowing green text, the beginning of activation energy, anticipation before transformation. Cinematic, minimal, the moment a match is struck.

**Negative Prompt**: text overlay, watermark, bright background, colorful, UI elements, multiple lines of text, busy screen, human, face

**Duration**: 3s (includes transition glow into Clip 4)
**Transition to next**: The green energy at center grows, becoming the seed of the neural network in the next clip.

---

## CLIP 4: NEURAL NETWORK ACTIVATION
**Timestamp**: 0:10 - 0:18 (8s -- generate as two 5s clips: 4A and 4B)
**Storyboard Scene**: Scene 4 -- "The Brain Wakes"

### CLIP 4A: Network Materializes (0:10-0:15)
**Prompt**:
Top-down view of a neural network graph materializing from darkness. A single bright green node appears at the center and pulses with energy. From this seed, five nodes bloom outward in the first ring, connected by glowing green lines that draw themselves. A second ring of eight nodes appears in cool blue, connected by blue edges. A third ring of twelve nodes in warm amber appears further out. The activation spreads like a wave rippling through water, each ring lighting up sequentially. The camera slowly zooms in as the network grows. Faint purple dashed lines appear between some nodes, indicating missing connections, gaps in the structure. Dark void background, the network floats in space like a living constellation. Volumetric green and blue glow, bioluminescent aesthetic, neural pathway visualization, brain topology, cinematic scientific visualization.

**Negative Prompt**: text overlay, watermark, bright background, cartoon brain, anatomical brain, human, face, realistic neurons, medical imagery, white background, grid lines

### CLIP 4B: Network Breathing (0:15-0:18)
**Prompt**:
A fully formed neural network graph viewed from above, floating in dark space. Approximately fifty nodes of varying sizes connected by luminous edges in green, blue, and amber. The network gently pulses and breathes, nodes softly glowing brighter and dimmer in a slow rhythm like a heartbeat. Several purple dashed lines pulse between disconnected nodes, highlighting structural gaps. Small floating labels appear next to the purple gaps. A subtle result badge glows at the bottom center showing performance metrics. The overall feeling is a living, breathing intelligence that has found something. Calm after activation. Bioluminescent, dark void background, scientific visualization, volumetric lighting, serene but powerful.

**Negative Prompt**: text overlay, watermark, bright background, cartoon, human, face, busy UI, dashboard, multiple panels, white background

**Duration**: 5s each
**Transition to next**: Network dims to a muted state, preparing for the XLR demonstration.

---

## CLIP 5: XLR NOISE CANCELLATION
**Timestamp**: 0:18 - 0:21 (3s)
**Storyboard Scene**: Scene 5 -- "The Secret Weapon"

**Prompt**:
A dark visualization showing two parallel glowing green signal paths curving through a neural network from opposite sides, converging toward a single bright node at the center. Red noise particles appear along both paths simultaneously, corrupting the signals with erratic red pulses and static interference. As both paths reach the central convergence node, the red noise particles collide and annihilate each other in a brief white flash. The central node erupts with a strong clean green pulse. Both paths turn pure green again, the signal surviving perfectly. The concept of noise cancellation visualized as energy flow. Dark void background, audio engineering aesthetic, balanced signal visualization, the elegance of differential noise rejection. Cinematic, volumetric green glow, particle effects, scientific beauty.

**Negative Prompt**: text overlay, watermark, bright background, audio equipment, XLR cables, physical cables, microphone, studio, human, face, cartoon

**Duration**: 5s
**Transition to next**: The XLR paths fade, the network returns to a neutral state.

---

## CLIP 6: HYPOTHESIS PATHS
**Timestamp**: 0:21 - 0:25 (4s)
**Storyboard Scene**: Scene 6 -- "The Verdict"

**Prompt**:
A neural network graph in dark space with two highlighted nodes far apart: one glowing green on the left, one glowing blue on the right. Multiple thin exploratory paths fan out simultaneously from the green node, tracing through the network like search beams. Some paths reach dead ends and fade away into darkness. Three paths successfully find their way to the blue target node and thicken into bright glowing connections, each a different color: green, blue, and amber. The successful paths pulse with confidence. A verdict panel appears at the bottom showing a high confidence percentage glowing in green. The feeling of an investigation concluding, evidence found, hypothesis confirmed. Dark void background, detective investigation aesthetic merged with tech visualization, path-finding algorithm beauty, cinematic lighting.

**Negative Prompt**: text overlay, watermark, bright background, map, GPS navigation, road map, human, face, cartoon, magnifying glass

**Duration**: 5s
**Transition to next**: Paths and verdict fade to black.

---

## CLIP 7: CAPABILITIES REVEAL
**Timestamp**: 0:25 - 0:29 (4s)
**Storyboard Scene**: Scene 7 -- "8 Things Grep Can't See"

**Prompt**:
Dark screen with a faint neural network breathing in the deep background at very low opacity. Eight text items appear one by one in two columns, materializing from below with a subtle upward drift. Each item glows softly as it appears, clean modern typography against the void. The items accumulate on screen, building an argument through sheer quantity. The text is white and light gray on the near-black background. Behind the text, the ghost of the neural network pulses faintly, reminding the viewer of the intelligence powering these capabilities. Minimal, typographic, the beauty of a feature list presented as an indictment. Each capability appears with a small accent of green glow. Clean, modern, dark cinematic design.

**Negative Prompt**: text overlay, watermark, bright background, colorful icons, emoji, bullet points, presentation slide, PowerPoint, human, face, busy design

**Duration**: 5s
**Transition to next**: All text elements fade out to black.

---

## CLIP 8: THE COMPARISON
**Timestamp**: 0:29 - 0:32 (3s)
**Storyboard Scene**: Scene 8 -- "The Numbers"

**Prompt**:
A dark screen where a comparison visualization builds from the center outward. Two columns of data appear: the left column pulses in red showing large, expensive numbers, the right column glows in vibrant green showing small, efficient numbers. Rows of metrics appear one by one from top to bottom, each row revealing the stark contrast between old technology and new. The red numbers are uniformly bad: high costs, slow times, zero capabilities. The green numbers are uniformly superior: near-zero cost, millisecond speed, many capabilities. One particular green zero glows intensely, the most important number on screen, radiating with significance. The visualization is clean, typographic, data-driven. Dark void background, the devastating simplicity of side-by-side comparison. Infographic aesthetic, cinematic data visualization, the kill shot of numbers.

**Negative Prompt**: text overlay, watermark, bright background, bar chart, pie chart, 3D graph, colorful infographic, cartoon, human, face, busy design, Excel spreadsheet

**Duration**: 5s
**Transition to next**: The entire table fades to pure black.

---

## CLIP 9: FINALE
**Timestamp**: 0:32 - 0:37 (5s)
**Storyboard Scene**: Scene 9 -- "The Brand"

**Prompt**:
Pure black void. A logo appears at center with dramatic impact, a bold typographic mark in vibrant neon green, arriving with weight and confidence, slightly overshooting then settling into place. A soft green glow radiates from behind the logo like a halo. Below the logo, a tagline fades in with elegant restraint in cool gray. Below the tagline, a secondary line in white. The entire composition is centered, minimal, powerful. Behind everything, at barely perceptible opacity, the full neural network graph makes one final appearance, all nodes pulsing once in unison like a single heartbeat, then settling into stillness. The green glow breathes gently. The feeling of resolution, a brand landing with the confidence of proven technology. Dark cinematic void, minimal design, the elegance of a film's final frame. Logo reveal, brand moment, ethereal and inevitable.

**Negative Prompt**: text overlay, watermark, bright background, busy design, multiple logos, colorful, playful, cartoon, human, face, corporate stock photo feel

**Duration**: 5s
**Transition to next**: Holds as the loop point. The heartbeat glow fades gently toward black, connecting back to Clip 1's opening void.

---

## ASSEMBLY GUIDE

### Generation Order
Generate all 10 clips (Clip 4 is split into 4A and 4B). Each clip is independent -- the AI model has no memory between clips.

### Post-Processing Pipeline
```bash
# 1. Download all generated clips
# 2. Trim each to exact duration needed
ffmpeg -i clip1.mp4 -t 3 -c copy clip1_trimmed.mp4
ffmpeg -i clip2.mp4 -t 5 -c copy clip2_trimmed.mp4
ffmpeg -i clip3.mp4 -t 2 -c copy clip3_trimmed.mp4
ffmpeg -i clip4a.mp4 -t 5 -c copy clip4a_trimmed.mp4
ffmpeg -i clip4b.mp4 -t 3 -c copy clip4b_trimmed.mp4
ffmpeg -i clip5.mp4 -t 3 -c copy clip5_trimmed.mp4
ffmpeg -i clip6.mp4 -t 4 -c copy clip6_trimmed.mp4
ffmpeg -i clip7.mp4 -t 4 -c copy clip7_trimmed.mp4
ffmpeg -i clip8.mp4 -t 4 -c copy clip8_trimmed.mp4
ffmpeg -i clip9.mp4 -t 5 -c copy clip9_trimmed.mp4

# 3. Create concat list
cat > clips.txt << 'LIST'
file 'clip1_trimmed.mp4'
file 'clip2_trimmed.mp4'
file 'clip3_trimmed.mp4'
file 'clip4a_trimmed.mp4'
file 'clip4b_trimmed.mp4'
file 'clip5_trimmed.mp4'
file 'clip6_trimmed.mp4'
file 'clip7_trimmed.mp4'
file 'clip8_trimmed.mp4'
file 'clip9_trimmed.mp4'
LIST

# 4. Concatenate
ffmpeg -f concat -safe 0 -i clips.txt -c copy m1nd-cinema.mp4

# 5. Optional: convert to GIF for README
gifski --fps 24 --width 960 --quality 85 \
  <(ffmpeg -i m1nd-cinema.mp4 -f image2pipe -vcodec ppm -) \
  -o m1nd-cinema.gif
```

### Color Grading (post-processing)
If the AI-generated clips don't match the exact color palette, apply color correction:
```bash
# Darken backgrounds toward #060B14, boost greens toward #00E5A0
ffmpeg -i input.mp4 -vf "curves=r='0/0 0.1/0.02 1/0.8':g='0/0 0.1/0.03 0.5/0.5 1/0.9':b='0/0 0.1/0.02 1/0.7',eq=brightness=-0.1:contrast=1.2:saturation=1.3" output.mp4
```

### Cross-Fade Transitions (optional)
For smoother clip-to-clip transitions:
```bash
# Add 0.5s cross-fade between clips using xfade filter
ffmpeg -i clip1.mp4 -i clip2.mp4 -filter_complex "xfade=transition=fade:duration=0.5:offset=2.5" output.mp4
```
