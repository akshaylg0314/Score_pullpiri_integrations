#!/usr/bin/env python3

import json
import time
import os
from datetime import datetime
from threading import Thread, Lock
from flask import Flask, jsonify
from flask_cors import CORS
import logging

# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = Flask(__name__)
CORS(app)  # Enable CORS for all routes

# Global variables for message state
current_message = None
message_lock = Lock()
last_file_modified = 0

# Configuration
MESSAGE_FILE_PATH = "/tmp/driver_distraction/driver_distraction_messages.json"
CHECK_INTERVAL = 0.5  # Check file every 500ms
MESSAGE_TIMEOUT = 5.0  # Clear message after 5 seconds

class FileMonitor:
    def __init__(self):
        self.running = True
        
    def start_monitoring(self):
        """Start monitoring the message file for changes"""
        thread = Thread(target=self._monitor_file, daemon=True)
        thread.start()
        logger.info(f"📁 Started monitoring file: {MESSAGE_FILE_PATH}")
        
    def _monitor_file(self):
        """Monitor file for changes and update current message"""
        global current_message, last_file_modified
        
        while self.running:
            try:
                if os.path.exists(MESSAGE_FILE_PATH):
                    # Get file modification time
                    current_modified = os.path.getmtime(MESSAGE_FILE_PATH)
                    
                    # If file was modified, read new content
                    if current_modified != last_file_modified:
                        last_file_modified = current_modified
                        
                        with open(MESSAGE_FILE_PATH, 'r') as f:
                            content = f.read().strip()
                            
                        if content:
                            try:
                                new_message = json.loads(content)
                                new_message['file_modified_at'] = current_modified
                                
                                with message_lock:
                                    # Priority logic: if a 10s message, always prefer it
                                    if current_message:
                                        # If current is 10s and new is 5s, ignore new
                                        if (
                                            current_message.get('scenario_name', '').endswith('10sec') and
                                            new_message.get('scenario_name', '').endswith('5sec')
                                        ):
                                            logger.info("🔴 Ignoring 5s message because 10s message is active")
                                            pass  # keep current_message
                                        # If current is 5s and new is 10s, replace
                                        elif (
                                            current_message.get('scenario_name', '').endswith('5sec') and
                                            new_message.get('scenario_name', '').endswith('10sec')
                                        ):
                                            logger.info("🟢 Replacing 5s message with 10s emergency message")
                                            current_message = new_message
                                        # If both are same type, or only one present, update
                                        else:
                                            current_message = new_message
                                    else:
                                        current_message = new_message
                                    
                                logger.info(f"📄 New message loaded: {new_message.get('scenario_name', 'Unknown')} - {new_message.get('content', 'No content')}")
                                
                            except json.JSONDecodeError as e:
                                logger.error(f"❌ Failed to parse JSON from file: {e}")
                    
                    # Check if current message should be expired (older than timeout)
                    if current_message:
                        message_age = time.time() - current_message.get('file_modified_at', 0)
                        if message_age > MESSAGE_TIMEOUT:
                            with message_lock:
                                if current_message:  # Double check in case it was cleared
                                    logger.info(f"⏰ Message expired after {message_age:.1f}s, clearing for UI reset")
                                    current_message = None
                else:
                    # File doesn't exist, ensure current_message is None
                    with message_lock:
                        if current_message is not None:
                            logger.info("📭 Message file not found, clearing current message")
                            current_message = None
                            
            except Exception as e:
                logger.error(f"❌ Error monitoring file: {e}")
                
            time.sleep(CHECK_INTERVAL)

# REST API Endpoints
@app.route('/data', methods=['GET'])
def get_data():
    """Get the latest driver distraction message"""
    with message_lock:
        if current_message:
            # Return the message data (without internal file metadata)
            response_data = {k: v for k, v in current_message.items() if k != 'file_modified_at'}
            logger.info(f"🌐 Returning latest message: {response_data.get('scenario_name', 'Unknown')}")
            return jsonify(response_data)
        else:
            logger.info("📭 No messages available")
            return jsonify({"error": "No messages received yet"})

@app.route('/health', methods=['GET'])
def get_health():
    """Get health status of the file monitoring backend"""
    with message_lock:
        has_data = current_message is not None
        data_age = 0
        
        if has_data:
            data_age = time.time() - current_message.get('file_modified_at', 0)
    
    health_data = {
        "status": "healthy",
        "service": "file-monitoring-backend",
        "file_path": MESSAGE_FILE_PATH,
        "file_exists": os.path.exists(MESSAGE_FILE_PATH),
        "data_available": has_data,
        "data_age_seconds": round(data_age, 1) if has_data else 0,
        "timeout_threshold_seconds": MESSAGE_TIMEOUT,
        "last_check": datetime.now().isoformat()
    }
    
    return jsonify(health_data)

@app.route('/status', methods=['GET'])
def get_status():
    """Get detailed system status"""
    with message_lock:
        status_data = {
            "monitoring": True,
            "file_path": MESSAGE_FILE_PATH,
            "file_exists": os.path.exists(MESSAGE_FILE_PATH),
            "current_message": current_message if current_message else None,
            "check_interval": CHECK_INTERVAL,
            "message_timeout": MESSAGE_TIMEOUT,
            "system_time": datetime.now().isoformat()
        }
    
    return jsonify(status_data)

@app.route('/clear', methods=['POST'])
def clear_message():
    """Manually clear the current message (for testing)"""
    global current_message
    
    with message_lock:
        current_message = None
        
    logger.info("🧹 Message manually cleared")
    return jsonify({"status": "cleared", "message": "Current message cleared successfully"})

if __name__ == '__main__':
    logger.info("🚀 Starting File Monitoring Backend")
    logger.info(f"📁 Monitoring file: {MESSAGE_FILE_PATH}")
    logger.info(f"⏱️  Check interval: {CHECK_INTERVAL}s")
    logger.info(f"⏰ Message timeout: {MESSAGE_TIMEOUT}s")
    logger.info("🌐 REST API will be available on http://127.0.0.1:8089")
    
    # Start file monitoring in background
    monitor = FileMonitor()
    monitor.start_monitoring()
    
    # Start Flask app
    app.run(host='192.168.2.177', port=8089, debug=False)