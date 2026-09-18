#!/bin/sh
# Name: Jobman
# Author: KindleTweaks
# DontUseFBInk

stop statusbar
lipc-set-prop com.lab126.appmgrd start app://com.lab126.webview/?url=x

chmod +x /mnt/us/jobman/bin/sftp-server
chmod +x /mnt/us/jobman/bin/dropbearmulti
chmod +x /mnt/us/jobman/jobman

sleep 2

/mnt/us/jobman/jobman

lipc-set-prop com.lab126.appmgrd start app://com.lab126.booklet.home
start statusbar
xrefresh