#!/bin/bash

set -e

echo "cleaning up..."
echo "CERTBOT_DOMAIN: $CERTBOT_DOMAIN"
echo "CERTBOT_VALIDATION: $CERTBOT_VALIDATION"
echo "CERTBOT_TOKEN: $CERTBOT_TOKEN"
echo "CERTBOT_REMAINING_CHALLENGES: $CERTBOT_REMAINING_CHALLENGES"

su james -c "
/home/james/bin/doctl compute domain records list $CERTBOT_DOMAIN \
    | grep $CERTBOT_VALIDATION \
    | awk '{print \$1}' \
    | xargs -I {} sh -c '/home/james/bin/doctl compute domain records delete $CERTBOT_DOMAIN {} --force'
"

