#!/bin/sh
# Name: Jobman
# Author: KindleTweaks

# Detach from the library's script launcher, otherwise it can bring the home screen back over jobman
if [ "$1" != "--detached" ]; then
    if command -v setsid > /dev/null 2>&1; then
        setsid sh /mnt/us/documents/jobman.sh --detached > /dev/null 2>&1 < /dev/null &
    else
        nohup sh /mnt/us/documents/jobman.sh --detached > /dev/null 2>&1 < /dev/null &
    fi
    exit 0
fi
sleep 2 # Let the launcher finish and return to the home screen first

# This script *needs* FBInk, otherwise jobman won't launch. Interferes with fb0?
stop statusbar >/dev/null 2>&1
lipc-set-prop com.lab126.appmgrd start app://com.lab126.webview/?url=x

# Trigger installation process: this is also ran by jb.sh, but we keep it here if the user needs to (for whatever reason) download manually, or something is broken so they can re-execute.
if [ ! -f /mnt/us/jobman/JOBMAN_INSTALLED ]; then
    # Ensure binaries are executable
    chmod +x /mnt/us/jobman/bin/sftp-server
    chmod +x /mnt/us/jobman/bin/dropbearmulti
    chmod +x /mnt/us/jobman/jobman

    # Create scp symlink
    if [ ! -L /usr/bin/scp ]; then
        mntroot rw >/dev/null 2>&1
        ln -s /mnt/us/jobman/bin/dropbearmulti /usr/bin/scp >/dev/null 2>&1
        mntroot ro >/dev/null 2>&1
    fi # When using scp with the SSH impl, use -O for the original protocol.

    if [ -f /mnt/us/jobman/INSTALL_RECOVERY ]; then
        mntroot rw

        # Everything in boot converges into this singular file, so we prevent it from starting automatically 
        # to halt it. The jobman_recovery job starts it manually, using initctl.
        sed -i "/start on started system_setup/d" /etc/upstart/system_setup_after_shpm.conf

        cp /mnt/us/jobman/jobman_recovery.conf /etc/upstart/jobman_recovery.conf
        mntroot ro

        rm -f /mnt/us/jobman/INSTALL_RECOVERY
    fi

    if [ -f /mnt/us/jobman/REMOVE_RECOVERY ]; then
        mntroot rw

        if ! grep -q "start on started system_setup" /etc/upstart/system_setup_after_shpm.conf; then
            sed -i "/#This upstart node handles configuring of system after coming out of shipment mode/a start on started system_setup" /etc/upstart/system_setup_after_shpm.conf
        fi

        rm -f /etc/upstart/jobman_recovery.conf
        mntroot ro

        rm -f /mnt/us/jobman/REMOVE_RECOVERY
    fi

    touch /mnt/us/jobman/JOBMAN_INSTALLED
fi

# End of initialisation

# Wait until the webview app is actually in front (up to 15s), then let it finish drawing
for i in $(seq 1 15); do
    [ "$(lipc-get-prop com.lab126.appmgrd activeApp 2>/dev/null)" = "com.lab126.webview" ] && break
    sleep 1
done
sleep 2

nohup /mnt/us/jobman/jobman < /dev/null > /mnt/us/jobman/jobman.log 2>&1 &
JOB_PID=$!

sleep 1
if ! kill -0 $JOB_PID 2>/dev/null; then
    #echo "[Jobman] Failed to start! Check /mnt/us/jobman/jobman.log"
    exit 1
fi

lipc-wait-event -m com.lab126.powerd "*" | while read line; do #Not ideal at all. Will exit jobman when exiting USB SSH, but xrefresh doesn't help, perhaps I need to implement an argument in jobman binary. And for screensavers, jobman still is running and you cannot exit because it's intercepting evdev touch events.
    case "$line" in 
        goingToScreenSaver*)
            #echo "[Jobman] Killing - going to screensaver, will mess up UI!"
            kill $JOB_PID 2>/dev/null
            break 
        ;; 
        # NOT Unconfigured, Rust sends that event.
        usbConfigured*)
            #echo "[Jobman] Killing - going to blanket USB, will mess up UI!"
            kill $JOB_PID 2>/dev/null
            break 
        ;;
    esac
done &
LIPC_PID=$! 

while kill -0 $JOB_PID 2>/dev/null; do
    sleep 1
done

#echo "[Jobman] Stopped..."

kill -9 $JOB_PID 2>/dev/null 
kill $LIPC_PID 2>/dev/null 

lipc-set-prop com.lab126.appmgrd start app://com.lab126.booklet.home
start statusbar >/dev/null 2>&1
xrefresh
