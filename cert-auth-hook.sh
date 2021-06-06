#!/bin/bash

set -e

echo "CERTBOT_DOMAIN: $CERTBOT_DOMAIN"
echo "CERTBOT_VALIDATION: $CERTBOT_VALIDATION"
echo "CERTBOT_TOKEN: $CERTBOT_TOKEN"
echo "CERTBOT_REMAINING_CHALLENGES: $CERTBOT_REMAINING_CHALLENGES"

txtname="_acme-challenge"
echo "TXT: $txtname"

su james -c "
/home/james/bin/doctl compute domain records create $CERTBOT_DOMAIN \
    --record-type TXT \
    --record-name $txtname \
    --record-data $CERTBOT_VALIDATION \
    --record-ttl 30
"
sleep 120

