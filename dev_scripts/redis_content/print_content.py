import redis
import json

r = redis.Redis(host='localhost', port=6379, db=0)

keys = r.keys('*')
print(f"Found {len(keys)} vector(s).")

for key in keys:
    value = r.get(key)
    if not value:
        print(f"{key.decode()}: [empty or null value]")
        continue

    try:
        vec = json.loads(value)
        print(f"{key.decode()}: {vec[:5]}...")  # print only the first 5 elements for readability
    except json.JSONDecodeError:
        print(f"{key.decode()}: [invalid JSON] {value}")
