#!/usr/bin/env python3
"""
m1nd Cinema -- AI Video Clip Generator
Generates video clips via SiliconFlow API (Wan2.2-T2V-A14B)
then downloads and optionally assembles them.

Usage:
    python generate_clips.py --api-key YOUR_KEY
    python generate_clips.py --api-key YOUR_KEY --clip 4a
    python generate_clips.py --api-key YOUR_KEY --assemble
"""

import argparse
import httpx
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------

BASE_URL = "https://api.siliconflow.com/v1"
MODEL = "Wan-AI/Wan2.2-T2V-A14B"
IMAGE_SIZE = "1280x720"
OUTPUT_DIR = Path(__file__).parent / "clips"
POLL_INTERVAL = 10  # seconds between status checks
MAX_POLL_TIME = 600  # 10 minutes max wait per clip


# ---------------------------------------------------------------------------
# Clip Definitions
# ---------------------------------------------------------------------------

@dataclass
class Clip:
    id: str
    name: str
    timestamp: str
    duration_trim: float  # seconds to trim to in final assembly
    prompt: str
    negative_prompt: str


CLIPS = [
    Clip(
        id="1",
        name="terminal_cold_open",
        timestamp="0:00-0:03",
        duration_trim=3.0,
        prompt=(
            "A dark terminal screen in near-black void. A single green cursor "
            "blinks on the left side of the screen, sharp monospace font. "
            "Characters begin typing one by one from left to right, forming a "
            "command line: a grep search command in bright green text on "
            "pitch-black background. The typing is steady and deliberate. The "
            "rest of the screen is completely empty, just darkness. Extreme "
            "minimalism. Only the terminal text glows faintly green against "
            "the abyss. Cinematic 4K, ultra-sharp text rendering, no blur, "
            "scanline aesthetic, cyberpunk terminal feel. The atmosphere is "
            "cold, precise, professional."
        ),
        negative_prompt=(
            "text overlay, watermark, blurry, colorful, bright background, "
            "UI elements, buttons, multiple windows, cartoon, anime, human, "
            "face, hands"
        ),
    ),
    Clip(
        id="2",
        name="the_cost",
        timestamp="0:03-0:08",
        duration_trim=5.0,
        prompt=(
            "A dark terminal screen filling with scrolling lines of code "
            "search results, gray monospace text appearing rapidly line by "
            "line. In the upper portion of the screen, three data counters "
            "fade into view: a red token counter spinning upward through "
            "thousands, an amber clock ticking seconds, and a red dollar cost "
            "incrementing. The counters glow with their respective colors "
            "against the near-black background. The scrolling text creates a "
            "feeling of information overload. Below the results, a dim gray "
            "question appears. The mood shifts from neutral to uneasy as the "
            "red numbers grow larger. Dark cinematic atmosphere, data "
            "visualization aesthetic, financial dashboard feel, "
            "anxiety-inducing pace, dark navy-black background."
        ),
        negative_prompt=(
            "text overlay, watermark, bright colors, white background, "
            "cartoon, human, face, cheerful, colorful charts, 3D pie charts"
        ),
    ),
    Clip(
        id="3",
        name="the_command",
        timestamp="0:08-0:10",
        duration_trim=2.0,
        prompt=(
            "Pure black screen. A blinking green cursor appears on the left. "
            "Characters type out a new command in a warmer, more vibrant green "
            "than before. The text glows softly, the word at the start is bold "
            "and luminous. The typing finishes, the cursor vanishes instantly "
            "as if Enter was pressed. A subtle green energy begins building at "
            "the center of the screen, barely perceptible, like something "
            "waking up. The atmosphere shifts from cold terminal to something "
            "alive. Dark void background, single line of glowing green text, "
            "the beginning of activation energy, anticipation before "
            "transformation. Cinematic, minimal, the moment a match is struck."
        ),
        negative_prompt=(
            "text overlay, watermark, bright background, colorful, UI "
            "elements, multiple lines of text, busy screen, human, face"
        ),
    ),
    Clip(
        id="4a",
        name="network_materializes",
        timestamp="0:10-0:15",
        duration_trim=5.0,
        prompt=(
            "Top-down view of a neural network graph materializing from "
            "darkness. A single bright green node appears at the center and "
            "pulses with energy. From this seed, five nodes bloom outward in "
            "the first ring, connected by glowing green lines that draw "
            "themselves. A second ring of eight nodes appears in cool blue, "
            "connected by blue edges. A third ring of twelve nodes in warm "
            "amber appears further out. The activation spreads like a wave "
            "rippling through water, each ring lighting up sequentially. The "
            "camera slowly zooms in as the network grows. Faint purple dashed "
            "lines appear between some nodes, indicating missing connections, "
            "gaps in the structure. Dark void background, the network floats "
            "in space like a living constellation. Volumetric green and blue "
            "glow, bioluminescent aesthetic, neural pathway visualization, "
            "brain topology, cinematic scientific visualization."
        ),
        negative_prompt=(
            "text overlay, watermark, bright background, cartoon brain, "
            "anatomical brain, human, face, realistic neurons, medical "
            "imagery, white background, grid lines"
        ),
    ),
    Clip(
        id="4b",
        name="network_breathing",
        timestamp="0:15-0:18",
        duration_trim=3.0,
        prompt=(
            "A fully formed neural network graph viewed from above, floating "
            "in dark space. Approximately fifty nodes of varying sizes "
            "connected by luminous edges in green, blue, and amber. The "
            "network gently pulses and breathes, nodes softly glowing brighter "
            "and dimmer in a slow rhythm like a heartbeat. Several purple "
            "dashed lines pulse between disconnected nodes, highlighting "
            "structural gaps. Small floating labels appear next to the purple "
            "gaps. A subtle result badge glows at the bottom center showing "
            "performance metrics. The overall feeling is a living, breathing "
            "intelligence that has found something. Calm after activation. "
            "Bioluminescent, dark void background, scientific visualization, "
            "volumetric lighting, serene but powerful."
        ),
        negative_prompt=(
            "text overlay, watermark, bright background, cartoon, human, "
            "face, busy UI, dashboard, multiple panels, white background"
        ),
    ),
    Clip(
        id="5",
        name="xlr_noise_cancellation",
        timestamp="0:18-0:21",
        duration_trim=3.0,
        prompt=(
            "A dark visualization showing two parallel glowing green signal "
            "paths curving through a neural network from opposite sides, "
            "converging toward a single bright node at the center. Red noise "
            "particles appear along both paths simultaneously, corrupting the "
            "signals with erratic red pulses and static interference. As both "
            "paths reach the central convergence node, the red noise particles "
            "collide and annihilate each other in a brief white flash. The "
            "central node erupts with a strong clean green pulse. Both paths "
            "turn pure green again, the signal surviving perfectly. The "
            "concept of noise cancellation visualized as energy flow. Dark "
            "void background, audio engineering aesthetic, balanced signal "
            "visualization, the elegance of differential noise rejection. "
            "Cinematic, volumetric green glow, particle effects, scientific "
            "beauty."
        ),
        negative_prompt=(
            "text overlay, watermark, bright background, audio equipment, "
            "XLR cables, physical cables, microphone, studio, human, face, "
            "cartoon"
        ),
    ),
    Clip(
        id="6",
        name="hypothesis_paths",
        timestamp="0:21-0:25",
        duration_trim=4.0,
        prompt=(
            "A neural network graph in dark space with two highlighted nodes "
            "far apart: one glowing green on the left, one glowing blue on "
            "the right. Multiple thin exploratory paths fan out simultaneously "
            "from the green node, tracing through the network like search "
            "beams. Some paths reach dead ends and fade away into darkness. "
            "Three paths successfully find their way to the blue target node "
            "and thicken into bright glowing connections, each a different "
            "color: green, blue, and amber. The successful paths pulse with "
            "confidence. A verdict panel appears at the bottom showing a high "
            "confidence percentage glowing in green. The feeling of an "
            "investigation concluding, evidence found, hypothesis confirmed. "
            "Dark void background, detective investigation aesthetic merged "
            "with tech visualization, path-finding algorithm beauty, "
            "cinematic lighting."
        ),
        negative_prompt=(
            "text overlay, watermark, bright background, map, GPS navigation, "
            "road map, human, face, cartoon, magnifying glass"
        ),
    ),
    Clip(
        id="7",
        name="capabilities_reveal",
        timestamp="0:25-0:29",
        duration_trim=4.0,
        prompt=(
            "Dark screen with a faint neural network breathing in the deep "
            "background at very low opacity. Eight text items appear one by "
            "one in two columns, materializing from below with a subtle upward "
            "drift. Each item glows softly as it appears, clean modern "
            "typography against the void. The items accumulate on screen, "
            "building an argument through sheer quantity. The text is white "
            "and light gray on the near-black background. Behind the text, "
            "the ghost of the neural network pulses faintly, reminding the "
            "viewer of the intelligence powering these capabilities. Minimal, "
            "typographic, the beauty of a feature list presented as an "
            "indictment. Each capability appears with a small accent of green "
            "glow. Clean, modern, dark cinematic design."
        ),
        negative_prompt=(
            "text overlay, watermark, bright background, colorful icons, "
            "emoji, bullet points, presentation slide, PowerPoint, human, "
            "face, busy design"
        ),
    ),
    Clip(
        id="8",
        name="the_comparison",
        timestamp="0:29-0:32",
        duration_trim=4.0,
        prompt=(
            "A dark screen where a comparison visualization builds from the "
            "center outward. Two columns of data appear: the left column "
            "pulses in red showing large, expensive numbers, the right column "
            "glows in vibrant green showing small, efficient numbers. Rows of "
            "metrics appear one by one from top to bottom, each row revealing "
            "the stark contrast between old technology and new. The red "
            "numbers are uniformly bad: high costs, slow times, zero "
            "capabilities. The green numbers are uniformly superior: near-zero "
            "cost, millisecond speed, many capabilities. One particular green "
            "zero glows intensely, the most important number on screen, "
            "radiating with significance. The visualization is clean, "
            "typographic, data-driven. Dark void background, the devastating "
            "simplicity of side-by-side comparison. Infographic aesthetic, "
            "cinematic data visualization, the kill shot of numbers."
        ),
        negative_prompt=(
            "text overlay, watermark, bright background, bar chart, pie "
            "chart, 3D graph, colorful infographic, cartoon, human, face, "
            "busy design, Excel spreadsheet"
        ),
    ),
    Clip(
        id="9",
        name="finale",
        timestamp="0:32-0:37",
        duration_trim=5.0,
        prompt=(
            "Pure black void. A logo appears at center with dramatic impact, "
            "a bold typographic mark in vibrant neon green, arriving with "
            "weight and confidence, slightly overshooting then settling into "
            "place. A soft green glow radiates from behind the logo like a "
            "halo. Below the logo, a tagline fades in with elegant restraint "
            "in cool gray. Below the tagline, a secondary line in white. The "
            "entire composition is centered, minimal, powerful. Behind "
            "everything, at barely perceptible opacity, the full neural "
            "network graph makes one final appearance, all nodes pulsing once "
            "in unison like a single heartbeat, then settling into stillness. "
            "The green glow breathes gently. The feeling of resolution, a "
            "brand landing with the confidence of proven technology. Dark "
            "cinematic void, minimal design, the elegance of a film's final "
            "frame. Logo reveal, brand moment, ethereal and inevitable."
        ),
        negative_prompt=(
            "text overlay, watermark, bright background, busy design, "
            "multiple logos, colorful, playful, cartoon, human, face, "
            "corporate stock photo feel"
        ),
    ),
]


