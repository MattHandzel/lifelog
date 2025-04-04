import argparse
import os
import re
from datetime import datetime
from moviepy.video.io.ImageSequenceClip import ImageSequenceClip
import pytz

def parse_arguments():
    parser = argparse.ArgumentParser(description='Create video from timestamped images')
    parser.add_argument('--input', required=True, help='Input directory containing images')
    parser.add_argument('--output', required=True, help='Output video filename (e.g., output.mp4)')
    parser.add_argument('--timezone', required=True, 
                      help='Timezone for image timestamps (e.g., America/Chicago)')
    parser.add_argument('--start', type=float, 
                      help='Start timestamp in epoch seconds (default: 0)', default=0.0)
    parser.add_argument('--end', type=float, 
                      help='End timestamp in epoch seconds (default: infinity)', default=float('inf'))
    parser.add_argument('--duration', type=float, default=5.0,
                      help='Duration per image in seconds (default: 5)')
    parser.add_argument('--fps', type=int, default=24,
                      help='Frames per second for output video (default: 24)')
    return parser.parse_args()

def main():
    args = parse_arguments()
    
    try:
        tz = pytz.timezone(args.timezone)
    except pytz.exceptions.UnknownTimeZoneError:
        print(f"Error: Unknown timezone '{args.timezone}'. Use format like 'America/Chicago'")
        return

    image_data = []
    pattern = re.compile(r'^(\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}\.\d{3}).*\.(png|jpg|jpeg)$', re.IGNORECASE)


    frames_per_image = int(round(args.duration * args.fps))
    assert frames_per_image > 0, f"Frames per image is too small, this means your duration or fps need to increase. Right now, you have {args.duration * args.fps} frames per image"

    for filename in os.listdir(args.input):
        if not filename.lower().endswith(('.png', '.jpg', '.jpeg')):
            continue
            
        match = pattern.match(filename)
        if not match:
            continue

            
        try:
            dt_naive = datetime.strptime(match.group(1), "%Y-%m-%d_%H-%M-%S.%f")
        except ValueError:
            print("Found a file with an invalid timestamp format: {filename}")
            continue

        try:
            dt_aware = tz.localize(dt_naive, is_dst=None)
        except pytz.exceptions.AmbiguousTimeError:
            dt_aware = tz.localize(dt_naive, is_dst=False)
        except pytz.exceptions.NonExistentTimeError:
            continue

        epoch_time = dt_aware.timestamp()
        if args.start <= epoch_time <= args.end:
            image_data.append((epoch_time, os.path.join(args.input, filename)))

    if not image_data:
        print("No images found in the specified time range")
        return

    image_data.sort()
    print(f"Found {len(image_data)} images in the specified time range")
    sorted_paths = [img[1] for img in image_data]
    
    # Calculate frame repetition for desired duration
    video_frames = [path for path in sorted_paths for _ in range(frames_per_image)]
    
    clip = ImageSequenceClip(video_frames, fps=args.fps)
    clip.write_videofile(args.output)
    print(f"Video created successfully: {args.output}")

if __name__ == "__main__":
    main()
