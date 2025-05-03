from sentence_transformers import SentenceTransformer
from sklearn.preprocessing import normalize
import json

model = SentenceTransformer('all-MiniLM-L6-v2')

def generate_embeddings(texts: list[str]) -> list[list[float]]:
    embeddings = model.encode(texts)
    return normalize(embeddings).tolist()

def store_embeddings(redis_client, post_ids: list[str], texts: list[str], embeddings: list[list[float]]):
    for post_id, text, embedding in zip(post_ids, texts, embeddings):
        redis_client.set(f"post:{post_id}", text)
        redis_client.set(f"embedding:post:{post_id}", json.dumps(embedding))
