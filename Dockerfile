# ==========================================
# STAGE 1: Build the Vite React Frontend
# ==========================================
FROM node:18-alpine AS build-stage
WORKDIR /app/frontend

# Install dependencies first (caches this step if package.json doesn't change)
COPY frontend/package*.json ./
RUN npm install

# Copy frontend source and build
COPY frontend/ .
RUN npm run build

# ==========================================
# STAGE 2: Setup Flask Runtime
# ==========================================
FROM python:3.11-slim
WORKDIR /app/backend

# Copy the compiled React UI from Stage 1
COPY --from=build-stage /app/frontend/dist /app/backend/dist

# Install Python dependencies
COPY backend/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy Flask app source
COPY backend/ .

# Expose the port Google Cloud Run expects
EXPOSE 8080

# Use Gunicorn to run the Flask app (production grade)
CMD ["gunicorn", "--bind", "0.0.0.0:8080", "--workers", "1", "--threads", "8", "--timeout", "120", "api:app"]
