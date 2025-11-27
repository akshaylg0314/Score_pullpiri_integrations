import uvicorn
import asyncio
import psutil
import subprocess
import json
import sqlite3
import datetime # <-- 1. IMPORT DATETIME
import os
import signal # <-- Import signal for SIGINT
import sys
import argparse
import socket
from fastapi import FastAPI, HTTPException
from fastapi.responses import StreamingResponse, FileResponse # <-- 2. IMPORT FILERESPONSE
from pydantic import BaseModel
from typing import Dict, List, Optional
from fastapi.middleware.cors import CORSMiddleware


def get_ip_from_interface(interface_name):
    """Get IP address from network interface"""
    try:
        import netifaces
        addresses = netifaces.ifaddresses(interface_name)
        ipv4_info = addresses.get(netifaces.AF_INET)
        if ipv4_info:
            return ipv4_info[0]['addr']
    except ImportError:
        # Fallback method without netifaces
        try:
            result = subprocess.run(['ip', 'addr', 'show', interface_name], 
                                  capture_output=True, text=True, check=True)
            for line in result.stdout.split('\n'):
                if 'inet ' in line and not '127.0.0.1' in line:
                    ip = line.strip().split()[1].split('/')[0]
                    return ip
        except subprocess.CalledProcessError:
            pass
    
    raise ValueError(f"Could not get IP address from interface '{interface_name}'. Interface may not exist or have no IP assigned.")


def validate_executable_paths():
    """Validate that all executable paths exist"""
    missing_executables = []
    
    for name, command in COMMAND_CONFIG.items():
        executable_path = command[0]
        if not os.path.exists(executable_path):
            missing_executables.append(f"{name}: {executable_path}")
        elif not os.access(executable_path, os.X_OK):
            missing_executables.append(f"{name}: {executable_path} (not executable)")
    
    if missing_executables:
        print("ERROR: The following executables are missing or not executable:")
        for missing in missing_executables:
            print(f"  - {missing}")
        print("\nPlease ensure all executable files are present and have execute permissions.")
        sys.exit(1)
    
    print("✓ All executable paths validated successfully")


def ensure_logs_directory():
    """Create logs directory if it doesn't exist"""
    logs_dir = "./logs"
    if not os.path.exists(logs_dir):
        os.makedirs(logs_dir)
        print(f"✓ Created logs directory: {logs_dir}")
    else:
        print(f"✓ Logs directory already exists: {logs_dir}")


def parse_arguments():
    """Parse command line arguments"""
    parser = argparse.ArgumentParser(description='ADAS Server')
    parser.add_argument('--interface', '-i', required=True,
                       help='Network interface to get IP address from (e.g., eth0, wlan0)')
    return parser.parse_args()


app = FastAPI()

# ==============================================================================
# == CONFIGURATION - FINAL (with local logs fix)
# ==============================================================================

# 1. EXECUTABLE COMMANDS
COMMAND_CONFIG = {
    "tracer": [
        './executables/feo_tracer',
        '-o', 
        './logs/trace.log',
        '-l', 
        'debug'
    ],
    "logd": [
        './executables/logd'  # Captures all ADAS process logs
    ],
    "adas_primary": [
        './executables/adas_primary',
        '4000'  # cycle time in milliseconds
    ],
    "adas_secondary_1": [
        './executables/adas_secondary',
        '1'  # secondary index
    ],
    "adas_secondary_2": [
        './executables/adas_secondary',
        '2'  # secondary index
    ]
}

# 2. LOG FILE PATHS
LOG_FILE_PATHS = {
    "logd": "./logs/logd_redirect.log",  # logd captures all ADAS process logs
    "tracer": "./logs/trace.log",
}

# 3. PERSISTENCY DB
PERSISTENCY_FILE_PATH = "./adas_data/kvs_0_0.json"
PERSISTENCY_FILE_TYPE = "json" 

# 4. NETWORK CONFIG
# SERVER_HOST_IP will be set dynamically from command line interface argument
SERVER_HOST_IP = None  # Will be set in main
ALLOWED_ORIGINS = ["*"] 

# ==============================================================================
# == APPLICATION LOGIC
# ==============================================================================

tracked_processes: Dict[str, Dict] = {}

def launch_process(name: str, command: List[str]) -> psutil.Process:
    """Helper function to start and track a process."""
    if name in tracked_processes:
        raise Exception(f"Process '{name}' is already running.")
    
    print(f"Launching '{name}' from: {' '.join(command)}")
    proc = subprocess.Popen(command)
    psutil_proc = psutil.Process(proc.pid)
    
    tracked_processes[name] = {
        "process": psutil_proc,
        "log_handle": None 
    }
    return psutil_proc

# --- 1. Startup and Process Control ---

