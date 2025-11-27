#!/bin/bash
# Save as: cleanup_iox2.sh
pkill -9 -f adas
pkill -9 -f iox
sleep 1
find /dev/shm -name "*iox*" -o -name "*feo*" -o -name "*iceoryx*" 2>/dev/null | xargs -r rm -rf
find /tmp -name "*iox*" -o -name "*feo*" 2>/dev/null | xargs -r rm -rf
ipcs -m | grep $USER | awk '{print $2}' | xargs -r -n1 ipcrm -m 2>/dev/null
ipcs -s | grep $USER | awk '{print $2}' | xargs -r -n1 ipcrm -s 2>/dev/null
echo "✅ Iceoryx2 cleaned - ready to run"