# ---------------------------------------------------------------------------
# API Functions
# ---------------------------------------------------------------------------

def submit_video(client: httpx.Client, clip: Clip) -> str:
    """Submit a video generation request. Returns requestId."""
    payload = {
        "model": MODEL,
        "prompt": clip.prompt,
        "negative_prompt": clip.negative_prompt,
        "image_size": IMAGE_SIZE,
    }

    print(f"[SUBMIT] Clip {clip.id} ({clip.name}) ...")
    resp = client.post(
        f"{BASE_URL}/video/submit",
        json=payload,
        timeout=60,
    )
    resp.raise_for_status()
    data = resp.json()
    request_id = data["requestId"]
    print(f"  -> requestId: {request_id}")
    return request_id


def poll_status(client: httpx.Client, request_id: str) -> dict:
    """Poll until video is ready. Returns result dict with video URL."""
    elapsed = 0
    while elapsed < MAX_POLL_TIME:
        resp = client.post(
            f"{BASE_URL}/video/status",
            json={"requestId": request_id},
            timeout=30,
        )
        resp.raise_for_status()
        data = resp.json()
        status = data.get("status", "Unknown")

        if status == "Succeed":
            video_url = data["results"]["videos"][0]["url"]
            inference_time = data["results"]["timings"]["inference"]
            seed = data["results"].get("seed", "N/A")
            print(f"  -> DONE (inference: {inference_time:.1f}s, seed: {seed})")
            return {"url": video_url, "seed": seed, "inference_time": inference_time}

        if status == "Failed":
            reason = data.get("reason", "Unknown error")
            raise RuntimeError(f"Video generation failed: {reason}")

        print(f"  -> Status: {status} (elapsed: {elapsed}s)")
        time.sleep(POLL_INTERVAL)
        elapsed += POLL_INTERVAL

    raise TimeoutError(f"Timed out after {MAX_POLL_TIME}s waiting for video")


