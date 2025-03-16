# Mostly generated with deepseek
# usage: `python symbol_detector.py "screenshot.png"`
#    or: `python symbol_detector.py "screenshot.png" debug` - to save screenshot with marked symbols

import cv2
import numpy as np
import json
import sys
import time
from multiprocessing import cpu_count
from concurrent.futures import ProcessPoolExecutor

screenshot = cv2.imread(sys.argv[1])
screenshot = cv2.resize(screenshot, (0, 0), fx = 0.5, fy = 0.5, interpolation = cv2.INTER_NEAREST)

# Load symbol templates and convert to grayscale
SYMBOL_TEMPLATES = {
    'gold': cv2.imread('symbols/symbol_gold.png', cv2.IMREAD_COLOR),
    'air_a': cv2.imread('symbols/symbol_air_a.png', cv2.IMREAD_COLOR),
    'fire_a': cv2.imread('symbols/symbol_fire_a.png', cv2.IMREAD_COLOR),
    'water_a': cv2.imread('symbols/symbol_water_a.png', cv2.IMREAD_COLOR),
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
    'essence': cv2.imread('symbols/symbol_essence.png', cv2.IMREAD_COLOR),
    'essence_a': cv2.imread('symbols/symbol_essence_a.png', cv2.IMREAD_COLOR),
}

CONFIDENCE_THRESHOLD = 0.7



def process_template(args):
    symbol_name, screenshot = args
    template = SYMBOL_TEMPLATES[symbol_name]
    template = cv2.resize(template, (0, 0), fx = 0.5, fy = 0.5, interpolation = cv2.INTER_NEAREST)
    # SYMBOL_TEMPLATES[symbol_name] = template
    result = cv2.matchTemplate(screenshot, template, cv2.TM_CCOEFF_NORMED, None, template)
    
    # bug in opencv? https://github.com/opencv/opencv/issues/23257
    result[np.isinf(result)] = 0

    locations = np.where(result >= CONFIDENCE_THRESHOLD)

    detected_symbols = []
    for (x, y) in zip(*locations[::-1]):
        detected_symbols.append({
            'symbol': symbol_name,
            'location': (x, y),
            'confidence': result[y, x],
            'size': template.shape[:2][::-1]  # (w, h)
        })

    return detected_symbols


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
        _, w, h = SYMBOL_TEMPLATES[detection['symbol']].shape[::-1]
        boxes.append([x, y, x + w, y + h, detection['confidence']])

    boxes = np.array(boxes)
    boxes = np.array([(d['location'][0], d['location'][1], 
                     d['size'][0], d['size'][1]) for d in detections])
    scores = np.array([d['confidence'] for d in detections])

    indices = cv2.dnn.NMSBoxes(boxes, scores, CONFIDENCE_THRESHOLD, 0.5)
    return [detections[i] for i in indices] if indices is not None else []

if __name__ == "__main__":
    with ProcessPoolExecutor(cpu_count()) as executor:
        args = [(name, screenshot) for name in SYMBOL_TEMPLATES]
        results = list(executor.map(process_template, args))

    all_detections = [d for sublist in results for d in sublist]
    detected_symbols = non_max_suppression(all_detections)

    # Draw rectangles around detected symbols
    if len(sys.argv) > 2 and sys.argv[2] == 'debug':
        for detection in detected_symbols:
            x, y = detection['location']
            _, w, h = SYMBOL_TEMPLATES[detection['symbol']].shape[::-1]
            cv2.rectangle(screenshot, (x, y), (x + w, y + h), (0, 255, 0), 2)
            cv2.putText(screenshot, detection['symbol'] + " " + str(round(detection['confidence'], 2)), (x, y - 10), cv2.FONT_HERSHEY_SIMPLEX, 0.5, (0, 255, 0), 2)
        cv2.imwrite('detected_symbols.png', screenshot)

    result = {
        'symbols': []
    }

    for detection in detected_symbols:
        x, y = detection['location']
        _, w, h = SYMBOL_TEMPLATES[detection['symbol']].shape[::-1]

        center_x = float(x + w / 2.0)
        center_y = float(y + h / 2.0)

        result['symbols'].append({
            'x': center_x, 
            'y': center_y,
            'name': detection['symbol'],
            'confidence': round(float(detection['confidence']), 2)
        })

    print(json.dumps(result, ensure_ascii=False))