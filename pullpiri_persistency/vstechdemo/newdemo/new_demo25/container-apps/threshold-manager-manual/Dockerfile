FROM python:3.11-slim

WORKDIR /app

COPY threshold_manager_manual.py .
RUN chmod +x threshold_manager_manual.py

RUN mkdir -p /data
VOLUME ["/data"]

ENTRYPOINT ["python3", "/app/threshold_manager_manual.py"]