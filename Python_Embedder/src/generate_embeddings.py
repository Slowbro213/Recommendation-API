from sentence_transformers import SentenceTransformer
import redis
import json
from sklearn.preprocessing import normalize

# 1. Load a pre-trained Sentence Transformer model
model = SentenceTransformer('all-MiniLM-L6-v2')  # Or another suitable model

# 2. Load your text data
def load_data(filepath):
    """Loads text data from a file. Assumes one text per line."""
    with open(filepath, 'r') as f:
        texts = [line.strip() for line in f]
    return texts

# 3. Generate embeddings
def generate_embeddings(model, texts):
    """Generates embeddings for a list of texts using the Sentence Transformer model."""
    # Extract content (everything after the first comma)
    contents = [line.split(',')[1].strip(' " ') for line in texts]
    embeddings = model.encode(contents)

    # Normalize embeddings (optional but recommended for later processing)
    embeddings = normalize(embeddings)
    return embeddings

# 4. Store embeddings and text in Redis (for demonstration and persistence)
def store_embeddings_redis(redis_client, texts, embeddings):
    """Stores the texts and their corresponding embeddings in Redis."""
    for (text, embedding) in zip(texts, embeddings):
        post_id = f"post:{text.split(',')[0]}"  # Extract post ID from the first part of the line
        content = text.split(',')[1].strip(' " ')  # Extract content
        # Store the text (optional, for demonstration)
        redis_client.set(post_id, content)
        # Store the embedding as a JSON string
        embedding_key = f"embedding:{post_id}"
        embedding_value = json.dumps(embedding.tolist())  # Convert to list before JSON
        redis_client.set(embedding_key, embedding_value)
        print(f"Stored embedding for post ID {post_id} in Redis")

    print("Embeddings stored in Redis.")

def main():
    """Main function to coordinate embedding generation and storage."""
    # Load data
    texts_file = 'data/posts.txt'  # Make sure this file exists
    texts = load_data(texts_file)
    
    # Generate embeddings
    embeddings = generate_embeddings(model, texts)

    # Initialize Redis client
    redis_client = redis.Redis(host='localhost', port=6379, db=0)  # Adjust as needed

    # Store embeddings in Redis
    store_embeddings_redis(redis_client, texts, embeddings)

    # Print the first embedding and text for verification
    print(f"First Text: {texts[0].split(',')[1]}")
    print(f"First Embedding: {embeddings[0][:5]}... (shape: {embeddings[0].shape[0]})")  # Print only the first 5 values

if __name__ == "__main__":
    main()