@app.on_event("startup")
async def launch_startup_processes():
    """Launches ALL ADAS processes when the server starts."""
    print("Server starting... launching all ADAS processes.")
    
    # Define startup order (with delays for proper initialization)
    startup_sequence = [
        ("logd", 0),
        ("adas_primary", 2),
        ("adas_secondary_1", 4),
        ("adas_secondary_2", 4),
    ]
    
    try:
        for name, delay in startup_sequence:
            if delay > 0:
                print(f"Waiting {delay}s before launching {name}...")
                await asyncio.sleep(delay)
            
            command = COMMAND_CONFIG[name]
            log_path = LOG_FILE_PATHS.get(name)
            
            if log_path:
                print(f"Launching '{name}' with stdout redirection to {log_path}")
                log_file_handle = open(log_path, "wb")
                proc = subprocess.Popen(command, stdout=log_file_handle, stderr=subprocess.STDOUT)
            else:
                print(f"Launching '{name}' without log redirection")
                proc = subprocess.Popen(command)
                log_file_handle = None
            
            psutil_proc = psutil.Process(proc.pid)
            tracked_processes[name] = {
                "process": psutil_proc,
                "log_handle": log_file_handle
            }
            print(f"✓ {name} started (PID: {proc.pid})")

    except Exception as e:
        print(f"FATAL: Could not launch startup processes: {e}")
        import traceback
        traceback.print_exc()
    
    print(f"Startup complete. Tracking: {list(tracked_processes.keys())}")

@app.on_event("shutdown")
async def shutdown_event():
    """Cleans up all child processes on server shutdown."""
    print("Server shutting down... terminating all tracked processes.")
    for name, proc_data in list(tracked_processes.items()):
        try:
            print(f"Stopping process: {name} (PID: {proc_data['process'].pid})")
            proc = proc_data["process"]
            proc.send_signal(signal.SIGINT)  # Send SIGINT for graceful shutdown
            proc.wait(timeout=5)  # Wait for process to terminate and reap zombie
            
            if proc_data["log_handle"]:
                proc_data["log_handle"].close()
            
            del tracked_processes[name]

        except psutil.NoSuchProcess:
            print(f"Process {name} was already gone.")
        except psutil.TimeoutExpired:
            print(f"Process {name} did not terminate gracefully, killing it.")
            proc.kill()  # Force kill with SIGKILL
            proc.wait()  # Reap the zombie process
            if proc_data["log_handle"]:
                proc_data["log_handle"].close()
            del tracked_processes[name]
        except Exception as e:
            print(f"Error during shutdown for {name}: {e}")
            
    print("Cleanup complete.")

# (All other endpoints like /start, /stop, /status are unchanged)
class StartRequest(BaseModel):
    name: str 

@app.post("/api/processes/start")
async def start_process_api():
    """Executes demo_apply.sh script for system updates."""
    script_path = "./executables/examples/demo_apply.sh"
    
    if not os.path.exists(script_path):
        raise HTTPException(status_code=404, detail=f"Script not found: {script_path}")
    
    if not os.access(script_path, os.X_OK):
        raise HTTPException(status_code=403, detail=f"Script is not executable: {script_path}")
    
    try:
        print(f"Executing update script: {script_path}")
        result = subprocess.run(
            [script_path],
            capture_output=True,
            text=True,
            timeout=60  # 60 second timeout
        )
        
        return {
            "status": "success" if result.returncode == 0 else "failed",
            "message": "update.sh executed",
            "returncode": result.returncode,
            "stdout": result.stdout,
            "stderr": result.stderr
        }
    except subprocess.TimeoutExpired:
        raise HTTPException(status_code=408, detail="Script execution timed out")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Script execution failed: {e}")

class StopRequest(BaseModel):
    name: str

@app.post("/api/processes/stop")
async def stop_process_api():
    """Executes delete.sh script for cleanup operations."""
    script_path = "./executables/examples/demo_delete.sh"
    
    if not os.path.exists(script_path):
        raise HTTPException(status_code=404, detail=f"Script not found: {script_path}")
    
    if not os.access(script_path, os.X_OK):
        raise HTTPException(status_code=403, detail=f"Script is not executable: {script_path}")
    
    try:
        print(f"Executing delete script: {script_path}")
        result = subprocess.run(
            [script_path],
            capture_output=True,
            text=True,
            timeout=60  # 60 second timeout
        )
        
        return {
            "status": "success" if result.returncode == 0 else "failed",
            "message": "delete.sh executed",
            "returncode": result.returncode,
            "stdout": result.stdout,
            "stderr": result.stderr
        }
    except subprocess.TimeoutExpired:
        raise HTTPException(status_code=408, detail="Script execution timed out")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Script execution failed: {e}")

