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
sudo cp /etc/letsencrypt/live/$DOMAIN/fullchain.pem bin/cert.pem
sudo cp /etc/letsencrypt/live/$DOMAIN/privkey.pem bin/key.pem
# change to -rw-r--r-- permissions
sudo chmod 644 bin/key.pem


# Todo: setup auto dns cert challenge
# https://serverfault.com/questions/750902/how-to-use-lets-encrypt-dns-challenge-validation
# https://www.digitalocean.com/community/questions/change-dns-records-via-doctl
# https://computingforgeeks.com/using-letsencrypt-wildcard-certificate-nginx-apache/
# https://certbot.eff.org/docs/using.html#hooks
#
# ::: doctl compute domain records list jaemk.me
# ::: doctl compute domain records list jaemk.me | rg challenge | awk '{print $1}' | xargs -I {} sh -c 'doctl compute domain records delete jaemk.me {} -f'
