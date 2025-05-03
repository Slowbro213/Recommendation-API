from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
from typing import List
from src.services.embedding_service import generate_embeddings, store_embeddings
from src.core.redis_client import redis_client

router = APIRouter()

class EmbeddingItem(BaseModel):
    post_id: str
    text: str

class EmbeddingRequest(BaseModel):
    items: List[EmbeddingItem]

@router.post("/generate")
def generate_and_store_embeddings(request: EmbeddingRequest):
    try:
        post_ids = [item.post_id for item in request.items]
        texts = [item.text for item in request.items]
        embeddings = generate_embeddings(texts)
        store_embeddings(redis_client, post_ids, texts, embeddings)
        return {"message": "Embeddings stored successfully.", "count": len(embeddings)}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
