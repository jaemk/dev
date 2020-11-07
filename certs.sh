#!/bin/bash

set -e

cmd="$1"

if [ -z "$DOMAIN" ]; then
    echo "DOMAIN must be defined"
fi

if [ "$cmd" = "new" ]; then
    if [ -z "$PROJECT_ROOT" ]; then
        echo "PROJECT_ROOT must be defined"
    fi
    if [ -z "$EMAIL" ]; then
        echo "EMAIL must be defined"
    fi

    sudo certbot certonly \
        -a webroot \
        --webroot-path $PROJECT_ROOT/static \
        --email $EMAIL \
        -d $DOMAIN
elif [ "$cmd" = "renew" ]; then
    sudo certbot renew
else
    echo "command required"
    echo "  $0 new"
    echo "  $0 renew"
    exit 1
fi


echo "copying cert files to bin/"

if [ ! -e bin/cert.pem ]; then
    sudo cp /etc/letsencrypt/live/$DOMAIN/fullchain.pem bin/cert.pem
fi

if [ ! -e bin/key.pem ]; then
    sudo cp /etc/letsencrypt/live/$DOMAIN/privkey.pem bin/key.pem
    # change to -rw-r--r-- permissions
    sudo chmod 644 bin/key.pem
fi

