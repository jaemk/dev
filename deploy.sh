set -ex

git rev-parse HEAD | head -c 7 | awk '{ printf "%s", $0 >"commit_hash.txt" }'

cargo build --release
mkdir -p bin

if [[ "$1" = "renew" ]]; then
    ./certs.sh renew
fi

if [ -f bin/dev ]; then
    mv bin/dev bin/dev-running
    cp target/release/dev bin/dev
    sudo systemctl restart dev
    rm bin/dev-running
else
    cp target/release/dev bin/dev
    sudo systemctl restart dev
fi

sudo cp nginx/jaemk_404.html /usr/share/nginx/html/jaemk_404.html
sudo cp nginx/jaemk-ssl-config.conf /etc/nginx/snippets/jaemk-ssl-config.conf
sudo cp nginx/jaemk.conf /etc/nginx/sites-available/jaemk.conf
sudo ln -sf /etc/nginx/sites-available/jaemk.conf /etc/nginx/sites-enabled/jaemk.conf

sudo nginx -t
sudo systemctl enable nginx
sudo systemctl restart nginx

./logs.sh
