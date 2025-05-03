from fastapi import FastAPI
from src.api.embeddings import router as embeddings_router

app = FastAPI()

app.include_router(embeddings_router, prefix="/embeddings", tags=["embeddings"])
