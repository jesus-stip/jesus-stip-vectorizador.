# usage: python3 -I prequant.py <in.png> <out.png> <colors> <scale>
# spike-only stand-in for the browser preprocessing step:
# upscale (so thin strokes and small text survive), light denoise, snap to N flat colors, no dithering
import sys
from PIL import Image, ImageFilter
src, dst, n, scale = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
im = Image.open(src).convert('RGBA')
if scale > 1: im = im.resize((im.width * scale, im.height * scale), Image.LANCZOS)
alpha = im.getchannel('A').point(lambda a: 255 if a >= 128 else 0)
rgb = Image.new('RGB', im.size, 'white'); rgb.paste(im.convert('RGB'), mask=im.getchannel('A'))
rgb = rgb.filter(ImageFilter.MedianFilter(3))
q = rgb.quantize(colors=n, method=Image.Quantize.MEDIANCUT, kmeans=3, dither=Image.Dither.NONE).convert('RGBA')
q.putalpha(alpha); q.save(dst)
