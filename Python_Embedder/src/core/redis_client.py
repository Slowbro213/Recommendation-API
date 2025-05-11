import redis
import os
from dotenv import load_dotenv

# Load environment variables from .env file
load_dotenv()



redis_client = redis.Redis(host=os.getenv("REDIS_HOST","redis"), port=os.getenv("REDIS_PORT",6379), db=0)

