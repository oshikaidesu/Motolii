# The cat's outline, each picture of a film keyed on green (OpenCV): the outlines it is seen by, baked once to a file
# a picture a line ("t x y x y ... | x y ..."), in shares of the film's width and height, top left 0 0.
import sys, cv2, numpy as np
film, out = sys.argv[1], sys.argv[2]
cap = cv2.VideoCapture(film)
fps = cap.get(cv2.CAP_PROP_FPS)
with open(out, 'w') as f:
    f.write(f'fps {fps}\n')
    n = 0
    while True:
        ok, bgr = cap.read()
        if not ok: break
        h, w = bgr.shape[:2]
        hsv = cv2.cvtColor(bgr, cv2.COLOR_BGR2HSV)
        green = cv2.inRange(hsv, (35, 80, 60), (85, 255, 255))   # (the ground: green enough, bright enough)
        cat = cv2.morphologyEx(255 - green, cv2.MORPH_OPEN, np.ones((5, 5), np.uint8))
        rings, _ = cv2.findContours(cat, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        parts = []
        for r in rings:
            if cv2.contourArea(r) < 0.0005 * w * h: continue   # (specks)
            r = cv2.approxPolyDP(r, 0.004 * w, True).reshape(-1, 2)
            if len(r) >= 3: parts.append(' '.join(f'{x / w:.4f} {y / h:.4f}' for x, y in r))
        f.write(f'{n / fps:.4f} ' + ' | '.join(parts) + '\n')
        n += 1
print(n, 'pictures at', fps)
