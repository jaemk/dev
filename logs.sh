#!/bin/bash

if [[ "$1" = "dev" ]]; then
    sudo journalctl -fu dev
else
    sudo tail -f /var/log/nginx/access.log
fi
