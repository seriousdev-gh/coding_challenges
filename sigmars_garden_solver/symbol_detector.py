# Generated with deepseek
# usage: `python symbol_detector.py "screenshot.png"`
#    or: `python symbol_detector.py "screenshot.png" debug` - to save screenshot with marked symbols

import cv2
import numpy as np
import json
import sys

screenshot = cv2.imread(sys.argv[1])
screenshot_gray = screenshot
# screenshot_gray = cv2.cvtColor(screenshot, cv2.COLOR_BGR2GRAY)

# clahe_screenshot = cv2.createCLAHE(clipLimit=2.0, tileGridSize=(8, 8))
# screenshot_gray = clahe_screenshot.apply(screenshot_gray)
cv2.imwrite('clahe/screenshot.png', screenshot_gray)

# clahe = cv2.createCLAHE(clipLimit=2.0, tileGridSize=(1, 1))


# Load symbol templates and convert to grayscale
symbol_templates = {
    'gold': cv2.imread('symbols/symbol_gold.png', cv2.IMREAD_COLOR),
    # 'air': cv2.imread('symbols/symbol_air.png', cv2.IMREAD_COLOR),
    'air_a': cv2.imread('symbols/symbol_air_a.png', cv2.IMREAD_COLOR),
    # 'fire': cv2.imread('symbols/symbol_fire.png', cv2.IMREAD_COLOR),
    'fire_a': cv2.imread('symbols/symbol_fire_a.png', cv2.IMREAD_COLOR),
    # 'water': cv2.imread('symbols/symbol_water.png', cv2.IMREAD_COLOR),
    'water_a': cv2.imread('symbols/symbol_water_a.png', cv2.IMREAD_COLOR),
    # 'earth': cv2.imread('symbols/symbol_earth.png', cv2.IMREAD_COLOR),
    'earth_a': cv2.imread('symbols/symbol_earth_a.png', cv2.IMREAD_COLOR),
    'life': cv2.imread('symbols/symbol_life.png', cv2.IMREAD_COLOR),
    'life_a': cv2.imread('symbols/symbol_life_a.png', cv2.IMREAD_COLOR),
    'death': cv2.imread('symbols/symbol_death.png', cv2.IMREAD_COLOR),
    'lead': cv2.imread('symbols/symbol_lead.png', cv2.IMREAD_COLOR),
    'tin': cv2.imread('symbols/symbol_tin.png', cv2.IMREAD_COLOR),
    'iron': cv2.imread('symbols/symbol_iron.png', cv2.IMREAD_COLOR),
    'copper': cv2.imread('symbols/symbol_copper.png', cv2.IMREAD_COLOR),
    'silver': cv2.imread('symbols/symbol_silver.png', cv2.IMREAD_COLOR),
    'silver_a': cv2.imread('symbols/symbol_silver_a.png', cv2.IMREAD_COLOR),
    'mercury': cv2.imread('symbols/symbol_mercury.png', cv2.IMREAD_COLOR),
    'salt': cv2.imread('symbols/symbol_salt.png', cv2.IMREAD_COLOR),
}


confidence_threshold = 0.7

detected_symbols = []

for symbol_name, template in symbol_templates.items():
    # template = clahe.apply(template)

    # cv2.imwrite('clahe/' + symbol_name + '.png', template)

    result = cv2.matchTemplate(screenshot_gray, template, cv2.TM_CCOEFF_NORMED, None, template)
    locations = np.where(result >= confidence_threshold)

    for (x, y) in zip(*locations[::-1]):  # Swap x and y for OpenCV coordinates
        detected_symbols.append({
            'symbol': symbol_name,
            'location': (x, y),
            'confidence': result[y, x]
        })

def non_max_suppression(detections, overlap_threshold=0.5):
    """
    Apply non-maximum suppression to avoid duplicate detections.
    """
    if len(detections) == 0:
        return []

    # Convert detections to a list of bounding boxes
    boxes = []
    for detection in detections:
        x, y = detection['location']
        _, w, h = symbol_templates[detection['symbol']].shape[::-1]
        boxes.append([x, y, x + w, y + h, detection['confidence']])

    boxes = np.array(boxes)

    # Apply non-maximum suppression
    pick = []
    x1 = boxes[:, 0]
    y1 = boxes[:, 1]
    x2 = boxes[:, 2]
    y2 = boxes[:, 3]
    scores = boxes[:, 4]

    area = (x2 - x1 + 1) * (y2 - y1 + 1)
    idxs = np.argsort(scores)[::-1]

    while len(idxs) > 0:
        i = idxs[0]
        pick.append(i)
        xx1 = np.maximum(x1[i], x1[idxs[1:]])
        yy1 = np.maximum(y1[i], y1[idxs[1:]])
        xx2 = np.minimum(x2[i], x2[idxs[1:]])
        yy2 = np.minimum(y2[i], y2[idxs[1:]])

        w = np.maximum(0, xx2 - xx1 + 1)
        h = np.maximum(0, yy2 - yy1 + 1)

        overlap = (w * h) / area[idxs[1:]]

        idxs = idxs[np.where(overlap <= overlap_threshold)[0] + 1]

    return [detections[i] for i in pick]

# Apply NMS to detected symbols
detected_symbols = non_max_suppression(detected_symbols)


# Draw rectangles around detected symbols
if len(sys.argv) > 2 and sys.argv[2] == 'debug':
    for detection in detected_symbols:
        x, y = detection['location']
        _, w, h = symbol_templates[detection['symbol']].shape[::-1]
        cv2.rectangle(screenshot, (x, y), (x + w, y + h), (0, 255, 0), 2)
        cv2.putText(screenshot, detection['symbol'] + " " + str(round(detection['confidence'], 2)), (x, y - 10), cv2.FONT_HERSHEY_SIMPLEX, 0.5, (0, 255, 0), 2)
    cv2.imwrite('detected_symbols.png', screenshot)

result = {
    'symbols': [],
    'width': int(screenshot.shape[1]),  
    'height': int(screenshot.shape[0]),
}

for detection in detected_symbols:
    x, y = detection['location']
    _, w, h = symbol_templates[detection['symbol']].shape[::-1]

    center_x = float(x + w / 2.0)
    center_y = float(y + h / 2.0)

    result['symbols'].append({
        'x': center_x, 
        'y': center_y,
        'name': detection['symbol'],
        'confidence': round(float(detection['confidence']), 2)
    })

print(json.dumps(result, ensure_ascii=False))