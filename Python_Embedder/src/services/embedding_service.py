from sentence_transformers import SentenceTransformer
from sklearn.preprocessing import normalize
import json
import hashlib
import struct

model = SentenceTransformer('all-distilroberta-v1')

def generate_embeddings(texts: list[str]) -> list[list[float]]:
    embeddings = model.encode(texts)
    return normalize(embeddings).tolist()


def hash_embedding(embedding: list[float]) -> str:
    # Pack floats as binary little-endian f32
    binary = b''.join(struct.pack('<f', x) for x in embedding)
    return hashlib.sha256(binary).hexdigest()


def store_embeddings(redis_client, post_ids: list[str], texts: list[str], embeddings: list[list[float]]):
    for post_id, text, embedding in zip(post_ids, texts, embeddings):
        # Forward mapping
        redis_client.set(f"post:{post_id}", text)
        redis_client.set(f"embedding:post:{post_id}", json.dumps(embedding))

        # Reverse mapping
        emb_hash = hash_embedding(embedding)
        print(f"Hash for embedding: {emb_hash}")
        redis_client.set(f"post_from_embedding:{emb_hash}", post_id)

        # Notify listeners
        redis_client.publish("new_embedding", post_id)
