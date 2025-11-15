#!/usr/bin/env node

const http = require('http');
const fs = require('fs');
const path = require('path');

// Load environment variables
require('dotenv').config();

const PORT = process.env.SERVER_PORT || 9085;
const SERVER_IP = process.env.SERVER_IP || '0.0.0.0';
const EMERGENCY_FLAG = process.env.EMERGENCY_FLAG_PATH || '/home/acrn/new_ak/demo25/driving_mode/emergency_active';

const server = http.createServer((req, res) => {
  // Enable CORS
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
  
  if (req.method === 'OPTIONS') {
    res.writeHead(200);
    res.end();
    return;
  }

  if (req.url === '/emergency' && req.method === 'GET') {
    try {
      const emergencyActive = fs.existsSync(EMERGENCY_FLAG);
      
      res.setHeader('Content-Type', 'application/json');
      res.writeHead(200);
      res.end(JSON.stringify({ emergency_active: emergencyActive }));
    } catch (error) {
      console.error('Error checking emergency flag:', error);
      res.writeHead(500);
      res.end(JSON.stringify({ error: 'Failed to check emergency status' }));
    }
  } else if (req.url === '/emergency/clear' && req.method === 'POST') {
    try {
      // Clear emergency flag
      if (fs.existsSync(EMERGENCY_FLAG)) {
        fs.unlinkSync(EMERGENCY_FLAG);
        console.log('Emergency flag cleared');
      }
      
      res.setHeader('Content-Type', 'application/json');
      res.writeHead(200);
      res.end(JSON.stringify({ success: true, message: 'Emergency cleared' }));
    } catch (error) {
      console.error('Error clearing emergency flag:', error);
      res.writeHead(500);
      res.end(JSON.stringify({ error: 'Failed to clear emergency' }));
    }
  } else {
    res.writeHead(404);
    res.end('Not Found');
  }
});

server.listen(PORT, SERVER_IP, () => {
  console.log(`Emergency monitor server running on ${SERVER_IP}:${PORT}`);
  console.log(`Monitoring emergency flag: ${EMERGENCY_FLAG}`);
});

// Watch for emergency flag file changes
fs.watchFile(EMERGENCY_FLAG, (curr, prev) => {
  if (curr.mtime !== prev.mtime) {
    console.log('Emergency flag file changed - Emergency activated!');
  }
});