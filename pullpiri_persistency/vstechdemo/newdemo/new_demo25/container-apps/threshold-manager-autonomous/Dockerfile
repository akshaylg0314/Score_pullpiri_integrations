FROM python:3.11-slim

# Set working directory
WORKDIR /app

# Copy the threshold manager script
COPY threshold_manager.py .

# Make script executable
RUN chmod +x threshold_manager.py

# Create data directory for thresholds
RUN mkdir -p /data

# Set volume for persistent threshold storage
VOLUME ["/data"]

# Set entrypoint
ENTRYPOINT ["python3", "/app/threshold_manager.py"]
