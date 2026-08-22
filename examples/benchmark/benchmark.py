import time
import math

INPUT_SIZE = 1000
ITERATIONS = 1
SEED = 123456789

def generate_events():
    events = []
    seed = SEED
    i = 0
    
    while i < INPUT_SIZE:
        # Deterministic pseudo-random generation.
        seed = (seed * 1103515245 + 12345) & 0x7FFFFFFF
        user_id = seed % 100000
        x = math.sin(float(i)) * 100.0
        y = math.cos(float(i)) * 100.0

        # Simulating some CPU work
        transformed = math.sqrt(abs((x * 1.5) + (y / 2.0) - (x * 0.1)))
        s_mod = seed % 10000
        value = float(s_mod) / 100.0
        category = seed % 10
        timestamp = 1700000000 + i

        events.append((i, timestamp, user_id, value, category))
        i += 1
        
    return events

def process_events(events):
    count = 0
    total = 0.0
    minimum = float('inf')
    maximum = -float('inf')
    
    cat0 = 0.0
    cat1 = 0.0
    cat2 = 0.0
    cat3 = 0.0
    cat4 = 0.0
    cat5 = 0.0
    cat6 = 0.0
    cat7 = 0.0
    cat8 = 0.0
    cat9 = 0.0
    
    checksum = 0
    processed = []
    
    for event in events:
        event_id, timestamp, user_id, value, category = event
        
        # Filter
        if user_id % 2 != 0:
            continue
            
        # Transform
        transformed = value * 1.15 + float(category)
        
        # Hash/checksum
        t_val = transformed * 100.0
        t_millis = int(t_val)
        
        checksum = checksum ^ event_id ^ timestamp ^ user_id ^ t_millis ^ category
        
        # Statistics
        count += 1
        total += transformed
        minimum = min(minimum, transformed)
        maximum = max(maximum, transformed)
        
        if category == 0: cat0 += transformed
        elif category == 1: cat1 += transformed
        elif category == 2: cat2 += transformed
        elif category == 3: cat3 += transformed
        elif category == 4: cat4 += transformed
        elif category == 5: cat5 += transformed
        elif category == 6: cat6 += transformed
        elif category == 7: cat7 += transformed
        elif category == 8: cat8 += transformed
        elif category == 9: cat9 += transformed
            
        processed.append((transformed, event_id))
        
    category_totals = [cat0, cat1, cat2, cat3, cat4, cat5, cat6, cat7, cat8, cat9]
    return (count, total, minimum, maximum, category_totals, checksum, processed)

def main():
    complete_start = int(time.time() * 1000)
    
    # -----------------------------
    # Generate input
    # -----------------------------
    generation_start = int(time.time() * 1000)
    events = generate_events()
    generation_time = int(time.time() * 1000) - generation_start
    
    # -----------------------------
    # Benchmark processing
    # -----------------------------
    processing_start = int(time.time() * 1000)
    result = process_events(events)
    processing_time = int(time.time() * 1000) - processing_start
    
    # -----------------------------
    # Complete execution
    # -----------------------------
    complete_time = int(time.time() * 1000) - complete_start
    count, total, minimum, maximum, category_totals, checksum, processed = result
    
    print("========================================")
    print(" Python Benchmark")
    print("========================================")
    print(f"Input events       : {INPUT_SIZE}")
    print(f"Iterations         : {ITERATIONS}")
    print(f"Processed events   : {count}")
    print("")
    print(f"Total              : {total}")
    print(f"Minimum            : {minimum}")
    print(f"Maximum            : {maximum}")
    print(f"Checksum           : {checksum}")
    print("")
    print("Category totals:")
    
    for i in range(10):
        print(f"{i}: {category_totals[i]}")
        
    print("")
    print("Timing:")
    print(f"Generation         : {generation_time}ms")
    print(f"Processing         : {processing_time}ms")
    print(f"Complete execution : {complete_time}ms")
    print("========================================")

if __name__ == "__main__":
    main()