@app.get("/api/processes/status")
async def get_status_api():
    statuses = []
    for name, proc_data in list(tracked_processes.items()):
        try:
            proc = proc_data["process"]
            statuses.append({
                "name": name,
                "pid": proc.pid,
                "status": proc.status() 
            })
        except psutil.NoSuchProcess:
            if proc_data["log_handle"]:
                proc_data["log_handle"].close()
            del tracked_processes[name]
    return statuses

# --- 2. System and Process Metrics APIs ---

@app.get("/api/metrics/all")
async def get_all_metrics():
    """Gets overall system metrics AND all tracked process metrics."""
    
    # 3. ADD TIMESTAMP
    timestamp = datetime.datetime.now().isoformat()
    
    mem = psutil.virtual_memory()
    net_io = psutil.net_io_counters()
    
    process_metrics = []
    for name, proc_data in list(tracked_processes.items()):
        try:
            proc = proc_data["process"]
            process_metrics.append({
                "name": name,
                "pid": proc.pid,
                "cpu_percent": proc.cpu_percent(interval=None),
                "memory_mb": proc.memory_info().rss / (1024 * 1024),
                "network_io_rate": 0.0,  # Set to 0.0 for individual processes
                "timestamp": timestamp # <-- ADDED
            })
        except psutil.NoSuchProcess:
            if proc_data["log_handle"]:
                proc_data["log_handle"].close()
            del tracked_processes[name]
        except Exception:
            pass 
    
    return {
        "system": {
            "cpu_percent": psutil.cpu_percent(),
            "memory_percent": mem.percent,
            "network_bytes_sent": net_io.bytes_sent if net_io else 0,
            "network_bytes_recv": net_io.bytes_recv if net_io else 0,
            "timestamp": timestamp # <-- ADDED
        },
        "processes": process_metrics
    }

# --- 3. Logging and Persistency APIs ---

@app.get("/api/logs/stream")
async def get_log_file(log_name: str, limit: int = 100):
    """Reads the last N lines of a specified log file."""
    if log_name not in LOG_FILE_PATHS:
        raise HTTPException(status_code=404, detail=f"No log file path configured for '{log_name}'. Available logs are: {list(LOG_FILE_PATHS.keys())}")
    
    file_path = LOG_FILE_PATHS[log_name]
    
    try:
        with open(file_path, "r") as f:
            lines = f.readlines()[-limit:]
        return {"log_name": log_name, "lines": [line.strip() for line in lines]}
    except FileNotFoundError:
        raise HTTPException(status_code=404, detail=f"Log file not found at {file_path}")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


# === 4. NEW DOWNLOAD ENDPOINT ===
@app.get("/api/logs/download")
async def download_log_file(log_name: str):
    """
    Downloads a specified log file as an attachment.
    """
    if log_name not in LOG_FILE_PATHS:
        raise HTTPException(status_code=404, detail=f"No log file path configured for '{log_name}'.")
    
    file_path = LOG_FILE_PATHS[log_name]
    
    if not os.path.exists(file_path):
        raise HTTPException(status_code=404, detail=f"Log file not found at {file_path}")
    
    # Use FileResponse to send the file.
    # This automatically sets the correct headers for a download.
    return FileResponse(
        path=file_path, 
        filename=os.path.basename(file_path), # e.g., "trace.log"
        media_type='text/plain'
    )


@app.get("/api/persistency/activities")
async def get_persistency_api():
    """
    Reads the kvs_0_0.json file, parses the nested JSON strings,
    and returns a clean list of activity objects.
    """
    if PERSISTENCY_FILE_TYPE != "json":
         raise HTTPException(status_code=500, detail="Server configured for wrong persistency type.")
    
    if not os.path.exists(PERSISTENCY_FILE_PATH):
        raise HTTPException(status_code=404, detail=f"Persistency file not found: {PERSISTENCY_FILE_PATH}")
        
    try:
        with open(PERSISTENCY_FILE_PATH, "r") as f:
            outer_data = json.load(f)
        
        data_entries = outer_data.get("v")
        if not data_entries or not isinstance(data_entries, dict):
            raise HTTPException(status_code=500, detail="Invalid JSON structure: 'v' key not found or not an object.")

        activities = []
        for key, entry in data_entries.items():
            if isinstance(entry, dict) and "v" in entry and isinstance(entry["v"], str):
                try:
                    inner_activity = json.loads(entry["v"])
                    activities.append(inner_activity)
                except json.JSONDecodeError:
                    print(f"Warning: Skipping malformed JSON string in key '{key}'")
            
        return {"activities": activities}
        
    except json.JSONDecodeError:
        raise HTTPException(status_code=500, detail=f"Could not parse outer JSON file: {PERSISTENCY_FILE_PATH}")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"An error occurred: {e}")

