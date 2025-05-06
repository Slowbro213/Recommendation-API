from fastapi import FastAPI
from src.api.embeddings import router as embeddings_router
from redis import asyncio as aioredis  # Use async Redis client
import asyncio
import signal
import sys
import os

app = FastAPI()
app.include_router(embeddings_router, prefix="/embeddings", tags=["embeddings"])

# Async Redis client
redis_client = aioredis.Redis(host="localhost", port=6379, db=0)

async def handle_pubsub_messages():
    try:
        pubsub = redis_client.pubsub()
        await pubsub.subscribe("new_embedding")

        while True:
            message = await pubsub.get_message(ignore_subscribe_messages=True, timeout=1.0)
            if message:
                post_id = message['data'].decode()
                print(f"[FastAPI] Received new post_id: {post_id}")
    except asyncio.CancelledError:
        print("PubSub task cancelled")
    except Exception as e:
        print(f"PubSub error: {e}")
    finally:
        await pubsub.unsubscribe()
        await pubsub.close()


async def listen_for_shutdown():
    try:
        pubsub = redis_client.pubsub()
        await pubsub.subscribe("shutdown")

        while True:
            message = await pubsub.get_message(ignore_subscribe_messages=True, timeout=1.0)
            if message:
                #shutdown the server 
                os.kill(os.getpid(), signal.SIGINT)
    except asyncio.CancelledError:
        print("PubSub task cancelled")
    except Exception as e:
        print(f"PubSub error: {e}")
    finally:
        await pubsub.unsubscribe()
        await pubsub.close()



async def graceful_shutdown():
    print("Shutting down gracefully...")
    await redis_client.publish("shutdown", "shutdown")
    await redis_client.close()

@app.on_event("startup")
async def startup_event():
    # Store the task to cancel it later
    app.state.pubsub_task = asyncio.create_task(handle_pubsub_messages())
    app.state.shutdown_task = asyncio.create_task(listen_for_shutdown())

@app.on_event("shutdown")
async def shutdown_event():
    await graceful_shutdown()
    if hasattr(app.state, 'pubsub_task'):
        app.state.pubsub_task.cancel()
        try:
            await app.state.pubsub_task
        except asyncio.CancelledError:
            pass
