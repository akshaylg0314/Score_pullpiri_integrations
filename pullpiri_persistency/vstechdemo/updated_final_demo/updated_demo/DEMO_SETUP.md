# Pullpiri Demo Setup Guide

This README provides step-by-step instructions to set up and run the Pullpiri demo system, including BlueChi configuration, container image building, NodeAgent setup, and scenario execution.

---

## Prerequisites

- **ETCD must be running on RHIVOS.**
- Verify with:
  ```bash
  etcdctl get "" --prefix
  ```

---


## Step 1: BlueChi Configuration (One-time Setup)


### 1.1 On RHIVOS (Controller + Agent)

1.1.1 **Configure BlueChi Controller and Agent:**
   ```bash
   sudo bash -c "echo -e '[bluechi-controller]\nControllerPort=2020\nAllowedNodeNames=$(hostname),<HOSTNAME_OF_NUC>\n' > /etc/bluechi/controller.conf.d/1.conf"
   # Replace <HOSTNAME_OF_NUC> with the actual NUC hostname

   sudo bash -c "echo -e '[bluechi-agent]\nControllerAddress=unix:path=/run/bluechi/bluechi.sock\n' > /etc/bluechi/agent.conf.d/1.conf"
   ```

   **Example `/etc/bluechi/controller.conf.d/1.conf`:**
   ```
   [bluechi-controller]
   ControllerPort=2020
   AllowedNodeNames=localhost,host,localhost.localdomain
   ```

1.1.2 **Start and enable BlueChi services:**
   ```bash
   sudo systemctl start bluechi-controller bluechi-agent
   sudo systemctl enable bluechi-controller bluechi-agent
   ```

---


### 1.2 On NUC (Managed Agent)

1.2.1 **Get RHIVOS IP address and set up agent:**
   ```bash
   RHIVOS_IP="<RHIVOS_IP_ADDRESS>"

   sudo bash -c "echo -e '[bluechi-agent]\nControllerHost=$RHIVOS_IP\nControllerPort=2020\n' > /etc/bluechi/agent.conf.d/1.conf"
   ```

   **Example `/etc/bluechi/agent.conf.d/1.conf`:**
   ```
   [bluechi-agent]
   ControllerHost=192.168.10.22
   ControllerPort=2020
   ```

1.2.2 **Start and enable BlueChi agent:**
   ```bash
   sudo systemctl start bluechi-agent
   sudo systemctl enable bluechi-agent
   ```

---


## Step 2: Build Container Images (On NUC)

```bash
cd /home/lge/new_demo/updated_demo/container-app/file-message-writer

# Build autonomous threshold manager
sudo podman build -t file-message-writer:latest .

# Verify images
sudo podman images | grep file
```
sudo setenforce 0

**Example output:**
```
localhost/file-message-writer  latest          a6127542d4ae  14 hours ago   89.4 MB

```

---


## Step 3: Configure NodeAgent on NUC

Create `/etc/piccolo/nodeagent.yaml`:

```yaml
nodeagent:
  node_name: "acrn-NUC11TNHi5"           # Replace with actual hostname
  node_type: "vehicle"
  node_role: "bluechi"
  master_ip: "<NUC_IP_ADDRESS>"          # Replace with NUC IP
  node_ip: "<NUC_IP_ADDRESS>"            # Replace with NUC IP
  grpc_port: 47004
  log_level: "info"
  metrics:
    collection_interval: 5
    batch_size: 50
  system:
    hostname: "acrn-NUC11TNHi5"          # Replace with actual hostname
    platform: "Linux"
    architecture: "x86_64"
yaml_storage: "/etc/piccolo/yaml"
```

---



## Step 4: Start Backend Pullpiri on RHIVOS

```bash
sudo systemctl start new_score.service
```
# For Logs: journalctl -u new_score.service -f

---

## Step 5: Setup Backend on NUC

```bash
# Navigate to backend directory
cd /home/lge/new_demo/updated_demo/dds_backend_app

# Create shared data directory
sudo mkdir -p /tmp/driver_distraction
sudo chmod 777 /tmp/driver_distraction
```

## Step 6: Start the Backend

```bash
# Start the file monitoring backend
sudo cargo run
```

## Step 7: Start NodeAgent on NUC

```bash
cd /home/lge/new_demo/updated_demo/pullpiri
sudo ./nodeagent
```

**Example logs:**
```
Loaded configuration from /etc/piccolo/nodeagent.yaml
Starting NodeAgent on host: localhost.localdomain
NodeAgentManager init
NodeAgentManager successfully initialized
NodeAgent listening on 192.168.10.100:47004
NodeAgent config - master_ip: 192.168.10.22, grpc_port: 47004
Successfully registered with API server
```

---

## Step 8: Edit the demo script if required IN RHIVOS

8.1 **Edit the demo script if required:**
   ```bash
   cd /root/new_demo/updated_demo/executables/examples
   ```

   - In `demo_*.sh`, set the IP to the RHIVOS IP:
     ```bash
     curl --location 'http://192.168.10.22:47099/api/artifact' \
     --header 'Content-Type: text/plain' \
     --data "${BODY}"
     ```
   ```

8.2 Run in RHIVOS
```bash
etcdctl del "nodes/0.0.0.0" --prefix
etcdctl del "nodes/HPC" --prefix
```

---

## Step 9: Launch the UI

- See the UI folder for source code and README.
- Start the UI as described there and click on Update as per requirement

---

### Step 10: Verify Service and File Generation

After clicking on update from UI:
```bash
ls /etc/containers/systemd
# Should see:
# driver_distraction_5sec.kube

#### 10.1 Reload the Daemon
sudo systemctl daemon-reload

#### 10.2 Verify the .service file generation
systemctl list-unit-files | grep driver
# Should see:
# driver_distraction_5sec.service  generated
```
---

## Step 11: Troubleshooting

### 11.1 NodeAgent Connection Error

If you see:
```
NodeAgent connection error: Status { code: Unavailable, message: "Failed to connect to NodeAgent at http://0.0.0.0:47004: transport error", source: None }
```
Run:
```bash
etcdctl del "nodes/0.0.0.0" --prefix
etcdctl del "nodes/HPC" --prefix
```
Then retry `./demo_*.sh`.

---
**This README provides a step-by-step guide to set up, run, and verify the Pullpiri demo system.**  