def download_video(client: httpx.Client, url: str, output_path: Path) -> None:
    """Download the generated video file."""
    print(f"  -> Downloading to {output_path} ...")
    with client.stream("GET", url, timeout=120) as resp:
        resp.raise_for_status()
        with open(output_path, "wb") as f:
            for chunk in resp.iter_bytes(chunk_size=8192):
                f.write(chunk)
    size_mb = output_path.stat().st_size / (1024 * 1024)
    print(f"  -> Saved ({size_mb:.1f} MB)")


# ---------------------------------------------------------------------------
# Assembly
# ---------------------------------------------------------------------------

def trim_clip(input_path: Path, output_path: Path, duration: float) -> None:
    """Trim a clip to exact duration."""
    cmd = [
        "ffmpeg", "-y",
        "-i", str(input_path),
        "-t", str(duration),
        "-c", "copy",
        str(output_path),
    ]
    subprocess.run(cmd, check=True, capture_output=True)


def assemble_clips(output_dir: Path) -> None:
    """Concatenate all trimmed clips into final video."""
    trimmed_dir = output_dir / "trimmed"
    trimmed_dir.mkdir(exist_ok=True)

    # Trim each clip
    clip_order = []
    for clip in CLIPS:
        raw_path = output_dir / f"clip_{clip.id}_{clip.name}.mp4"
        if not raw_path.exists():
            print(f"[WARN] Missing clip: {raw_path.name} -- skipping")
            continue
        trimmed_path = trimmed_dir / f"clip_{clip.id}_trimmed.mp4"
        print(f"[TRIM] {raw_path.name} -> {clip.duration_trim}s")
        trim_clip(raw_path, trimmed_path, clip.duration_trim)
        clip_order.append(trimmed_path)

    if not clip_order:
        print("[ERROR] No clips to assemble")
        return

    # Write concat list
    concat_file = trimmed_dir / "clips.txt"
    with open(concat_file, "w") as f:
        for path in clip_order:
            f.write(f"file '{path.name}'\n")

    # Concatenate
    final_path = output_dir / "m1nd-cinema.mp4"
    cmd = [
        "ffmpeg", "-y",
        "-f", "concat",
        "-safe", "0",
        "-i", str(concat_file),
        "-c", "copy",
        str(final_path),
    ]
    print(f"[ASSEMBLE] -> {final_path}")
    subprocess.run(cmd, check=True, capture_output=True)

    size_mb = final_path.stat().st_size / (1024 * 1024)
    print(f"[DONE] Final video: {final_path} ({size_mb:.1f} MB)")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="m1nd Cinema clip generator")
    parser.add_argument("--api-key", required=True, help="SiliconFlow API key")
    parser.add_argument(
        "--clip",
        help="Generate a specific clip ID (e.g. '1', '4a'). Default: all clips.",
    )
    parser.add_argument(
        "--assemble",
        action="store_true",
        help="Assemble downloaded clips into final video (no generation).",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=OUTPUT_DIR,
        help=f"Output directory for clips (default: {OUTPUT_DIR})",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print prompts without calling API.",
    )
    args = parser.parse_args()

    args.output_dir.mkdir(parents=True, exist_ok=True)

    # Assemble-only mode
    if args.assemble:
        assemble_clips(args.output_dir)
        return

    # Select clips to generate
    if args.clip:
        targets = [c for c in CLIPS if c.id == args.clip]
        if not targets:
            valid = ", ".join(c.id for c in CLIPS)
            print(f"[ERROR] Unknown clip '{args.clip}'. Valid: {valid}")
            sys.exit(1)
    else:
        targets = CLIPS

    # Dry run: just print prompts
    if args.dry_run:
        for clip in targets:
            print(f"\n{'='*70}")
            print(f"CLIP {clip.id}: {clip.name} ({clip.timestamp})")
            print(f"{'='*70}")
            print(f"PROMPT:\n{clip.prompt}\n")
            print(f"NEGATIVE:\n{clip.negative_prompt}\n")
        return

    # Generate clips
    headers = {"Authorization": f"Bearer {args.api_key}"}
    client = httpx.Client(headers=headers)

    results = {}
    try:
        for clip in targets:
            print(f"\n{'='*70}")
            print(f"CLIP {clip.id}: {clip.name} ({clip.timestamp})")
            print(f"{'='*70}")

            # Submit
            request_id = submit_video(client, clip)

            # Poll
            result = poll_status(client, request_id)

            # Download
            output_path = args.output_dir / f"clip_{clip.id}_{clip.name}.mp4"
            download_video(client, result["url"], output_path)

            results[clip.id] = {
                "request_id": request_id,
                "seed": result["seed"],
                "inference_time": result["inference_time"],
                "file": str(output_path),
            }

            print()

    finally:
        client.close()

    # Save generation manifest
    manifest_path = args.output_dir / "manifest.json"
    with open(manifest_path, "w") as f:
        json.dump(
            {
                "model": MODEL,
                "image_size": IMAGE_SIZE,
                "clips": results,
            },
            f,
            indent=2,
        )
    print(f"\n[MANIFEST] Saved to {manifest_path}")

    # Summary
    print(f"\n{'='*70}")
    print("GENERATION COMPLETE")
    print(f"{'='*70}")
    print(f"Clips generated: {len(results)}")
    print(f"Output directory: {args.output_dir}")
    print(f"\nTo assemble final video:")
    print(f"  python {__file__} --api-key YOUR_KEY --assemble")


if __name__ == "__main__":
    main()
