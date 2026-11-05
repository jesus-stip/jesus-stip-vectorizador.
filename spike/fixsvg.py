# usage: python3 -I fixsvg.py <file.svg> <orig_w> <orig_h> : sets viewBox so a 2x trace renders at original size
import sys, re
p, w, h = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
m = re.search(r'<svg[^>]*width="(\d+)"[^>]*height="(\d+)"', s)
s = s.replace(m.group(0), m.group(0).replace(f'width="{m.group(1)}"', f'width="{w}" viewBox="0 0 {m.group(1)} {m.group(2)}"').replace(f'height="{m.group(2)}"', f'height="{h}"'), 1)
open(p, 'w').write(s)