@app.delete("/api/persistency/clear")
async def clear_persistency_api():
    """
    Clears the persistency data by removing the kvs_0_0.json and kvs_0_0.hash files.
    """
    if PERSISTENCY_FILE_TYPE != "json":
        raise HTTPException(status_code=500, detail="Server configured for wrong persistency type.")
    
    try:
        removed_files = []
        hash_file_path = PERSISTENCY_FILE_PATH.replace('.json', '.hash')
        
        # Remove JSON file
        if os.path.exists(PERSISTENCY_FILE_PATH):
            os.remove(PERSISTENCY_FILE_PATH)
            removed_files.append(PERSISTENCY_FILE_PATH)
        
        # Remove hash file
        if os.path.exists(hash_file_path):
            os.remove(hash_file_path)
            removed_files.append(hash_file_path)
        
        if removed_files:
            return {"status": "success", "message": f"Persistency files removed successfully: {', '.join(removed_files)}"}
        else:
            return {"status": "info", "message": f"No persistency files found to remove"}
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to remove persistency files: {e}")

@app.delete("/api/logs/clear")
async def clear_logs_api(log_name: str = None):
    """
    Clears log files. If log_name is provided, clears that specific log.
    If no log_name is provided, clears all configured log files.
    """
    if log_name:
        # Clear specific log file
        if log_name not in LOG_FILE_PATHS:
            raise HTTPException(status_code=404, detail=f"No log file path configured for '{log_name}'. Available logs are: {list(LOG_FILE_PATHS.keys())}")
        
        file_path = LOG_FILE_PATHS[log_name]
        try:
            if os.path.exists(file_path):
                open(file_path, 'w').close()  # Truncate the file
                return {"status": "success", "message": f"Log file {log_name} ({file_path}) cleared successfully"}
            else:
                return {"status": "info", "message": f"Log file {log_name} ({file_path}) does not exist, nothing to clear"}
        except Exception as e:
            raise HTTPException(status_code=500, detail=f"Failed to clear log file {log_name}: {e}")
    else:
        # Clear all log files
        cleared_logs = []
        errors = []
        
        for log_name, file_path in LOG_FILE_PATHS.items():
            try:
                if os.path.exists(file_path):
                    open(file_path, 'w').close()  # Truncate the file
                    cleared_logs.append(f"{log_name} ({file_path})")
                else:
                    cleared_logs.append(f"{log_name} ({file_path}) - did not exist")
            except Exception as e:
                errors.append(f"Failed to clear {log_name} ({file_path}): {e}")
        
        if errors:
            raise HTTPException(status_code=500, detail=f"Some log files could not be cleared: {'; '.join(errors)}")
        
        return {
            "status": "success", 
            "message": f"All log files cleared successfully",
            "cleared_logs": cleared_logs
        }

# --- 4. WebSocket (Real-time) Endpoints ---

@app.websocket("/ws/metrics")
async def websocket_metrics_endpoint(websocket):
    """Streams all process metrics over a websocket."""
    await websocket.accept()
    try:
        while True:
            # 5. ADD TIMESTAMP TO WEBSOCKET
            timestamp = datetime.datetime.now().isoformat()
            
            process_metrics = []
            for name, proc_data in list(tracked_processes.items()):
                try:
                    proc = proc_data["process"]
                    process_metrics.append({
                        "name": name,
                        "pid": proc.pid,
                        "cpu_percent": proc.cpu_percent(interval=None),
                        "memory_mb": proc.memory_info().rss / (1024 * 1024),
                        "network_io_rate": 0.0,  # Set to 0.0 for individual processes
                        "timestamp": timestamp # <-- ADDED
                    })
                except psutil.NoSuchProcess:
                    if proc_data["log_handle"]:
                        proc_data["log_handle"].close()
                    del tracked_processes[name]
                except Exception:
                    pass 
            
            await websocket.send_json(process_metrics)
            await asyncio.sleep(2) 
    except Exception:
        print("Metrics websocket disconnected.")
    
# --- CORS Configuration ---
app.add_middleware(
    CORSMiddleware,
    allow_origins=ALLOWED_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

if __name__ == "__main__":
    # Parse command line arguments
    args = parse_arguments()
    
    # Get IP from specified interface
    try:
        SERVER_HOST_IP = get_ip_from_interface(args.interface)
        print(f"✓ Using IP {SERVER_HOST_IP} from interface {args.interface}")
    except ValueError as e:
        print(f"ERROR: {e}")
        sys.exit(1)
    
    # Ensure logs directory exists
    ensure_logs_directory()
    
    # Validate all executable paths
    validate_executable_paths()
    
    psutil.cpu_percent(interval=None)
    
    print(f"Starting server on http://{SERVER_HOST_IP}:8000")
    print(f"Allowed Origins: {ALLOWED_ORIGINS}")
    print(f"Tracking logs for: {list(LOG_FILE_PATHS.keys())}")
    uvicorn.run(app, host=SERVER_HOST_IP, port=8000)
