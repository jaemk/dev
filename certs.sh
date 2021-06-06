#!/bin/bash

set -e

cmd="$1"

if [ -z "$DOMAIN" ]; then
    echo "DOMAIN must be defined"
fi
if [ -z "$PROJECT_ROOT" ]; then
    echo "PROJECT_ROOT must be defined"
fi
if [ -z "$AUTH_HOOK" ]; then
    echo "AUTH_HOOK must be defined"
fi
if [ -z "$CLEANUP_HOOK" ]; then
    echo "CLEANUP_HOOK must be defined"
fi

if [ -z "$DRY_RUN" ]; then
    echo "DRY_RUN must be defined"
fi

dryrun="--dry-run"
if [[ "$DRY_RUN" = "false" ]]; then
    dryrun=""
fi


if [ "$cmd" = "new" ]; then
    if [ -z "$EMAIL" ]; then
        echo "EMAIL must be defined"
        exit 1
    fi

    set -x
    sudo certbot certonly \
        --manual \
        --email $EMAIL \
        --preferred-challenges=dns \
        --manual-auth-hook $PROJECT_ROOT/$AUTH_HOOK \
        --manual-cleanup-hook $PROJECT_ROOT/$CLEANUP_HOOK \
        --cert-name $DOMAIN \
        -d *.$DOMAIN \
        -d $DOMAIN \
        $dryrun
    set +x

elif [ "$cmd" = "renew" ]; then
    $0 new
    exit 0
elif [ "$cmd" = "copy" ]; then
    echo "only copying..."
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

